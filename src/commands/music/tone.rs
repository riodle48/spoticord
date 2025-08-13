use serenity::all::*;
use songbird::{input, SerenityInit};

/// Registers the /tone slash command globally.
/// Call this once on startup (e.g., in your `Ready` event).
pub async fn register_tone_cmd(ctx: &Context) {
    if let Err(err) = Command::create_global_application_command(&ctx.http, |c| {
        c.name("tone")
            .description("Join your voice channel and play a short test tone")
    })
    .await
    {
        eprintln!("[/tone] failed to register: {err}");
    }
}

/// Handles /tone: joins the user's VC and plays a tiny MP3 via ffmpeg.
/// - Requires ffmpeg in your runtime.
/// - Requires Songbird to be initialized on the client.
pub async fn run_tone(ctx: &Context, cmd: &CommandInteraction) {
    // Respond immediately so Discord doesn't show "did not respond"
    let _ = cmd
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("🎧 Joining and playing a short test tone...")
                    .ephemeral(true),
            ),
        )
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

    // Get the guild from cache (may be None right after startup)
    let Some(guild) = guild_id.to_guild_cached(&ctx.cache) else {
        let _ = cmd
            .create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .content("Couldn’t read guild from cache yet. Try again in a few seconds.")
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
                    .content("Join a voice channel first, then run /tone.")
                    .ephemeral(true),
            )
            .await;
        return;
    };

    // Get Songbird manager
    let Some(manager) = songbird::get(ctx).await.map(|m| m.clone()) else {
        let _ = cmd
            .create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .content("Voice client (Songbird) isn’t available. Did you add `SerenityInit` to the client?")
                    .ephemeral(true),
            )
            .await;
        return;
    };

    // Try to join VC
    if let Err(join_err) = manager.join(guild_id, vc).await {
        let _ = cmd
            .create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .content(format!("Failed to join voice channel: {join_err}"))
                    .ephemeral(true),
            )
            .await;
        return;
    }

    // Small public MP3 (avoids the `lavfi` args issue with sine). Needs ffmpeg in the container/host.
    let test_url = "https://file-examples.com/storage/fe9a7a0e9a8d3a198b1b0aa/2017/11/file_example_MP3_700KB.mp3";

    match input::ffmpeg(test_url) {
        Ok(src) => {
            if let Some(call) = manager.get(guild_id) {
                let mut handler = call.lock().await;
                handler.play_input(src);

                // Optionally, leave after ~7 seconds so you don't linger in VC.
                {
                    let manager = manager.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(7)).await;
                        let _ = manager.remove(guild_id).await;
                    });
                }

                let _ = cmd
                    .create_followup(
                        &ctx.http,
                        CreateInteractionResponseFollowup::new()
                            .content("🔊 Playing! I’ll disconnect shortly.")
                            .ephemeral(true),
                    )
                    .await;
            } else {
                let _ = cmd
                    .create_followup(
                        &ctx.http,
                        CreateInteractionResponseFollowup::new()
                            .content("Joined, but couldn’t access the call handler.")
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
                            "ffmpeg failed to start (is ffmpeg installed in your runtime?): {err}"
                        ))
                        .ephemeral(true),
                )
                .await;
        }
    }
}
