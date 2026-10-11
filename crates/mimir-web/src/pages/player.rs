//! The player page (MIMIR-T-0716, ADR MIMIR-A-0010): what a player link
//! opens. The player display of their campaign, and their character (the
//! sheet joins in MIMIR-T-0717). No app shell, no DM pages.

use aurora_leptos::components::*;
use aurora_leptos::theme::ThemeToggle;
use leptos::prelude::*;
use mimir_wire::Session;

use crate::map::display::DisplayView;

#[component]
pub fn PlayerPage(
    session: Session,
    on_sign_out: Callback<()>,
    on_link_ended: Callback<()>,
) -> impl IntoView {
    let campaign = Signal::stored(session.campaign_id.clone().unwrap_or_default());
    let name = session.character_name.clone().unwrap_or_default();
    view! {
        <div class="mimir-player">
            <header class="mimir-player__bar">
                <span class="mimir-brand">"Mimir"</span>
                <Text bold=true>{name}</Text>
                <div class="mimir-player__actions">
                    <ThemeToggle hyper=true />
                    <Button variant="subtle" on_click=on_sign_out>"Sign out"</Button>
                </div>
            </header>
            <main class="mimir-player__display">
                <DisplayView campaign=campaign framed=true on_signed_out=on_link_ended />
            </main>
        </div>
    }
}
