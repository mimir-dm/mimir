//! `mimir-web`: the Mimir web app (Leptos CSR on Aurora), served by
//! mimir-server at `/` (COLLIERY-I-0612, ADR MIMIR-A-0009).
//!
//! - [`app`]: the root component: the sign-in gate, the shell, the router.
//! - [`auth`]: the DM token in browser storage and the gate decision.
//! - [`api`]: a thin client over the same-origin `/api`, with the types of
//!   `mimir-wire`.
//! - [`pages`]: one module per route; [`components`]: pieces they share;
//!   [`markdown`]: safe markdown rendering.
//!
//! Styling: Aurora components and tokens only; `style/mimir.css` holds the
//! few classes Aurora does not have.

pub mod api;
pub mod app;
pub mod auth;
pub mod components;
pub mod live;
pub mod map;
pub mod markdown;
pub mod pages;

pub use app::App;

#[cfg(test)]
mod tests {
    /// index.html carries Aurora's theme script inline (it must run before
    /// the first paint, before the wasm loads). Keep it equal to Aurora's.
    #[test]
    fn index_html_has_auroras_theme_init_script() {
        let html = include_str!("../index.html");
        assert!(
            html.contains(aurora_leptos::theme::THEME_INIT_SCRIPT),
            "index.html theme script differs from aurora THEME_INIT_SCRIPT"
        );
    }
}
