//! A module's NPCs (MIMIR-T-0750): their part in the module. Add one by
//! hand or from a campaign NPC, change it, remove it; a name opens the
//! details.

use aurora_leptos::components::*;
use aurora_leptos::data::SectionLabel;
use aurora_leptos::frame::{ConfirmDialog, Modal};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_wire::{CharacterSummary, ModuleNpcPatch, ModuleNpcSummary, NewModuleNpc};

use crate::api;
use crate::components::or_dash;

#[component]
pub fn NpcsPanel(
    #[prop(into)] module_id: Signal<String>,
    #[prop(into)] campaign_id: Signal<String>,
) -> impl IntoView {
    let version = RwSignal::new(0u32);
    let npcs = LocalResource::new(move || {
        version.track();
        api::get::<Vec<ModuleNpcSummary>>(format!("/modules/{}/npcs", module_id.get()))
    });
    let reload = Callback::new(move |_| version.update(|v| *v += 1));
    let error = RwSignal::new(String::new());
    // The form: `Some(None)` adds, `Some(Some(npc))` edits.
    let form = RwSignal::new(None::<Option<ModuleNpcSummary>>);
    let showing = RwSignal::new(None::<ModuleNpcSummary>);
    let show_open = RwSignal::new(false);
    Effect::new(move |_| show_open.set(showing.with(Option::is_some)));
    Effect::new(move |prev: Option<bool>| {
        let now = show_open.get();
        if prev == Some(true) && !now {
            showing.set(None);
        }
        now
    });
    let removing = RwSignal::new(None::<ModuleNpcSummary>);
    let confirm_open = RwSignal::new(false);
    Effect::new(move |_| confirm_open.set(removing.get().is_some()));

    let remove = move |_| {
        let Some(n) = removing.get_untracked() else {
            return;
        };
        spawn_local(async move {
            match api::delete(&format!("/module-npcs/{}", n.id)).await {
                Ok(()) => {
                    removing.set(None);
                    reload.run(());
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    let title = Signal::derive(move || showing.get().map(|n| n.name).unwrap_or_default());

    view! {
        {move || match npcs.get() {
            None => view! { <Loading /> }.into_any(),
            Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
            Some(Ok(list)) => {
                let n = list.len();
                view! {
                    <SectionLabel label="NPCs" count=n />
                    <Table label="NPCs" min_width="560px">
                        <thead><tr><th>"Name"</th><th>"Role"</th><th>"Description"</th><th></th></tr></thead>
                        <tbody>
                            {if list.is_empty() {
                                view! { <TableEmpty message="No NPCs in this module." colspan=4 /> }.into_any()
                            } else {
                                list.into_iter().map(|n| {
                                    let (show, edit, del) = (n.clone(), n.clone(), n.clone());
                                    let name = n.name.clone();
                                    view! {
                                        <tr>
                                            <td><button type="button" class="mimir-link-button" on:click=move |_| showing.set(Some(show.clone()))>{n.name.clone()}</button></td>
                                            <td>{or_dash(&n.role)}</td>
                                            <td>{or_dash(&n.description)}</td>
                                            <td class="mimir-row-actions">
                                                <Button variant="subtle" on_click=Callback::new(move |_| form.set(Some(Some(edit.clone()))))>"Edit"</Button>
                                                <ActionIcon title=format!("Remove {name}") on_click=Callback::new(move |_| removing.set(Some(del.clone())))>"×"</ActionIcon>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view().into_any()
                            }}
                        </tbody>
                    </Table>
                }.into_any()
            }
        }}
        {move || {
            let e = error.get();
            (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
        }}
        <Group>
            <Button variant="light" on_click=Callback::new(move |_| form.set(Some(None)))>"Add an NPC"</Button>
        </Group>
        {move || form.get().map(|existing| view! {
            <NpcForm
                module_id=module_id
                campaign_id=campaign_id
                existing=existing
                on_done=Callback::new(move |changed: bool| {
                    form.set(None);
                    if changed { reload.run(()); }
                })
            />
        })}
        <Modal open=show_open title=title>
            <Stack gap="sm">
                <dl class="mimir-details">
                    {move || showing.get().map(|n| npc_details(&n).into_iter().map(|(k, v)| view! { <dt>{k}</dt><dd>{v}</dd> }).collect_view())}
                </dl>
                <Group>
                    <Button variant="subtle" on_click=Callback::new(move |_| show_open.set(false))>"Close"</Button>
                </Group>
            </Stack>
        </Modal>
        <ConfirmDialog
            open=confirm_open
            title=Signal::derive(move || format!("Remove {}?", removing.get().map(|n| n.name).unwrap_or_default()))
            message="The campaign NPC, if any, stays. Tokens on the maps that use this NPC stay."
            confirm_label="Remove"
            on_confirm=Callback::new(remove)
            on_cancel=Callback::new(move |_| removing.set(None))
        />
    }
}

/// The add or edit form. Adding can start from a campaign NPC: the server
/// fills the name, role and personality that the form leaves empty.
#[component]
fn NpcForm(
    #[prop(into)] module_id: Signal<String>,
    #[prop(into)] campaign_id: Signal<String>,
    existing: Option<ModuleNpcSummary>,
    on_done: Callback<bool>,
) -> impl IntoView {
    let open = RwSignal::new(true);
    let value = |f: &Option<String>| RwSignal::new(f.clone().unwrap_or_default());
    let e = existing.clone().unwrap_or_else(|| ModuleNpcSummary {
        id: String::new(),
        module_id: String::new(),
        name: String::new(),
        role: None,
        description: None,
        appearance: None,
        personality: None,
        motivation: None,
        secrets: None,
    });
    let name = RwSignal::new(e.name.clone());
    let role = value(&e.role);
    let description = value(&e.description);
    let appearance = value(&e.appearance);
    let personality = value(&e.personality);
    let motivation = value(&e.motivation);
    let secrets = value(&e.secrets);
    let from = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());
    let editing = StoredValue::new(existing.as_ref().map(|n| n.id.clone()));
    let campaign_npcs = LocalResource::new(move || {
        api::get::<Vec<CharacterSummary>>(format!("/campaigns/{}/npcs", campaign_id.get()))
    });
    Effect::new(move |prev: Option<bool>| {
        let now = open.get();
        if prev == Some(true) && !now {
            on_done.run(false);
        }
        now
    });
    let save = move |_| {
        let text = |s: RwSignal<String>| s.get_untracked().trim().to_string();
        let result_of = move |r: Result<ModuleNpcSummary, aurora_leptos::tokens::ApiError>| match r
        {
            Ok(_) => on_done.run(true),
            Err(e) => error.set(api::message(e)),
        };
        match editing.get_value() {
            Some(id) => {
                let body = ModuleNpcPatch {
                    name: Some(text(name)),
                    role: Some(text(role)),
                    description: Some(text(description)),
                    appearance: Some(text(appearance)),
                    personality: Some(text(personality)),
                    motivation: Some(text(motivation)),
                    secrets: Some(text(secrets)),
                };
                spawn_local(async move {
                    result_of(api::send("PATCH", &format!("/module-npcs/{id}"), &body).await);
                });
            }
            None => {
                let given = |s: RwSignal<String>| Some(text(s)).filter(|t| !t.is_empty());
                let body = NewModuleNpc {
                    character_id: given(from),
                    name: given(name),
                    role: given(role),
                    description: given(description),
                    appearance: given(appearance),
                    personality: given(personality),
                    motivation: given(motivation),
                    secrets: given(secrets),
                };
                let path = format!("/modules/{}/npcs", module_id.get_untracked());
                spawn_local(async move {
                    result_of(api::send("POST", &path, &body).await);
                });
            }
        }
    };
    let title = match &existing {
        Some(n) => format!("Edit {}", n.name),
        None => "Add an NPC".to_string(),
    };
    let adding = existing.is_none();
    view! {
        <Modal open=open title=title>
            <Stack gap="sm">
                {adding.then(|| view! {
                    {move || {
                        let opts: Vec<(String, String)> = campaign_npcs
                            .get()
                            .and_then(Result::ok)
                            .unwrap_or_default()
                            .into_iter()
                            .map(|c| (c.id, c.name))
                            .collect();
                        (!opts.is_empty()).then(|| view! {
                            <Select
                                label="From a campaign NPC (optional)"
                                placeholder="None: a new NPC"
                                option_pairs=opts
                                value=from
                            />
                        })
                    }}
                    <Text size="xs" dimmed=true>"With a campaign NPC, an empty name, role or personality comes from the character."</Text>
                })}
                <TextInput label="Name" value=name />
                <TextInput label="Role in this module" value=role />
                <Textarea label="Description" value=description />
                <Textarea label="Appearance" value=appearance />
                <Textarea label="Personality" value=personality />
                <Textarea label="Motivation" value=motivation />
                <Textarea label="Secrets (DM only)" value=secrets />
                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                }}
                <Group>
                    <Button on_click=Callback::new(save)>{if adding { "Add" } else { "Save" }}</Button>
                    <Button variant="subtle" on_click=Callback::new(move |_| on_done.run(false))>"Cancel"</Button>
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
