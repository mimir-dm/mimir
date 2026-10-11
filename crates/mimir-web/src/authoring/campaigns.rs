//! Campaigns (MIMIR-T-0720): create and edit (name, description);
//! archive, restore, and delete an archived campaign.

use aurora_leptos::components::*;
use aurora_leptos::frame::{ConfirmDialog, Modal};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use mimir_wire::{CampaignPatch, CampaignSummary, NewCampaign};

use crate::admin::pdf::{PdfDialog, CAMPAIGN};
use crate::admin::sources::SourcesDialog;
use crate::api;

/// An empty text is no text.
pub fn optional(text: &str) -> Option<String> {
    let t = text.trim();
    (!t.is_empty()).then(|| t.to_string())
}

/// The campaign form: a new campaign (`existing` none) or an edit.
#[component]
pub fn CampaignDialog(
    open: RwSignal<bool>,
    #[prop(optional)] existing: Option<CampaignSummary>,
    on_saved: Callback<CampaignSummary>,
) -> impl IntoView {
    let id = existing.as_ref().map(|c| c.id.clone());
    let name = RwSignal::new(
        existing
            .as_ref()
            .map(|c| c.name.clone())
            .unwrap_or_default(),
    );
    let description = RwSignal::new(
        existing
            .as_ref()
            .and_then(|c| c.description.clone())
            .unwrap_or_default(),
    );
    let error = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let is_new = id.is_none();
    let id = StoredValue::new(id);

    let save = move |_| {
        busy.set(true);
        let (n, d) = (name.get_untracked(), description.get_untracked());
        spawn_local(async move {
            let r = match id.get_value() {
                None => {
                    api::send::<_, CampaignSummary>(
                        "POST",
                        "/campaigns",
                        &NewCampaign {
                            name: n,
                            description: optional(&d),
                        },
                    )
                    .await
                }
                Some(id) => {
                    api::send::<_, CampaignSummary>(
                        "PATCH",
                        &format!("/campaigns/{id}"),
                        &CampaignPatch {
                            name: Some(n),
                            description: Some(optional(&d)),
                        },
                    )
                    .await
                }
            };
            busy.set(false);
            match r {
                Ok(c) => {
                    open.set(false);
                    error.set(String::new());
                    on_saved.run(c);
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };

    view! {
        <Modal open=open title=if is_new { "New campaign" } else { "Edit the campaign" } locked=busy>
            <Stack gap="sm">
                <TextInput label="Name" value=name />
                <Textarea label="Description" value=description />
                {is_new.then(|| view! {
                    <Text size="sm" dimmed=true>"A new campaign starts with the planning documents (pitch, world primer, safety tools and more)."</Text>
                })}
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

/// The campaign's actions on its dashboard: Edit, and a More menu with
/// Archive, or Restore and Delete when archived.
#[component]
pub fn CampaignActions(campaign: CampaignSummary, on_changed: Callback<()>) -> impl IntoView {
    let editing = RwSignal::new(false);
    let deleting = RwSignal::new(false);
    let sources = RwSignal::new(false);
    let printing = RwSignal::new(false);
    let exporting = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    let archived = campaign.archived_at.is_some();
    let id = StoredValue::new(campaign.id.clone());
    let name = campaign.name.clone();
    let name_for_file = StoredValue::new(campaign.name.clone());
    let navigate = use_navigate();

    let set_archived = move |on: bool| {
        spawn_local(async move {
            let path = format!("/campaigns/{}/archive", id.get_value());
            let r = if on {
                api::send::<_, CampaignSummary>("PUT", &path, &())
                    .await
                    .map(|_| ())
            } else {
                api::delete_json::<CampaignSummary>(&path).await.map(|_| ())
            };
            match r {
                Ok(()) => on_changed.run(()),
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    let remove = move |_| {
        let navigate = navigate.clone();
        spawn_local(async move {
            match api::delete(&format!("/campaigns/{}", id.get_value())).await {
                Ok(()) => navigate("/", Default::default()),
                Err(e) => error.set(api::message(e)),
            }
        });
    };

    view! {
        <Group>
            {archived.then(|| view! { <Pill color=token::GOLD>"Archived"</Pill> })}
            <Button variant="light" on_click=Callback::new(move |_| editing.set(true))>"Edit campaign"</Button>
            <Menu label="More" aria_label="More campaign actions" align="end">
                <MenuItem on_click=Callback::new(move |_| sources.set(true))>"Source books…"</MenuItem>
                <MenuItem on_click=Callback::new(move |_| printing.set(true))>"PDF…"</MenuItem>
                <MenuItem on_click=Callback::new(move |_| {
                    let (cid, cname) = (id.get_value(), name_for_file.get_value());
                    spawn_local(async move {
                        if let Err(e) = api::download(&format!("/campaigns/{cid}/export"), &format!("{cname}.mimir-campaign.tar.gz")).await {
                            error.set(api::message(e));
                        }
                    });
                })>"Export to a file"</MenuItem>
                <MenuItem on_click=Callback::new(move |_| exporting.set(true))>"Export as a 5etools adventure…"</MenuItem>
                {if archived {
                    view! {
                        <MenuItem on_click=Callback::new(move |_| set_archived(false))>"Restore"</MenuItem>
                        <MenuItem danger=true on_click=Callback::new(move |_| deleting.set(true))>"Delete…"</MenuItem>
                    }.into_any()
                } else {
                    view! {
                        <MenuItem on_click=Callback::new(move |_| set_archived(true))>"Archive"</MenuItem>
                    }.into_any()
                }}
            </Menu>
        </Group>
        {move || {
            let e = error.get();
            (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
        }}
        <SourcesDialog
            open=sources
            title=format!("Source books of {name}")
            path=format!("/campaigns/{}/sources", campaign.id)
            hint="The monster searches of this campaign's modules keep to the books chosen here. None chosen: every book."
        />
        <super::adventure_export::AdventureExportDialog
            open=exporting
            campaign_id=campaign.id.clone()
            campaign_name=campaign.name.clone()
        />
        <PdfDialog
            open=printing
            title=format!("PDF of {name}")
            path=format!("/campaigns/{}/pdf", campaign.id)
            options=CAMPAIGN
            file=format!("{name}.pdf")
        />
        <CampaignDialog
            open=editing
            existing=campaign.clone()
            on_saved=Callback::new(move |_| on_changed.run(()))
        />
        <ConfirmDialog
            open=deleting
            title=format!("Delete {name}?")
            message="The campaign, its documents, modules, characters and maps go for good."
            confirm_text=name.clone()
            confirm_label="Delete"
            on_confirm=Callback::new(remove)
        />
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_texts() {
        assert_eq!(optional("  "), None);
        assert_eq!(optional(" Giants "), Some("Giants".into()));
    }
}
