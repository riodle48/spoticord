use std::sync::Arc;

use anyhow::{anyhow, Result};
use log::{debug, info};
use poise::{serenity_prelude, Framework, FrameworkContext, FrameworkOptions};
use serenity::all::{ActivityData, FullEvent, Interaction, Ready, ShardManager};
use spoticord_database::Database;
use spoticord_session::manager::SessionManager;

use crate::commands;
use crate::commands::music::tone;

#[cfg(feature = "stats")]
use spoticord_stats::StatsManager;

pub type Context<'a> = poise::Context<'a, Data, anyhow::Error>;
pub type FrameworkError<'a> = poise::FrameworkError<'a, Data, anyhow::Error>;

type Data = SessionManager;

pub fn framework_opts() -> FrameworkOptions<Data, anyhow::Error> {
    poise::FrameworkOptions {
        commands: vec![
            #[cfg(debug_assertions)]
            commands::debug::ping(),
            #[cfg(debug_assertions)]
            commands::debug::token(),
            commands::core::help(),
            commands::core::version(),
            commands::core::rename(),
            commands::core::link(),
            commands::core::unlink(),
            commands::music::join(),
            commands::music::disconnect(),
            commands::music::stop(),
            commands::music::playing(),
            commands::music::lyrics(),
        ],
        event_handler: |ctx, event, framework, data| Box::pin(event_handler(ctx, event, framework, data)),
        ..Default::default()
    }
}

pub async fn setup(
    ctx: &serenity_prelude::Context,
    ready: &Ready,
    framework: &Framework<Data, anyhow::Error>,
    database: Database,
) -> Result<Data> {
    info!("Successfully logged in as {}", ready.user.name);

    // Register ALL Poise commands in one guild (instant) or globally (slower)
    if let Ok(gid) = std::env::var("GUILD_ID").ok().and_then(|s| s.parse::<u64>().ok()) {
        poise::builtins::register_in_guild(
            ctx,
            &framework.options().commands,
            serenity::all::GuildId::new(gid),
        ).await?;
        info!("Registered Poise commands in guild {}", gid);

        // Also add /tone to that guild (does NOT overwrite others)
        tone::register_tone_guild(ctx, serenity::all::GuildId::new(gid)).await;
    } else {
        poise::builtins::register_globally(ctx, &framework.options().commands).await?;
        info!("Registered Poise commands globally (may take minutes)");
        tone::register_tone_cmd(ctx).await; // global /tone
    }

    // Songbird handle
    let songbird = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow!("Songbird was not registered during setup"))?;

    let manager = SessionManager::new(songbird, database);

    #[cfg(feature = "stats")]
    let stats = StatsManager::new(spoticord_config::kv_url())?;

    tokio::spawn(background_loop(
        manager.clone(),
        framework.shard_manager().clone(),
        #[cfg(feature = "stats")]
        stats,
    ));

    Ok(manager)
}

async fn event_handler(
    ctx: &serenity_prelude::Context,
    event: &FullEvent,
    _framework: FrameworkContext<'_, Data, anyhow::Error>,
    _data: &Data,
) -> Result<()> {
    match event {
        FullEvent::Ready { data_about_bot } => {
            if let Some(shard) = data_about_bot.shard {
                debug!("Shard {} logged in (total shards: {})", shard.id.0, shard.total);
            }
            ctx.set_activity(Some(ActivityData::listening(spoticord_config::MOTD)));
        }
        FullEvent::InteractionCreate { interaction } => {
            if let Interaction::Command(cmd) = interaction {
                if cmd.data.name == "tone" {
                    tone::run_tone(ctx, cmd).await;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn background_loop(
    session_manager: SessionManager,
    shard_manager: Arc<ShardManager>,
    #[cfg(feature = "stats")] mut stats_manager: spoticord_stats::StatsManager,
) {
    #[cfg(feature = "stats")]
    use log::error;

    loop {
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
                #[cfg(feature = "stats")]
                {
                    let mut count = 0;
                    for session in session_manager.get_all_sessions() {
                        if matches!(session.active().await, Ok(true)) { count += 1; }
                    }
                    if let Err(why) = stats_manager.set_active_count(count) {
                        error!("Failed to update active sessions: {why}");
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                session_manager.shutdown_all().await;
                shard_manager.shutdown_all().await;
                #[cfg(feature = "stats")]
                let _ = stats_manager.set_active_count(0);
                break;
            }
        }
    }
}
