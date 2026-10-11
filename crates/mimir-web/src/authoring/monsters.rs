//! A module's monsters (MIMIR-T-0720): add from the catalog, change the
//! count, the name shown and the DM's notes, remove.

use aurora_leptos::components::*;
use aurora_leptos::data::SectionLabel;
use aurora_leptos::frame::{ConfirmDialog, Modal};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_wire::{Choice, ModuleMonsterPatch, ModuleMonsterSummary, NewModuleMonster};

use super::campaigns::optional;
use crate::api;
use crate::components::or_dash;
use crate::reference::detail::{Reference, Target};

#[component]
pub fn MonstersPanel(#[prop(into)] module_id: Signal<String>) -> impl IntoView {
    let version = RwSignal::new(0u32);
    let monsters = LocalResource::new(move || {
        version.track();
        api::get::<Vec<ModuleMonsterSummary>>(format!("/modules/{}/monsters", module_id.get()))
    });
    let reload = Callback::new(move |_| version.update(|v| *v += 1));
    let reference = Reference::get();
    let error = RwSignal::new(String::new());
    let editing = RwSignal::new(None::<ModuleMonsterSummary>);
    let removing = RwSignal::new(None::<ModuleMonsterSummary>);
    let confirm_open = RwSignal::new(false);
    Effect::new(move |_| confirm_open.set(removing.get().is_some()));

    let remove = move |_| {
        let Some(m) = removing.get_untracked() else {
            return;
        };
        spawn_local(async move {
            match api::delete(&format!("/module-monsters/{}", m.id)).await {
                Ok(()) => {
                    removing.set(None);
                    reload.run(());
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };

    view! {
        {move || match monsters.get() {
            None => view! { <Loading /> }.into_any(),
            Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
            Some(Ok(list)) => {
                let n = list.len();
                view! {
                    <SectionLabel label="Monsters" count=n />
                    <Table label="Monsters" min_width="560px">
                        <thead><tr><th>"Monster"</th><th>"Count"</th><th>"Source"</th><th>"Notes"</th><th></th></tr></thead>
                        <tbody>
                            {if list.is_empty() {
                                view! { <TableEmpty message="No monsters in this module." colspan=5 /> }.into_any()
                            } else {
                                list.into_iter().map(|m| {
                                    let source = if m.homebrew_monster_id.is_some() {
                                        "Homebrew".to_string()
                                    } else {
                                        or_dash(&m.monster_source)
                                    };
                                    let (edit, del) = (m.clone(), m.clone());
                                    let name = m.name.clone();
                                    view! {
                                        <tr>
                                            <td>{match (&m.monster_name, &m.monster_source) {
                                                // The inspector (MIMIR-T-0723): the stat block in the drawer.
                                                (Some(n), Some(src)) => {
                                                    let t = Target::new("monsters", src, n);
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class="mimir-linkbtn"
                                                            title="Show the stat block"
                                                            on:click=move |_| if let Some(r) = reference { r.open(t.clone()) }
                                                        >{m.name.clone()}</button>
                                                    }.into_any()
                                                }
                                                _ => view! { <span>{m.name.clone()}</span> }.into_any(),
                                            }}</td>
                                            <td>{m.quantity}</td>
                                            <td>{source}</td>
                                            <td>{or_dash(&m.notes)}</td>
                                            <td class="mimir-row-actions">
                                                <Button variant="subtle" on_click=Callback::new(move |_| editing.set(Some(edit.clone())))>"Edit"</Button>
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
        <AddMonster module_id=module_id on_added=reload />
        {move || editing.get().map(|m| view! { <EditMonster monster=m on_done=Callback::new(move |changed: bool| {
            editing.set(None);
            if changed { reload.run(()); }
        }) /> })}
        <ConfirmDialog
            open=confirm_open
            title=Signal::derive(move || format!("Remove {}?", removing.get().map(|m| m.name).unwrap_or_default()))
            message="Tokens on the maps that use it stay."
            confirm_label="Remove"
            on_confirm=Callback::new(remove)
            on_cancel=Callback::new(move |_| removing.set(None))
        />
    }
}

/// Search the catalog and add a monster (one; the same again adds one).
#[component]
fn AddMonster(#[prop(into)] module_id: Signal<String>, on_added: Callback<()>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let results = RwSignal::new(None::<Result<Vec<Choice>, String>>);
    let added = RwSignal::new(String::new());
    let find = move || {
        let q = search.get_untracked();
        spawn_local(async move {
            let path = format!(
                "/catalog/monsters?search={}",
                js_sys::encode_uri_component(q.trim())
            );
            results.set(Some(
                api::get::<Vec<Choice>>(path).await.map_err(api::message),
            ));
        });
    };
    let add = move |c: Choice| {
        spawn_local(async move {
            let body = NewModuleMonster {
                monster_name: Some(c.name.clone()),
                monster_source: Some(c.source.clone()),
                ..NewModuleMonster::default()
            };
            let path = format!("/modules/{}/monsters", module_id.get_untracked());
            match api::send::<_, ModuleMonsterSummary>("POST", &path, &body).await {
                Ok(m) => {
                    added.set(format!("{} added ({} in all).", c.name, m.quantity));
                    on_added.run(());
                }
                Err(e) => added.set(api::message(e)),
            }
        });
    };
    view! {
        <SectionLabel label="Add a monster" />
        <form class="mimir-inv__search" on:submit=move |ev| { ev.prevent_default(); find(); }>
            <TextInput label="Monster name" value=search />
            <Button variant="light" button_type="submit">"Search"</Button>
        </form>
        {move || {
            let a = added.get();
            (!a.is_empty()).then(|| view! { <Text size="sm" dimmed=true>{a}</Text> })
        }}
        {move || results.get().map(|r| match r {
            Err(e) => view! { <Alert color=token::BAD>{e}</Alert> }.into_any(),
            Ok(list) if list.is_empty() => view! { <Text size="sm" dimmed=true>"No monster by that name."</Text> }.into_any(),
            Ok(list) => view! {
                <ul class="mimir-inv__results" aria-label="Monsters found">
                    {list.into_iter().map(|c| {
                        let pick = c.clone();
                        view! {
                            <li>
                                <div class="mimir-inv__name">
                                    <Text bold=true>{c.name.clone()}</Text>
                                    <Text size="xs" dimmed=true>{format!("{}{}", c.source, c.detail.map(|d| format!(" · {d}")).unwrap_or_default())}</Text>
                                </div>
                                <Button variant="light" on_click=Callback::new(move |_| add(pick.clone()))>"Add"</Button>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            }.into_any(),
        })}
    }
}

#[component]
fn EditMonster(monster: ModuleMonsterSummary, on_done: Callback<bool>) -> impl IntoView {
    let open = RwSignal::new(true);
    let quantity = RwSignal::new(f64::from(monster.quantity));
    let shown = RwSignal::new(
        if monster.monster_name.as_deref() == Some(monster.name.as_str()) {
            String::new()
        } else {
            monster.name.clone()
        },
    );
    let notes = RwSignal::new(monster.notes.clone().unwrap_or_default());
    let error = RwSignal::new(String::new());
    let id = StoredValue::new(monster.id.clone());
    Effect::new(move |prev: Option<bool>| {
        let now = open.get();
        if prev == Some(true) && !now {
            on_done.run(false);
        }
        now
    });
    let save = move |_| {
        let body = ModuleMonsterPatch {
            quantity: Some(quantity.get_untracked().round() as i32),
            display_name: Some(optional(&shown.get_untracked()).unwrap_or_default()),
            notes: Some(notes.get_untracked().trim().to_string()),
        };
        spawn_local(async move {
            match api::send::<_, ModuleMonsterSummary>(
                "PATCH",
                &format!("/module-monsters/{}", id.get_value()),
                &body,
            )
            .await
            {
                Ok(_) => on_done.run(true),
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    view! {
        <Modal open=open title=format!("Edit {}", monster.name)>
            <Stack gap="sm">
                <NumberInput label="Count" value=quantity min=1.0 />
                <TextInput label="Name shown (empty: the monster's name)" value=shown />
                <Textarea label="DM notes" value=notes />
                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                }}
                <Group>
                    <Button on_click=Callback::new(save)>"Save"</Button>
                    <Button variant="subtle" on_click=Callback::new(move |_| on_done.run(false))>"Cancel"</Button>
                </Group>
            </Stack>
        </Modal>
    }
}
