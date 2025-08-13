use serenity::all::{Command, CommandInteraction, Context, CreateCommand, GuildId};
use songbird::input;
use songbird::SerenityInit;

pub async fn register_tone_cmd(ctx: &Context) {
    // Global registration
    let _ = Command::create_global_command(
        &ctx.http,
        CreateCommand::new("tone").description("Join your voice channel and play a short test tone"),
    )
    .await;
}

pub async fn register_tone_guild(ctx: &Context, guild_id: GuildId) {
    // Guild-only registration (instant)
    let _ = Command::create_guild_command(
        &ctx.http,
        guild_id,
        CreateCommand::new("tone").description("Join your voice channel and play a short test tone"),
    )
    .await;
}

pub async fn run_tone(ctx: &Context, cmd: &CommandInteraction) {
    let guild_id = cmd.guild_id.unwrap();
    let guild = guild_id.to_guild_cached(&ctx.cache).unwrap();
    let user_channel = guild
        .voice_states
        .get(&cmd.user.id)
        .and_then(|vs| vs.channel_id);

    if let Some(vc) = user_channel {
        let manager = songbird::get(ctx).await.unwrap().clone();
        let _ = manager.join(guild_id, vc).await;

        if let Ok(src) = input::ffmpeg("sine=frequency=440:duration=5") {
            if let Some(call) = manager.get(guild_id) {
                let mut handler = call.lock().await;
                handler.play_source(src.into());
            }
        }
    }

    let _ = cmd
        .create_response(
            ctx,
            serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new()
                    .content("Playing test tone 🎵"),
            ),
        )
        .await;
}
