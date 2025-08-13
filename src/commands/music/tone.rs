// src/commands/music/tone.rs
use serenity::all::{
    Command, CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
};
use songbird::input; // we'll call input::ffmpeg(...).await
// (No need for SerenityInit import here)

/// Register /tone globally via the batch API (stable on serenity 0.12.2).
pub async fn register_tone_cmd(ctx: &Context) {
    let cmds = vec![
        CreateCommand::new("tone")
            .description("Join your voice channel and play a short test tone"),
    ];

    if let Err(err) = Command::set_global_commands(&ctx.http, cmds).await {
        eprintln!("[/tone] failed to register: {err}");
    }
}

/// /tone: joins your VC, plays a tiny MP3 with ffmpeg, then leaves.
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
        let _ = cmd.create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new()
                .content("Run this in a server while you’re in a voice channel.")
                .ephemeral(true),
        ).await;
        return;
    };

    // Guild from cache
    let Some(guild) = guild_id.to_guild_cached(&ctx.cache) else {
        let _ = cmd.create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new()
                .content("Couldn’t read guild from cache yet. Try again in a few seconds.")
                .ephemeral(true),
        ).await;
        return;
    };

    // Caller’s voice channel
    let Some(vc) = guild.voice_states.get(&cmd.user.id).and_then(|vs| vs.channel_id) else {
        let _ = cmd.create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new()
                .content("Join a voice channel first, then run /tone.")
                .ephemeral(true),
        ).await;
        return;
    };

    // Songbird manager
    let Some(manager) = songbird::get(ctx).await.map(|m| m.clone()) else {
        let _ = cmd.create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new()
                .content("Voice client (Songbird) isn’t available. Did you call `.register_songbird()` on the client?")
                .ephemeral(true),
        ).await;
        return;
    };

    // Join VC
    if let Err(e) = manager.join(guild_id, vc).await {
        let _ = cmd.create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new()
                .content(format!("Failed to join voice channel: {e}"))
                .ephemeral(true),
        ).await;
        return;
    }

    // Simple public MP3; requires ffmpeg in PATH and the `ffmpeg` feature (enabled above).
    let test_url = "https://file-examples.com/storage/fe9a7a0e9a8d3a198b1b0aa/2017/11/file_example_MP3_700KB.mp3";

    match input::ffmpeg(test_url).await {
        Ok(src) => {
            if let Some(call) = manager.get(guild_id) {
                let mut handler = call.lock().await;
                handler.play_input(src);

                // Leave after ~7 seconds
                {
                    let manager = manager.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(7)).await;
                        let _ = manager.remove(guild_id).await;
                    });
                }

                let _ = cmd.create_followup(
                    &ctx.http,
                    CreateInteractionResponseFollowup::new()
                        .content("🔊 Playing! I’ll disconnect shortly.")
                        .ephemeral(true),
                ).await;
            } else {
                let _ = cmd.create_followup(
                    &ctx.http,
                    CreateInteractionResponseFollowup::new()
                        .content("Joined, but couldn’t access the call handler.")
                        .ephemeral(true),
                ).await;
            }
        }
        Err(err) => {
            let _ = cmd.create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .content(format!("ffmpeg failed to start (is ffmpeg installed in the container?): {err}"))
                    .ephemeral(true),
            ).await;
        }
    }
}
