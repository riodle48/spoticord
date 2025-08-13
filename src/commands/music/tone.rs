use serenity::all::*;
use songbird::input;
use songbird::SerenityInit;
use std::sync::Arc;

pub async fn register_tone_cmd(ctx: &Context) {
    let _ = Command::create_global_application_command(&ctx.http, |c| {
        c.name("tone").description("Play a 5-second test tone")
    }).await;
}

pub async fn run_tone(ctx: &Context, cmd: &CommandInteraction) {
    let guild_id = cmd.guild_id.unwrap();
    let guild = guild_id.to_guild_cached(&ctx.cache).unwrap();
    let user_channel = guild.voice_states.get(&cmd.user.id).and_then(|vs| vs.channel_id);

    if let Some(vc) = user_channel {
        let manager = songbird::get(ctx).await.unwrap().clone();
        let _ = manager.join(guild_id, vc).await;

        if let Ok(src) = input::ffmpeg("sine=frequency=440:duration=5") {
            if let Some(call) = manager.get(guild_id) {
                let mut handler = call.lock().await;
                handler.play_input(src);
            }
        }

        let _ = cmd.create_response(&ctx.http, CreateInteractionResponse::Acknowledge).await;
    } else {
        let _ = cmd.create_response(&ctx.http, |r| {
            r.kind(InteractionResponseType::ChannelMessageWithSource)
             .interaction_response_data(|d| d.content("Join a voice channel first.").ephemeral(true))
        }).await;
    }
}
