//! The DM token: kept in browser storage (`localStorage`), so a tablet stays
//! signed in. The server decides; this module only stores the token and
//! decides what the app shows first.

use mimir_wire::AuthMode;

const TOKEN_KEY: &str = "mimir-dm-token";

/// What the app shows before the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    /// The server needs no token: show the app.
    Open,
    /// A stored token: check it with the server, then show the app.
    CheckStored,
    /// No token: show the sign-in page.
    SignIn,
}

/// The first gate, from the server's auth mode and the stored token. A
/// stored token is checked even in open mode: it can be a player link.
pub fn gate(mode: AuthMode, stored: Option<&str>) -> Gate {
    match (mode, stored.map(str::trim)) {
        (_, Some(t)) if !t.is_empty() => Gate::CheckStored,
        (AuthMode::Open, _) => Gate::Open,
        (AuthMode::Token, _) => Gate::SignIn,
    }
}

/// The token of a player link address (`/play/<token>`).
pub fn play_token(path: &str) -> Option<&str> {
    let t = path.strip_prefix("/play/")?.trim_end_matches('/');
    (!t.is_empty() && !t.contains('/')).then_some(t)
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// The stored DM token.
pub fn stored_token() -> Option<String> {
    storage()?
        .get_item(TOKEN_KEY)
        .ok()?
        .filter(|t| !t.trim().is_empty())
}

/// Store the DM token.
pub fn store_token(token: &str) {
    if let Some(s) = storage() {
        let _ = s.set_item(TOKEN_KEY, token.trim());
    }
}

/// Forget the DM token (sign out, or the server refused it).
pub fn clear_token() {
    if let Some(s) = storage() {
        let _ = s.remove_item(TOKEN_KEY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_mode_needs_no_token_but_checks_a_stored_one() {
        assert_eq!(gate(AuthMode::Open, None), Gate::Open);
        assert_eq!(gate(AuthMode::Open, Some("x")), Gate::CheckStored);
    }

    #[test]
    fn player_link_addresses() {
        assert_eq!(play_token("/play/abc123"), Some("abc123"));
        assert_eq!(play_token("/play/abc123/"), Some("abc123"));
        assert_eq!(play_token("/play/"), None);
        assert_eq!(play_token("/play/a/b"), None);
        assert_eq!(play_token("/maps/x"), None);
    }

    #[test]
    fn token_mode_checks_a_stored_token_or_asks_for_one() {
        assert_eq!(gate(AuthMode::Token, Some("abc")), Gate::CheckStored);
        assert_eq!(gate(AuthMode::Token, None), Gate::SignIn);
        assert_eq!(gate(AuthMode::Token, Some("  ")), Gate::SignIn);
    }
}
