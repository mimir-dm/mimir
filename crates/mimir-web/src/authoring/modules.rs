//! Modules (MIMIR-T-0720): create (name, type, description), edit, move
//! up and down, delete.

use aurora_leptos::components::*;
use aurora_leptos::frame::{ConfirmDialog, Modal};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use mimir_wire::{ModulePatch, ModuleSummary, NewModule, MODULE_TYPES};

use super::campaigns::optional;
use crate::admin::pdf::{PdfDialog, MODULE};
use crate::api;

/// The module form: a new module of `campaign_id`, or an edit of
/// `existing` (name and description; the type only picks the starting
/// documents).
#[component]
pub fn ModuleDialog(
    open: RwSignal<bool>,
    #[prop(optional, into)] campaign_id: String,
    #[prop(optional)] existing: Option<ModuleSummary>,
    on_saved: Callback<ModuleSummary>,
) -> impl IntoView {
    let is_new = existing.is_none();
    let id = StoredValue::new(existing.as_ref().map(|m| m.id.clone()));
    let campaign_id = StoredValue::new(campaign_id);
    let name = RwSignal::new(
        existing
            .as_ref()
            .map(|m| m.name.clone())
            .unwrap_or_default(),
    );
    let description = RwSignal::new(
        existing
            .as_ref()
            .and_then(|m| m.description.clone())
            .unwrap_or_default(),
    );
    let kind = RwSignal::new("general".to_string());
    let error = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let save = move |_| {
        busy.set(true);
        let (n, d, k) = (
            name.get_untracked(),
            description.get_untracked(),
            kind.get_untracked(),
        );
        spawn_local(async move {
            let r = match id.get_value() {
                None => {
                    let body = NewModule {
                        name: n,
                        description: optional(&d),
                        module_type: Some(k),
                    };
                    let path = format!("/campaigns/{}/modules", campaign_id.get_value());
                    api::send::<_, ModuleSummary>("POST", &path, &body).await
                }
                Some(id) => {
                    let body = ModulePatch {
                        name: Some(n),
                        description: Some(optional(&d)),
                    };
                    api::send::<_, ModuleSummary>("PATCH", &format!("/modules/{id}"), &body).await
                }
            };
            busy.set(false);
            match r {
                Ok(m) => {
                    open.set(false);
                    error.set(String::new());
                    on_saved.run(m);
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    let types = StoredValue::new(
        MODULE_TYPES
            .iter()
            .map(|(k, n)| (k.to_string(), n.to_string()))
            .collect::<Vec<_>>(),
    );

    view! {
        <Modal open=open title=if is_new { "New module" } else { "Edit the module" } locked=busy>
            <Stack gap="sm">
                <TextInput label="Name" value=name />
                {is_new.then(|| view! {
                    <Select label="Type" option_pairs=types.get_value() value=kind />
                    <Text size="sm" dimmed=true>"The type picks the starting documents."</Text>
                })}
                <Textarea label="Description" value=description />
                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                }}
                <Group>
                    <Button
                        disabled=Signal::derive(move || name.get().trim().is_empty() || busy.get())
                        on_click=Callback::new(save)
                    >{if is_new { "Create" } else { "Save" }}</Button>
                    <Button variant="subtle" on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                </Group>
            </Stack>
        </Modal>
    }
}

/// The module's actions on its page: Play (its first map), Edit, and
/// More → Delete.
#[component]
pub fn ModuleActions(module: ModuleSummary, on_changed: Callback<()>) -> impl IntoView {
    let editing = RwSignal::new(false);
    let deleting = RwSignal::new(false);
    let printing = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    let id = StoredValue::new(module.id.clone());
    let file_name = StoredValue::new(module.name.clone());
    let campaign = StoredValue::new(module.campaign_id.clone());
    let name = module.name.clone();
    // Play opens the DM map of the module's first map (MIMIR-T-0729).
    let first_map = LocalResource::new(move || async move {
        api::module_list::<mimir_wire::MapSummary>(id.get_value(), "maps")
            .await
            .ok()
            .and_then(|mut l| {
                l.sort_by_key(|m| m.sort_order);
                l.into_iter().next()
            })
    });
    let navigate = use_navigate();
    let remove = move |_| {
        let navigate = navigate.clone();
        spawn_local(async move {
            match api::delete(&format!("/modules/{}", id.get_value())).await {
                Ok(()) => navigate(
                    &format!("/campaigns/{}/modules", campaign.get_value()),
                    Default::default(),
                ),
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    view! {
        <Group>
            {move || first_map.get().flatten().map(|m| view! {
                <a class="cl-btn cl-btn--filled" href=format!("/maps/{}", m.id)>"Play"</a>
            })}
            <Button variant="light" on_click=Callback::new(move |_| editing.set(true))>"Edit module"</Button>
            <Menu label="More" aria_label="More module actions" align="end">
                <MenuItem on_click=Callback::new(move |_| printing.set(true))>"PDF…"</MenuItem>
                <MenuItem on_click=Callback::new(move |_| {
                    let (mid, n) = (id.get_value(), file_name.get_value());
                    spawn_local(async move {
                        if let Err(e) = api::download(&format!("/modules/{mid}/monster-cards?show_cut_lines=true"), &format!("{n} monsters.pdf")).await {
                            error.set(api::message(e));
                        }
                    });
                })>"Monster cards (PDF)"</MenuItem>
                <MenuItem danger=true on_click=Callback::new(move |_| deleting.set(true))>"Delete module…"</MenuItem>
            </Menu>
        </Group>
        {move || {
            let e = error.get();
            (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
        }}
        <PdfDialog
            open=printing
            title=format!("PDF of {name}")
            path=format!("/modules/{}/pdf", module.id)
            options=MODULE
            file=format!("{name}.pdf")
        />
        <ModuleDialog open=editing existing=module.clone() on_saved=Callback::new(move |_| on_changed.run(())) />
        <ConfirmDialog
            open=deleting
            title=format!("Delete {name}?")
            message="The module, its documents, monsters and maps go for good."
            confirm_label="Delete"
            on_confirm=Callback::new(remove)
        />
    }
}
