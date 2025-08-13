use serenity::all::{
    Command, CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
};

/// Register /tone globally (serenity 0.12.2).
pub async fn register_tone_cmd(ctx: &Context) {
    let cmds = vec![
        CreateCommand::new("tone")
            .description("Join your voice channel and run a quick test"),
    ];
    let _ = Command::set_global_commands(&ctx.http, cmds).await;
}

/// /tone: join VC, wait ~7s, then leave (no audio).
pub async fn run_tone(ctx: &Context, cmd: &CommandInteraction) {
    // Acknowledge immediately
    let _ = cmd
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Joining your voice channel for a quick test (no audio).")
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
                    .content("Run this in a server while you are in a voice channel.")
                    .ephemeral(true),
            )
            .await;
        return;
    };

    // Find caller's current voice channel
    let Some(guild) = guild_id.to_guild_cached(&ctx.cache) else {
        let _ = cmd
            .create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .content("Could not read guild from cache yet. Try again in a few seconds.")
                    .ephemeral(true),
            )
            .await;
        return;
    };

    let Some(vc) = guild
        .voice_states
        .get(&cmd.user.id)
        .and_then(|vs| vs.channel_id)
    else {
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

    // Songbird manager (requires `.register_songbird()` on your client)
    let Some(manager) = songbird::get(ctx).await.map(|m| m.clone()) else {
        let _ = cmd
            .create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .content("Songbird is not available. Did you call .register_songbird()?")
                    .ephemeral(true),
            )
            .await;
        return;
    };

    // Join VC
    if let Err(e) = manager.join(guild_id, vc).await {
        let _ = cmd
            .create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .content(format!("Failed to join voice channel: {e}"))
                    .ephemeral(true),
            )
            .await;
        return;
    }

    // Leave after ~7 seconds
    let m2 = manager.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(7)).await;
        let _ = m2.remove(guild_id).await;
    });

    let _ = cmd
        .create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new()
                .content("Joined. I will disconnect shortly (no audio).")
                .ephemeral(true),
        )
        .await;
}
