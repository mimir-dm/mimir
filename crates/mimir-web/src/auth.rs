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

/// The first gate, from the server's auth mode and the stored token.
pub fn gate(mode: AuthMode, stored: Option<&str>) -> Gate {
    match (mode, stored.map(str::trim)) {
        (AuthMode::Open, _) => Gate::Open,
        (AuthMode::Token, Some(t)) if !t.is_empty() => Gate::CheckStored,
        (AuthMode::Token, _) => Gate::SignIn,
    }
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
    fn open_mode_needs_no_token() {
        assert_eq!(gate(AuthMode::Open, None), Gate::Open);
        assert_eq!(gate(AuthMode::Open, Some("x")), Gate::Open);
    }

    #[test]
    fn token_mode_checks_a_stored_token_or_asks_for_one() {
        assert_eq!(gate(AuthMode::Token, Some("abc")), Gate::CheckStored);
        assert_eq!(gate(AuthMode::Token, None), Gate::SignIn);
        assert_eq!(gate(AuthMode::Token, Some("  ")), Gate::SignIn);
    }
}
