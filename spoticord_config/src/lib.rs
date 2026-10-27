mod env;

use rspotify::{AuthCodePkceSpotify, Credentials, Token};
use serenity::all::GatewayIntents;

#[cfg(not(debug_assertions))]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(debug_assertions)]
pub const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "-dev");

/// The "listening to" message that shows up under the Spoticord bot user
pub const MOTD: &str = "some good 'ol music";

/// The time it takes (in seconds) for Spoticord to disconnect when no music is being played
pub const DISCONNECT_TIME: u64 = 5 * 60;

pub fn discord_token() -> &'static str {
    &env::DISCORD_TOKEN
}

pub fn discord_intents() -> GatewayIntents {
    GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES
}

pub fn database_url() -> &'static str {
    &env::DATABASE_URL
}

pub fn link_url() -> &'static str {
    &env::LINK_URL
}

pub fn kv_url() -> &'static str {
    &env::KV_URL
}

/// The Spotify application client ID. librespot must log in with the same client ID
/// that issued the user's access tokens, otherwise login5 rejects it with INVALID_CREDENTIALS.
pub fn spotify_client_id() -> &'static str {
    &env::SPOTIFY_CLIENT_ID
}

/// librespot's own ("keymaster") Spotify client ID. Spotify only lets sessions authenticated with
/// tokens issued to its own clients use Connect, so users link their account through this client
/// (PKCE, no secret) rather than through our developer application.
pub const LIBRESPOT_CLIENT_ID: &str = "65b708073fc0480ea92a077233ca87bd";

pub fn librespot_client_id() -> String {
    std::env::var("SPOTIFY_OAUTH_CLIENT_ID").unwrap_or_else(|_| LIBRESPOT_CLIENT_ID.to_string())
}

/// Client for refreshing a user's tokens. Tokens are issued to the librespot client via PKCE,
/// so refreshing needs that client ID and no secret.
pub fn get_spotify(token: Token) -> AuthCodePkceSpotify {
    let mut spotify = AuthCodePkceSpotify::from_token(token);
    spotify.creds = Credentials::new_pkce(&librespot_client_id());
    spotify
}
