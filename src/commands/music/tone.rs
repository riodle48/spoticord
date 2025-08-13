use serenity::all::{
    Command, CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
};
use songbird::input;

pub async fn register_tone_cmd(ctx: &Context) {
    let cmds = vec![
        CreateCommand::new("tone")
            .description("Join your voice channel and play a short test tone"),
    ];

    let _ = Command::set_global_commands(&ctx.http, cmds).await;
}

pub async fn run_tone(ctx: &Context, cmd: &CommandInteraction) {
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

    let Some(guild_id) = cmd.guild_id else { return };
    let Some(guild) = guild_id.to_guild_cached(&ctx.cache) else { return };
    let Some(vc) = guild.voice_states.get(&cmd.user.id).and_then(|vs| vs.channel_id) else { return };
    let Some(manager) = songbird::get(ctx).await.map(|m| m.clone()) else { return };

    if manager.join(guild_id, vc).await.is_err() {
        return;
    }

    let test_url = "https://file-examples.com/storage/fe9a7a0e9a8d3a198b1b0aa/2017/11/file_example_MP3_700KB.mp3";

    match input::ffmpeg(test_url) {
        Ok(src) => {
            if let Some(call) = manager.get(guild_id) {
                let mut handler = call.lock().await;
                handler.play_input(src);
                let manager_clone = manager.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(7)).await;
                    let _ = manager_clone.remove(guild_id).await;
                });
            }
        }
        Err(_) => {}
    }
}
