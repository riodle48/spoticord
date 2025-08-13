use serenity::all::{
    Command, CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, GuildId,
};
use songbird::input;
use songbird::SerenityInit;

/// Global /tone (slow propagation)
pub async fn register_tone_cmd(ctx: &Context) {
    let _ = Command::create_global_command(
        &ctx.http,
        CreateCommand::new("tone").description("Join your voice channel and play a short test"),
    ).await;
}

/// Guild-only /tone (instant, does NOT overwrite other commands)
pub async fn register_tone_guild(ctx: &Context, guild_id: GuildId) {
    let _ = Command::create_guild_command(
        &ctx.http,
        guild_id,
        CreateCommand::new("tone").description("Join your voice channel and play a short test"),
    ).await;
}

pub async fn run_tone(ctx: &Context, cmd: &CommandInteraction) {
    // simple feedback
    let _ = cmd.create_response(
        &ctx.http,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().content("Joining and playing a short test…"),
        ),
    ).await;

    let Some(guild_id) = cmd.guild_id else { return; };
    let Some(guild) = guild_id.to_guild_cached(&ctx.cache) else { return; };
    let vc = guild.voice_states.get(&cmd.user.id).and_then(|vs| vs.channel_id);
    let Some(channel_id) = vc else { return; };

    let Some(manager) = songbird::get(ctx).await.map(|m| m.clone()) else { return; };
    let _ = manager.join(guild_id, channel_id).await;

    // Use a tiny public MP3 through ffmpeg (your image installs ffmpeg)
    if let Ok(src) = input::ffmpeg("https://www.kozco.com/tech/piano2-CoolEdit.mp3") {
        if let Some(call) = manager.get(guild_id) {
            let mut handler = call.lock().await;
            handler.play_source(src.into());
        }
    }

    // leave after ~10s
    let mgr = manager.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let _ = mgr.remove(guild_id).await;
    });
}
