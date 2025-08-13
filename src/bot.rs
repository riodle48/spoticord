use std::sync::Arc;

use anyhow::{anyhow, Result};
use log::{debug, info};
use poise::{serenity_prelude, Framework, FrameworkContext, FrameworkOptions};
use serenity::all::{ActivityData, FullEvent, Interaction, Ready, ShardManager, GuildId, Command};
use spoticord_database::Database;
use spoticord_session::manager::SessionManager;

use crate::commands;
// OPTIONAL: if you want /tone, uncomment the next line and keep tone.rs present
// use crate::commands::music::tone;

#[cfg(feature = "stats")]
use spoticord_stats::StatsManager;

pub type Context<'a> = poise::Context<'a, Data, anyhow::Error>;
pub type FrameworkError<'a> = poise::FrameworkError<'a, Data, anyhow::Error>;

type Data = SessionManager;

/// === ONLY the OG commands you showed ===
pub fn framework_opts() -> FrameworkOptions<Data, anyhow::Error> {
    poise::FrameworkOptions {
        commands: vec![
            commands::core::help(),
            commands::core::link(),
            commands::music::join(),
            commands::music::disconnect(),
            commands::music::playing(),
            // OPTIONAL extras you can re-enable:
            // commands::core::version(),
            // commands::core::rename(),
            // commands::core::unlink(),
            // commands::music::stop(),
            // commands::music::lyrics(),
            // commands::music::tone(), // only if you have a poise wrapper for it
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

    // ---- Register commands to your server instantly ----
    let guild_id = GuildId::new(1070519778235658270); // your server ID

    // one-time wipe (set WIPE_COMMANDS=1 in Railway to enable)
    let do_wipe = std::env::var("WIPE_COMMANDS").ok().as_deref() == Some("1");
    if do_wipe {
        info!("Wiping ALL guild commands in {guild_id} …");
        let _ = Command::set_guild_commands(&ctx.http, guild_id, vec![]).await;
    }

    // register OG commands in that guild (instant)
    poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id).await?;
    info!("Re-registered commands in guild {guild_id}");

    // OPTIONAL: if you kept a separate /tone (non-Poise), ensure it exists too
    // tone::register_tone_guild(ctx, guild_id).await;

    // ---- normal bot wiring ----
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

        // if you kept a standalone /tone (not Poise), route it here
        FullEvent::InteractionCreate { interaction } => {
            if let Interaction::Command(_cmd) = interaction {
                // if _cmd.data.name == "tone" { tone::run_tone(ctx, _cmd).await; }
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
