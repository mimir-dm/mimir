//! `/`: the campaigns. A card opens the campaign's dashboard; New
//! campaign makes one (MIMIR-T-0720); Import campaign reads an archive
//! (MIMIR-T-0731).

use aurora_leptos::components::*;
use aurora_leptos::frame::{Card, Modal, PageHeader};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use mimir_wire::CampaignSummary;

use crate::admin::settings::ImportArchive;
use crate::api;
use crate::authoring::campaigns::CampaignDialog;
use crate::components::loaded;

#[component]
pub fn Home() -> impl IntoView {
    let show_archived = RwSignal::new(false);
    let creating = RwSignal::new(false);
    let importing = RwSignal::new(false);
    let version = RwSignal::new(0u32);
    let navigate = use_navigate();
    let campaigns = LocalResource::new(move || {
        version.track();
        api::campaigns(show_archived.get())
    });
    let current = crate::current::CurrentCampaign::get();
    let list = loaded(campaigns, move |list: Vec<CampaignSummary>| {
        if list.is_empty() {
            return view! { <Empty message="No campaigns yet." /> }.into_any();
        }
        // Continue the campaign the DM last opened (MIMIR-T-0734).
        let last = current
            .map(|c| c.0.get_untracked())
            .and_then(|id| list.iter().find(|c| c.id == id).cloned());
        view! {
            {last.map(|c| view! {
                <div class="mimir-continue">
                    <a class="cl-btn cl-btn--filled" href=format!("/campaigns/{}", c.id)>{format!("Continue {}", c.name)}</a>
                </div>
            })}
            <div class="mimir-cards">
                {list
                    .into_iter()
                    .map(|c| {
                        let archived = c.archived_at.is_some();
                        view! {
                            <Card href=format!("/campaigns/{}", c.id) title=c.name.clone()>
                                <Stack gap="xs">
                                    {archived.then(|| view! { <Pill color=token::GOLD>"Archived"</Pill> })}
                                    <Text size="sm" dimmed=true>
                                        {c.description.clone().unwrap_or_default()}
                                    </Text>
                                </Stack>
                            </Card>
                        }
                    })
                    .collect_view()}
            </div>
        }
            .into_any()
    });
    view! {
        <PageHeader
            title="Campaigns"
            actions=Box::new(move || {
                view! {
                    <Group>
                        <Switch label="Show archived" checked=show_archived />
                        <Button variant="light" on_click=Callback::new(move |_| importing.set(true))>"Import campaign"</Button>
                        <Button on_click=Callback::new(move |_| creating.set(true))>"New campaign"</Button>
                    </Group>
                }
                .into_any()
            })
        />
        {list}
        <Modal open=importing title="Import a campaign" size="lg">
            <ImportArchive on_imported=Callback::new(move |_| version.update(|v| *v += 1)) />
        </Modal>
        <CampaignDialog
            open=creating
            on_saved=Callback::new(move |c: CampaignSummary| {
                navigate(&format!("/campaigns/{}", c.id), Default::default())
            })
        />
    }
}
