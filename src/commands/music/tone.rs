use serenity::all::{
    Command, CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
    GuildId, UserId,
};

pub async fn register_tone_cmd(ctx: &Context) {
    let cmds = vec![
        CreateCommand::new("tone")
            .description("Join your voice channel and run a quick test"),
    ];
    let _ = Command::set_global_commands(&ctx.http, cmds).await;
}

pub async fn run_tone(ctx: &Context, cmd: &CommandInteraction) {
    // Acknowledge immediately
    let _ = cmd.create_response(
        &ctx.http,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("Joining your voice channel for a quick test (no audio).")
                .ephemeral(true),
        ),
    ).await;

    // Must be in a guild
    let Some(guild_id) = cmd.guild_id else {
        let _ = ephemeral_followup(ctx, cmd, "Run this in a server while you are in a voice channel.").await;
        return;
    };

    // ---- DO NOT HOLD CACHE GUARDS ACROSS AWAITS ----
    // Extract just the channel id we need, then drop cache refs before any await.
    let caller_id: UserId = cmd.user.id;
    let user_vc: Option<_> = user_voice_channel_from_cache(&ctx, guild_id, caller_id);

    let Some(vc) = user_vc else {
        let _ = ephemeral_followup(ctx, cmd, "Join a voice channel first, then run /tone.").await;
        return;
    };

    // Songbird manager
    let Some(manager) = songbird::get(ctx).await.map(|m| m.clone()) else {
        let _ = ephemeral_followup(ctx, cmd, "Songbird is not available. Did you call .register_songbird()?").await;
        return;
    };

    // Join VC
    if let Err(e) = manager.join(guild_id, vc).await {
        let _ = ephemeral_followup(ctx, cmd, &format!("Failed to join voice channel: {e}")).await;
        return;
    }

    // Leave after ~7 seconds
    let m2 = manager.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(7)).await;
        let _ = m2.remove(guild_id).await;
    });

    let _ = ephemeral_followup(ctx, cmd, "Joined. I will disconnect shortly (no audio).").await;
}

// Helper: read the caller's VC from cache WITHOUT holding a CacheRef across await.
fn user_voice_channel_from_cache(
    ctx: &Context,
    guild_id: GuildId,
    user_id: UserId,
) -> Option<serenity::all::ChannelId> {
    // Take a snapshot of just the needed field, then drop the guard.
    let voice_states = guild_id
        .to_guild_cached(&ctx.cache)
        .map(|g| g.voice_states.clone())?; // clone to avoid holding the cache guard

    voice_states.get(&user_id).and_then(|vs| vs.channel_id)
}

// Small helper to keep awaits localized and strings ASCII-safe.
async fn ephemeral_followup(
    ctx: &Context,
    cmd: &CommandInteraction,
    msg: &str,
) -> serenity::Result<()> {
    cmd.create_followup(
        &ctx.http,
        CreateInteractionResponseFollowup::new()
            .content(msg)
            .ephemeral(true),
    ).await
}
