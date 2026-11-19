//! `/modules/:id`: a module: its documents, NPCs, maps and dangers
//! (monsters, and the traps and points of interest of its maps). A click
//! on the title renames it in place (MIMIR-T-0732).

use aurora_leptos::components::*;
use aurora_leptos::data::SectionLabel;
use aurora_leptos::frame::{Modal, PageHeader, TabItem, TabPanel, Tabs};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_params_map, use_query_map};
use mimir_wire::{ModuleNpcSummary, PlayerNote};

use crate::api;
use crate::authoring::documents::DocumentWorkspace;
use crate::authoring::maps::MapsPanel;
use crate::authoring::markers::MarkersPanel;
use crate::authoring::modules::ModuleActions;
use crate::authoring::monsters::MonstersPanel;
use crate::character::notes::NoteCard;
use crate::components::{loaded, or_dash, ListResource};

/// The sections of the module page: (value, label).
const SECTIONS: [(&str, &str); 5] = [
    ("documents", "Documents"),
    ("npcs", "NPCs"),
    ("maps", "Maps"),
    ("dangers", "Dangers"),
    ("notes", "Player notes"),
];

/// The section of a `?section=` value; documents when absent or unknown.
pub fn section_of(value: Option<&str>) -> &'static str {
    SECTIONS
        .iter()
        .map(|(v, _)| *v)
        .find(|v| Some(*v) == value)
        .unwrap_or("documents")
}

#[component]
pub fn ModulePage() -> impl IntoView {
    let params = use_params_map();
    let id = Memo::new(move |_| params.get().get("id").unwrap_or_default());
    let version = RwSignal::new(0u32);
    let module = LocalResource::new(move || {
        version.track();
        api::module(id.get())
    });
    let campaign_id = Signal::derive(move || match module.get() {
        Some(Ok(m)) => m.campaign_id,
        _ => String::new(),
    });
    let npcs: ListResource<ModuleNpcSummary> =
        LocalResource::new(move || api::module_list(id.get(), "npcs"));
    let notes: ListResource<PlayerNote> =
        LocalResource::new(move || api::module_list(id.get(), "notes"));

    // Inline rename: Enter or leaving the field saves; Esc cancels.
    let renaming = RwSignal::new(false);
    let draft = RwSignal::new(String::new());
    let rename_error = RwSignal::new(String::new());
    let start_rename = Callback::new(move |_| {
        if let Some(Ok(m)) = module.get_untracked() {
            draft.set(m.name);
            rename_error.set(String::new());
            renaming.set(true);
        }
    });
    let save_name = move || {
        if !renaming.get_untracked() {
            return;
        }
        renaming.set(false);
        let name = draft.get_untracked().trim().to_string();
        let Some(Ok(m)) = module.get_untracked() else {
            return;
        };
        if name.is_empty() || name == m.name {
            return;
        }
        spawn_local(async move {
            let r = api::send::<_, serde_json::Value>(
                "PATCH",
                &format!("/modules/{}", m.id),
                &serde_json::json!({ "name": name }),
            )
            .await;
            match r {
                Ok(_) => version.update(|v| *v += 1),
                Err(e) => rename_error.set(api::message(e)),
            }
        });
    };
    let title_input = NodeRef::<leptos::html::Input>::new();
    Effect::new(move |_| {
        if renaming.get() {
            if let Some(el) = title_input.get() {
                let _ = el.focus();
                el.select();
            }
        }
    });
    let header = loaded(module, move |m| {
        let actions = m.clone();
        let back = format!("/campaigns/{}/modules", m.campaign_id);
        let sub = format!("Module {}", m.module_number);
        let name = m.name.clone();
        view! {
            {move || if renaming.get() {
                view! {
                    <div class="cl-page-header">
                        <div class="cl-page-header__main">
                            <a class="cl-page-header__back" href=back.clone()><span aria-hidden="true">"← "</span>"Modules"</a>
                            <input
                                class="cl-page-header__title mimir-title-input"
                                aria-label="Module name"
                                node_ref=title_input
                                prop:value=move || draft.get()
                                on:input=move |ev| draft.set(event_target_value(&ev))
                                on:keydown=move |ev: leptos::ev::KeyboardEvent| match ev.key().as_str() {
                                    "Enter" => save_name(),
                                    "Escape" => renaming.set(false),
                                    _ => {}
                                }
                                on:blur=move |_| save_name()
                            />
                            <div class="cl-page-header__sub">{sub.clone()}</div>
                        </div>
                    </div>
                }.into_any()
            } else {
                let actions = actions.clone();
                view! {
                    <div
                        class="mimir-title-edit"
                        title="Click the name to rename the module"
                        on:click=move |ev: leptos::ev::MouseEvent| {
                            let on_title = ev
                                .target()
                                .and_then(|t| wasm_bindgen::JsCast::dyn_into::<web_sys::Element>(t).ok())
                                .is_some_and(|el| el.class_list().contains("cl-page-header__title"));
                            if on_title {
                                start_rename.run(());
                            }
                        }
                    >
                        <PageHeader
                            title=name.clone()
                            sub=sub.clone()
                            back_href=back.clone()
                            back_label="Modules"
                            actions=Box::new(move || view! {
                                <ModuleActions
                                    module=actions.clone()
                                    on_changed=Callback::new(move |_| version.update(|v| *v += 1))
                                    on_rename=start_rename
                                />
                            }.into_any())
                        />
                    </div>
                }.into_any()
            }}
            {move || {
                let e = rename_error.get();
                (!e.is_empty()).then(|| view! { <Alert color=aurora_leptos::tokens::token::BAD>{e}</Alert> })
            }}
        }
    });

    // One section at a time: a long document would push the others far
    // down on a tablet. The section is in the address (`?section=`), so a
    // link or the back button opens it.
    let query = use_query_map();
    let current = move || section_of(query.get().get("section").as_deref()).to_string();
    let section = RwSignal::new(current());
    Effect::new(move |_| section.set(current()));
    let tabs = SECTIONS
        .iter()
        .map(|(value, label)| TabItem::new(*value, *label).href(format!("?section={value}")))
        .collect::<Vec<_>>();
    view! {
        {header}
        <Tabs tabs=tabs value=section label="Module sections">
            <TabPanel value="documents">
                <DocumentWorkspace base=Signal::derive(move || format!("/modules/{}", id.get())) />
            </TabPanel>
            <TabPanel value="npcs">{loaded(npcs, |list| view! { <NpcList list=list /> })}</TabPanel>
            <TabPanel value="maps">
                {move || (!campaign_id.get().is_empty()).then(|| view! {
                    <MapsPanel campaign_id=campaign_id module_id=Signal::derive(move || id.get()) />
                })}
            </TabPanel>
            <TabPanel value="dangers">
                <MonstersPanel module_id=id campaign_id=campaign_id />
                <MarkersPanel module_id=id />
            </TabPanel>
            <TabPanel value="notes">{loaded(notes, notes_view)}</TabPanel>
        </Tabs>
    }
}

/// The players' notes about the module (MIMIR-T-0718).
fn notes_view(list: Vec<PlayerNote>) -> impl IntoView {
    if list.is_empty() {
        return view! { <Text size="sm" dimmed=true>"No player notes about this module yet."</Text> }
            .into_any();
    }
    view! {
        <SectionLabel label="Player notes" count=list.len() />
        <ul class="mimir-notes__list" aria-label="Player notes">
            {list.into_iter().map(|n| view! { <NoteCard note=n by=true /> }).collect_view()}
        </ul>
    }
    .into_any()
}

/// The module's NPCs; a name opens the NPC's details.
#[component]
fn NpcList(list: Vec<ModuleNpcSummary>) -> impl IntoView {
    let n = list.len();
    let showing = RwSignal::new(None::<ModuleNpcSummary>);
    let open = RwSignal::new(false);
    Effect::new(move |_| open.set(showing.with(Option::is_some)));
    Effect::new(move |prev: Option<bool>| {
        let now = open.get();
        if prev == Some(true) && !now {
            showing.set(None);
        }
        now
    });
    let title = Signal::derive(move || showing.get().map(|n| n.name).unwrap_or_default());
    view! {
        <SectionLabel label="NPCs" count=n />
        <Table label="NPCs" min_width="480px">
            <thead><tr><th>"Name"</th><th>"Role"</th><th>"Description"</th></tr></thead>
            <tbody>
                {if list.is_empty() {
                    view! { <TableEmpty message="No NPCs in this module." colspan=3 /> }.into_any()
                } else {
                    list.into_iter().map(|n| {
                        let pick = n.clone();
                        view! {
                            <tr>
                                <td><button type="button" class="mimir-link-button" on:click=move |_| showing.set(Some(pick.clone()))>{n.name.clone()}</button></td>
                                <td>{or_dash(&n.role)}</td>
                                <td>{or_dash(&n.description)}</td>
                            </tr>
                        }
                    }).collect_view().into_any()
                }}
            </tbody>
        </Table>
        <Modal open=open title=title>
            <Stack gap="sm">
                <dl class="mimir-details">
                    {move || showing.get().map(|n| npc_details(&n).into_iter().map(|(k, v)| view! { <dt>{k}</dt><dd>{v}</dd> }).collect_view())}
                </dl>
                <Group>
                    <Button variant="subtle" on_click=Callback::new(move |_| open.set(false))>"Close"</Button>
                </Group>
            </Stack>
        </Modal>
    }
}

/// The details of an NPC, as label and text ("—" when empty).
pub fn npc_details(n: &ModuleNpcSummary) -> Vec<(&'static str, String)> {
    vec![
        ("Role", or_dash(&n.role)),
        ("Description", or_dash(&n.description)),
        ("Appearance", or_dash(&n.appearance)),
        ("Personality", or_dash(&n.personality)),
        ("Motivation", or_dash(&n.motivation)),
        ("Secrets", or_dash(&n.secrets)),
    ]
}

#[cfg(test)]
mod tests {
    use super::{npc_details, section_of};
    use mimir_wire::ModuleNpcSummary;

    #[test]
    fn npc_details_show_each_field() {
        let n = ModuleNpcSummary {
            id: "n".into(),
            module_id: "m".into(),
            name: "Sildar".into(),
            role: Some("Ally".into()),
            description: None,
            appearance: Some("Grey beard".into()),
            personality: None,
            motivation: None,
            secrets: Some("Lords' Alliance".into()),
        };
        let d = npc_details(&n);
        assert_eq!(d.len(), 6);
        assert_eq!(d[0], ("Role", "Ally".to_string()));
        assert_eq!(d[1].1, "—");
        assert_eq!(d[5], ("Secrets", "Lords' Alliance".to_string()));
    }

    #[test]
    fn the_section_comes_from_the_address() {
        assert_eq!(section_of(None), "documents");
        assert_eq!(section_of(Some("maps")), "maps");
        assert_eq!(section_of(Some("dangers")), "dangers");
        assert_eq!(section_of(Some("notes")), "notes");
        assert_eq!(section_of(Some("nonsense")), "documents");
    }
}
