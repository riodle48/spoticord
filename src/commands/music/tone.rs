use serenity::all::*;
use songbird::{input, SerenityInit};

/// Registers the /tone slash command globally
pub async fn register_tone_cmd(ctx: &Context) {
    let _ = Command::create_global_application_command(&ctx.http, |c| {
        c.name("tone").description("Play a 5-second test tone")
    })
    .await;
}

/// Handles /tone: joins your VC and plays a tiny MP3 via ffmpeg (bypasses Spotify)
pub async fn run_tone(ctx: &Context, cmd: &CommandInteraction) {
    // Acknowledge right away so Discord doesn't show "did not respond"
    let _ = cmd
        .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
        .await;

    // Must be in a guild
    let Some(guild_id) = cmd.guild_id else {
        let _ = cmd
            .create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .content("Run this in a server while you’re in a voice channel.")
                    .ephemeral(true),
            )
            .await;
        return;
    };

    // Get guild from cache
    let Some(guild) = guild_id.to_guild_cached(&ctx.cache) else {
        let _ = cmd
            .create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .content("Couldn't read guild from cache. Try again.")
                    .ephemeral(true),
            )
            .await;
        return;
    };

    // Find caller's current voice channel
    let user_channel = guild
        .voice_states
        .get(&cmd.user.id)
        .and_then(|vs| vs.channel_id);

    let Some(vc) = user_channel else {
        let _ = cmd
            .create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .content("Join a voice channel first.")
                    .ephemeral(true),
            )
            .await;
        return;
    };

    // Join VC
    let manager = match songbird::get(ctx).await {
        Some(m) => m.clone(),
        None => {
            let _ = cmd
                .create_followup(
                    &ctx.http,
                    CreateInteractionResponseFollowup::new()
                        .content("Voice client (Songbird) not available.")
                        .ephemeral(true),
                )
                .await;
            return;
        }
    };

    let _ = manager.join(guild_id, vc).await;

    // 🔈 Stream a tiny public MP3 (simpler than lavfi sine)
    // Requires ffmpeg + libopus in the runtime container.
    let test_url = "https://file-examples.com/storage/fe9a7a0e9a8d3a198b1b0aa/2017/11/file_example_MP3_700KB.mp3";

    match input::ffmpeg(test_url) {
        Ok(src) => {
            if let Some(call) = manager.get(guild_id) {
                let mut handler = call.lock().await;
                handler.play_input(src);
            } else {
                let _ = cmd
                    .create_followup(
                        &ctx.http,
                        CreateInteractionResponseFollowup::new()
                            .content("Couldn’t access the voice call handler.")
                            .ephemeral(true),
                    )
                    .await;
            }
        }
        Err(err) => {
            let _ = cmd
                .create_followup(
                    &ctx.http,
                    CreateInteractionResponseFollowup::new()
                        .content(format!(
                            "ffmpeg failed to start (is it installed in the container?): {err}"
                        ))
                        .ephemeral(true),
                )
                .await;
        }
    }
}
