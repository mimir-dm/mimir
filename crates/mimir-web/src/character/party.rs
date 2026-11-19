//! The party (MIMIR-T-0718): the player characters of a campaign and who
//! shares their sheet. A player opens a shared sheet, read-only.

use aurora_leptos::components::*;
use leptos::prelude::*;
use mimir_wire::{PartyMember, ServerMsg};

use super::sheet::{CharacterSheetView, SheetAccess};
use crate::api;
use crate::live::Live;

/// The line under a member's name: race and player.
pub fn member_line(m: &PartyMember) -> String {
    let parts: Vec<&str> = [m.race_name.as_deref(), m.player_name.as_deref()]
        .into_iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() {
        "—".into()
    } else {
        parts.join(" · ")
    }
}

/// Whether a member's sheet opens for this viewer: shared, not their own.
pub fn can_view(m: &PartyMember) -> bool {
    m.shared && !m.me
}

#[component]
pub fn PartyPanel(#[prop(into)] campaign: Signal<String>) -> impl IntoView {
    let version = RwSignal::new(0u32);
    let party = LocalResource::new(move || {
        version.track();
        api::get::<Vec<PartyMember>>(format!("/campaigns/{}/party", campaign.get()))
    });
    // Live: a member shares, stops sharing, or changes.
    let live = StoredValue::new_local(None::<Live>);
    Effect::new(move |_| {
        let c = campaign.get();
        if c.is_empty() {
            return;
        }
        live.set_value(Some(Live::watch(c, move |msg| {
            if let ServerMsg::CharacterChanged { .. } = msg {
                version.update(|v| *v += 1);
            }
        })));
    });

    let viewing = RwSignal::new(None::<String>);
    let members = move || party.get().and_then(Result::ok).unwrap_or_default();

    view! {
        <div class="mimir-party">
            {move || match viewing.get() {
                Some(id) => {
                    let m = members().into_iter().find(|m| m.character_id == id);
                    let back = view! {
                        <Button variant="subtle" on_click=Callback::new(move |_| viewing.set(None))>"Back to the party"</Button>
                    };
                    match m {
                        Some(m) if can_view(&m) => view! {
                            {back}
                            <CharacterSheetView id=Signal::stored(id) access=SheetAccess::ReadOnly />
                        }.into_any(),
                        Some(m) => view! {
                            {back}
                            <Text>{format!("{} does not share their sheet now.", m.name)}</Text>
                        }.into_any(),
                        None => view! { {back} }.into_any(),
                    }
                }
                None => match party.get() {
                    None => view! { <Loading /> }.into_any(),
                    Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                    Some(Ok(list)) => view! {
                        <ul class="mimir-party__list" aria-label="Party">
                            {list.into_iter().map(|m| {
                                let line = member_line(&m);
                                let open = can_view(&m);
                                let state = if m.me {
                                    if m.shared { "You · shared" } else { "You · not shared" }
                                } else if m.shared {
                                    "Shared"
                                } else {
                                    "Not shared"
                                };
                                let (id, label, name) = (m.character_id.clone(), m.name.clone(), m.name.clone());
                                view! {
                                    <li class="mimir-party__member" aria-label=label>
                                        <div>
                                            <Text bold=true>{name}</Text>
                                            <Text size="sm" dimmed=true>{line}</Text>
                                        </div>
                                        <span class="mimir-sheet__num">{state}</span>
                                        {open.then(|| view! {
                                            <Button variant="light" on_click=Callback::new(move |_| viewing.set(Some(id.clone())))>"View"</Button>
                                        })}
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    }.into_any(),
                },
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(shared: bool, me: bool) -> PartyMember {
        PartyMember {
            character_id: "c".into(),
            name: "Robin".into(),
            player_name: Some("Sam".into()),
            race_name: Some("Elf".into()),
            shared,
            me,
        }
    }

    #[test]
    fn only_a_shared_sheet_of_another_opens() {
        assert!(can_view(&member(true, false)));
        assert!(!can_view(&member(false, false)));
        assert!(!can_view(&member(true, true)));
    }

    #[test]
    fn member_lines() {
        assert_eq!(member_line(&member(true, false)), "Elf · Sam");
        let mut m = member(true, false);
        m.player_name = None;
        m.race_name = Some(String::new());
        assert_eq!(member_line(&m), "—");
    }
}
