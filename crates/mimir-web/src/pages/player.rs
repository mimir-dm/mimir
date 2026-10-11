//! The player page (MIMIR-T-0716, ADR MIMIR-A-0010): what a player link
//! opens. The player display of their campaign, their character (the
//! sheet, MIMIR-T-0717) and the party (MIMIR-T-0718). No app shell, no DM
//! pages.

use aurora_leptos::components::*;
use aurora_leptos::theme::ThemeToggle;
use leptos::prelude::*;
use mimir_wire::Session;

use aurora_leptos::frame::{TabItem, TabPanel, Tabs};

use crate::character::party::PartyPanel;
use crate::character::sheet::{CharacterSheetView, SheetAccess};
use crate::map::display::DisplayView;

#[component]
pub fn PlayerPage(
    session: Session,
    on_sign_out: Callback<()>,
    on_link_ended: Callback<()>,
) -> impl IntoView {
    let campaign = Signal::stored(session.campaign_id.clone().unwrap_or_default());
    let name = session.character_name.clone().unwrap_or_default();
    let character = Signal::stored(session.character_id.clone().unwrap_or_default());
    let tab = RwSignal::new("display".to_string());
    let tabs = vec![
        TabItem::new("display", "Display"),
        TabItem::new("character", "Character"),
        TabItem::new("party", "Party"),
    ];
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
            <div class="mimir-player__tabs">
                <Tabs tabs=tabs value=tab label="Player sections">
                    <TabPanel value="display">
                        <main class="mimir-player__display">
                            <DisplayView campaign=campaign framed=true on_signed_out=on_link_ended />
                        </main>
                    </TabPanel>
                    <TabPanel value="character">
                        <div class="mimir-player__sheet">
                            <CharacterSheetView id=character access=SheetAccess::Owner />
                        </div>
                    </TabPanel>
                    <TabPanel value="party">
                        <div class="mimir-player__sheet">
                            <PartyPanel campaign=campaign />
                        </div>
                    </TabPanel>
                </Tabs>
            </div>
        </div>
    }
}
