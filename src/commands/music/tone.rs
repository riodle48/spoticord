use serenity::all::{
    Command, CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
};

/// Register /tone globally (serenity 0.12.2).
pub async fn register_tone_cmd(ctx: &Context) {
    let cmds = vec![
        CreateCommand::new("tone")
            .description("Join your voice channel, wait a few seconds, then leave"),
    ];
    let _ = Command::set_global_commands(&ctx.http, cmds).await;
}

/// /tone: join VC, wait ~7s, then leave (no audio).
pub async fn run_tone(ctx: &Context, cmd: &CommandInteraction) {
    // Acknowledge immediately (ephemeral)
    let _ = cmd
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("🎧 Joining your voice channel for a quick test (no audio), then disconnecting…")
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

    // Need the caller’s current voice channel
    let Some(guild) = guild_id.to_guild_cached(&ctx.cache) else {
        let _ = cmd.create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new()
                .content("Couldn’t read guild from cache yet. Try again in a few seconds.")
                .ephemeral(true),
        ).await;
        return;
    };

    let Some(vc) = guild.voice_states.get(&cmd.user.id).and_then(|vs| vs.channel_id) else {
        let _ = cmd.create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new()
                .content("Join a voice channel first, then run /tone.")
                .ephemeral(true),
        ).await;
        return;
    };

    // Songbird manager (requires `.register_songbird()` on your client)
    let Some(manager) = songbird::get(ctx).await.map(|m| m.clone()) else {
        let _ = cmd.create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new()
                .content("Songbird isn’t ava
