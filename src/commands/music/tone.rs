use serenity::all::{
    Command, CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
};

/// Register /tone globally (serenity 0.12.2).
pub async fn register_tone_cmd(ctx: &Context) {
    let cmds = vec![
        CreateCommand::new("tone")
            .description("Join your voice channel and play a short test tone"),
    ];
    let _ = Command::set_global_commands(&ctx.http, cmds).await;
}

/// /tone: join VC, play tiny MP3 via ffmpeg, then leave.
pub async fn run_tone(ctx: &Context, cmd: &CommandInteraction) {
    // Respond immediately so Discord doesn't timeout
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

    // Songbird manager (requires `.register_songbird()` when building the client)
    let Some(manager) = songbird::get(ctx).await.map(|m| m.clone()) else {
        let _ = cmd.create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new()
                .content("Songbird isn’t available. Did you call `.register_songbird()`?")
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

    // Tiny public MP3 (requires ffmpeg binary in PATH)
    let test_url = "https://file-examples.com/storage/fe9a7a0e9a8d3a198b1b0aa/2017/11/file_example_MP3_700KB.mp3";

    // ffmpeg helper from Songbird
    match songbird::input::ffmpeg(test_url) {
        Ok(src) => {
            if let Some(call) = manager.get(guild_id) {
                let mut handler = call.lock().await;
                handler.play_input(src);

                // Leave after ~7 seconds
                let m2 = manager.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(7)).await;
                    let _ = m2.remove(guild_id).await;
                });

                let _ = cmd.create_followup(
                    &ctx.http,
                    CreateInteractionResponseFollowup::new()
                        .content("🔊 Playing! Disconnecting shortly...")
                        .ephemeral(true),
                ).await;
            }
        }
        Err(err) => {
            let _ = cmd.create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .content(format!("ffmpeg failed to start (is it installed?): {err}"))
                    .ephemeral(true),
            ).await;
        }
    }
}
