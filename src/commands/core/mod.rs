use crate::Context;

/// `/help` command
pub fn help() -> poise::Command<crate::Data, anyhow::Error> {
    poise::Command {
        name: "help",
        description: "Show help information about the bot",
        category: None,
        ..poise::builtins::help()
    }
}

/// `/link` command
pub fn link() -> poise::Command<crate::Data, anyhow::Error> {
    poise::Command {
        name: "link",
        description: "Link your Spotify account to Spoticord",
        category: None,
        action: |ctx| {
            Box::pin(async move {
                ctx.say("Click here to link your Spotify account: https://spoticord.com/link").await?;
                Ok(())
            })
        },
        ..Default::default()
    }
}
