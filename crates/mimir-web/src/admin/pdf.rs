//! The PDF dialog (MIMIR-T-0724): what to include, then a download. The
//! options are the desktop's (their query names are the server's).

use aurora_leptos::components::*;
use aurora_leptos::frame::Modal;
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;

/// An option: (query name, label, on at first).
pub type PdfOption = (&'static str, &'static str, bool);

pub const CHARACTER: &[PdfOption] = &[
    ("include_compact_sheet", "Character sheet", true),
    ("include_battle_card", "Battle card", true),
    ("include_spell_cards", "Spell cards", false),
    ("include_equipment_cards", "Equipment cards", false),
];

pub const CAMPAIGN: &[PdfOption] = &[
    ("include_campaign_docs", "Campaign documents", true),
    ("include_module_content", "Module content", true),
    ("include_npcs", "NPCs", true),
    ("include_module_map_previews", "Module map previews", false),
    (
        "include_module_tiled_maps",
        "Module maps to play on (tiled)",
        false,
    ),
    ("include_token_cutouts", "Token cutouts", false),
    (
        "include_campaign_map_previews",
        "Campaign map previews",
        false,
    ),
    (
        "include_campaign_tiled_maps",
        "Campaign maps to play on (tiled)",
        false,
    ),
];

pub const MODULE: &[PdfOption] = &[
    ("include_documents", "Documents", true),
    ("include_monsters", "Monsters", true),
    ("include_traps", "Traps", true),
    ("include_pois", "Points of interest", true),
    ("include_npcs", "NPCs", true),
    ("include_play_notes", "Play notes", false),
    ("include_preview", "Map previews", true),
    ("preview_grid", "Previews: grid", true),
    ("preview_los_walls", "Previews: walls", false),
    ("preview_positions", "Previews: positions", false),
    ("include_play", "Maps to play on (tiled)", false),
    ("play_grid", "Play: grid", true),
    ("play_los_walls", "Play: walls", false),
    ("play_cutouts", "Play: token cutouts", false),
];

pub const MAP: &[PdfOption] = &[
    ("include_preview", "Preview (one page)", true),
    ("preview_grid", "Preview: grid", true),
    ("preview_los_walls", "Preview: walls", false),
    ("preview_positions", "Preview: positions", false),
    ("include_play", "To play on (tiled, actual size)", false),
    ("play_grid", "Play: grid", true),
    ("play_los_walls", "Play: walls", false),
    ("play_cutouts", "Play: token cutouts", false),
];

/// The query of the chosen options: `?a=true&b=false`.
pub fn query(chosen: &[(&str, bool)]) -> String {
    let parts: Vec<String> = chosen.iter().map(|(k, on)| format!("{k}={on}")).collect();
    if parts.is_empty() {
        String::new()
    } else {
        format!("?{}", parts.join("&"))
    }
}

/// `path`: the PDF route (`/characters/<id>/pdf`).
#[component]
pub fn PdfDialog(
    open: RwSignal<bool>,
    #[prop(into)] title: String,
    #[prop(into)] path: Signal<String>,
    options: &'static [PdfOption],
    #[prop(into)] file: String,
) -> impl IntoView {
    let picks: Vec<RwSignal<bool>> = options.iter().map(|o| RwSignal::new(o.2)).collect();
    let picks = StoredValue::new(picks);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    let file = StoredValue::new(file);
    let make = move |_| {
        let chosen: Vec<(&str, bool)> = options
            .iter()
            .zip(picks.get_value())
            .map(|(o, s)| (o.0, s.get_untracked()))
            .collect();
        let url = format!("{}{}", path.get_untracked(), query(&chosen));
        busy.set(true);
        error.set(String::new());
        spawn_local(async move {
            let r = api::download(&url, &file.get_value()).await;
            busy.set(false);
            match r {
                Ok(()) => open.set(false),
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    view! {
        <Modal open=open title=title locked=busy>
            <Stack gap="sm">
                {options.iter().zip(picks.get_value()).map(|(o, s)| view! {
                    <label class="mimir-wizard__skill">
                        <input type="checkbox" prop:checked=move || s.get() on:change=move |_| s.update(|v| *v = !*v) />
                        <span>{o.1}</span>
                    </label>
                }).collect_view()}
                {move || busy.get().then(|| view! { <span class="mimir-doc__save" role="status">"Making the PDF…"</span> })}
                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                }}
                <Group>
                    <Button disabled=busy on_click=Callback::new(make)>"Download PDF"</Button>
                    <Button variant="subtle" on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                </Group>
            </Stack>
        </Modal>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries() {
        assert_eq!(query(&[]), "");
        assert_eq!(query(&[("a", true), ("b", false)]), "?a=true&b=false");
    }

    #[test]
    fn option_names_are_unique() {
        for list in [CHARACTER, CAMPAIGN, MODULE, MAP] {
            let mut names: Vec<_> = list.iter().map(|o| o.0).collect();
            let n = names.len();
            names.sort();
            names.dedup();
            assert_eq!(names.len(), n);
        }
    }
}
