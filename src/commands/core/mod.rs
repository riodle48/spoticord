pub mod core;
pub mod music;

// Uncomment if you want debug-only commands
// pub mod debug;

use poise::Command;

/// Returns all commands for the bot
pub fn all_commands() -> Vec<Command<crate::Data, anyhow::Error>> {
    vec![
        // ==== CORE ====
        core::help(),
        core::link(),
        // core::version(),
        // core::rename(),
        // core::unlink(),

        // ==== MUSIC ====
        music::join(),
        music::disconnect(),
        music::playing(),
        // music::stop(),
        // music::lyrics(),
        // music::tone(),
    ]
}
