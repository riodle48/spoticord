use anyhow::Result;
use serenity::all::*;
use songbird::SerenityInit;

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("logged in as {}", ready.user.name);

        // register /tone globally
        let _ = Command::set_global_commands(
            &ctx.http,
            vec![
                CreateCommand::new("tone")
                    .description("Join your voice channel for a quick test (no audio)"),
            ],
        )
        .await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(cmd) = interaction {
            if cmd.data.name == "tone" {
                let _ = cmd.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("Joining your voice channel (no audio)…")
                            .ephemeral(true),
                    ),
                ).await;

                // must be in a guild
                let Some(guild_id) = cmd.guild_id else { return; };

                // grab the user's current VC (clone to avoid holding cache guard across await)
                let vc = guild_id
                    .to_guild_cached(&ctx.cache)
                    .map(|g| g.voice_states.clone())
                    .and_then(|vs| vs.get(&cmd.user.id).and_then(|s| s.channel_id))
                    ;

                let Some(vc) = vc else {
                    let _ = cmd.create_followup(&ctx.http,
                        CreateInteractionResponseFollowup::new()
                            .content("Join a voice channel first, then run /tone.")
                            .ephemeral(true)
                    ).await;
                    return;
                };

                // songbird manager
                let Some(manager) = songbird::get(&ctx).await.map(|m| m.clone()) else { return; };

                if manager.join(guild_id, vc).await.is_ok() {
                    // leave after ~7s
                    let m2 = manager.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(7)).await;
                        let _ = m2.remove(guild_id).await;
                    });

                    let _ = cmd.create_followup(&ctx.http,
                        CreateInteractionResponseFollowup::new()
                            .content("Joined. I will disconnect shortly (no audio).")
                            .ephemeral(true)
                    ).await;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let token = std::env::var("DISCORD_TOKEN")?;
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(&token, intents)
        .register_songbird() // required for voice
        .event_handler(Handler)
        .await?;

    client.start().await?;
    Ok(())
}
