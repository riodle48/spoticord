// src/commands/music/tone.rs
use serenity::all::{
    ChannelId, Command, CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, GuildId, UserId,
};
use songbird::input;

pub async fn register_tone_cmd(ctx: &Context) {
    let cmds = vec![CreateCommand::new("tone")
        .description("Join your voice channel and play a short test tone")];
    let _ = Command::set_global_commands(&ctx.http, cmds).await;
}

pub async fn run_tone(ctx: &Context, cmd: &CommandInteraction) {
    // Acknowledge immediately
    let _ = cmd
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Joining your voice channel and playing a short test…")
                    .ephemeral(true),
            ),
        )
        .await;

    // Must be in a guild
    let Some(guild_id) = cmd.guild_id else {
        let _ = ephemeral_followup(ctx, cmd, "Run this in a server while you are in a voice channel.").await;
        return;
    };

    // Avoid holding cache guards across awaits
    let caller_id: UserId = cmd.user.id;
    let Some(vc) = user_voice_channel_from_cache(ctx, guild_id, caller_id) else {
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

    // --- Play a short MP3 via the symphonia decoder (no ffmpeg needed) ---
    // Feel free to swap this URL with any small MP3/OGG over HTTPS.
    let test_url = "https://file-examples.com/storage/fe9a7a0e9a8d3a198b1b0aa/2017/11/file_example_MP3_700KB.mp3";

    // Build a restartable HTTP source that symphonia can decode:
    // - Use "ytdl:false" so it fetches directly without requiring yt-dlp
    // - This yields a decoder-backed Input which Songbird can play.
    let source = match input::Restartable::new_http(test_url.to_string(), input::HttpRequest::default(), false).await {
        Ok(s) => s,
        Err(err) => {
            let _ = ephemeral_followup(ctx, cmd, &format!("Failed to create audio source: {err}")).await;
            // Leave if we couldn't start audio
            let _ = manager.remove(guild_id).await;
            return;
        }
    };

    if let Some(call) = manager.get(guild_id) {
        let mut handler = call.lock().await;
        handler.play_source(source.into());
    }

    // Disconnect after ~10 seconds so we don't linger
    let m2 = manager.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let _ = m2.remove(guild_id).await;
    });

    let _ = ephemeral_followup(ctx, cmd, "🔊 Playing! I will disconnect shortly.").await;
}

// Read the caller's VC from cache without holding a CacheRef across await.
fn user_voice_channel_from_cache(
    ctx: &Context,
    guild_id: GuildId,
    user_id: UserId,
) -> Option<ChannelId> {
    let voice_states = guild_id
        .to_guild_cached(&ctx.cache)
        .map(|g| g.voice_states.clone())?; // clone to drop cache guard
    voice_states.get(&user_id).and_then(|vs| vs.channel_id)
}

// Return () even though create_followup returns Message
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
    )
    .await
    .map(|_| ())
}
