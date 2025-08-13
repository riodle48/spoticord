use crate::Context;

/// `/join` command
pub fn join() -> poise::Command<crate::Data, anyhow::Error> {
    poise::Command {
        name: "join",
        description: "Make the bot join your current voice channel",
        category: None,
        action: |ctx| {
            Box::pin(async move {
                let guild_id = ctx.guild_id().ok_or("You must be in a server")?;
                let channel_id = ctx
                    .author_voice_channel()
                    .ok_or("You must be in a voice channel")?;

                let manager = songbird::get(ctx.serenity_context())
                    .await
                    .ok_or("Songbird Voice client not initialised")?
                    .clone();

                manager.join(guild_id, channel_id).await?;
                ctx.say("Joined your voice channel.").await?;
                Ok(())
            })
        },
        ..Default::default()
    }
}

/// `/disconnect` command
pub fn disconnect() -> poise::Command<crate::Data, anyhow::Error> {
    poise::Command {
        name: "disconnect",
        description: "Disconnect the bot from the voice channel",
        category: None,
        action: |ctx| {
            Box::pin(async move {
                let guild_id = ctx.guild_id().ok_or("You must be in a server")?;
                let manager = songbird::get(ctx.serenity_context())
                    .await
                    .ok_or("Songbird Voice client not initialised")?
                    .clone();

                manager.remove(guild_id).await?;
                ctx.say("Disconnected from voice channel.").await?;
                Ok(())
            })
        },
        ..Default::default()
    }
}

/// `/playing` command
pub fn playing() -> poise::Command<crate::Data, anyhow::Error> {
    poise::Command {
        name: "playing",
        description: "Show what’s currently playing",
        category: None,
        action: |ctx| {
            Box::pin(async move {
                ctx.say("Currently playing: (this will be hooked to Spotify session)").await?;
                Ok(())
            })
        },
        ..Default::default()
    }
}
// ➕ and this
