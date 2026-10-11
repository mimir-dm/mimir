//! `/`: the campaigns. A card opens the campaign's dashboard; New
//! campaign makes one (MIMIR-T-0720).

use aurora_leptos::components::*;
use aurora_leptos::frame::{Card, PageHeader};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use mimir_wire::CampaignSummary;

use crate::api;
use crate::authoring::campaigns::CampaignDialog;
use crate::components::loaded;

#[component]
pub fn Home() -> impl IntoView {
    let show_archived = RwSignal::new(false);
    let creating = RwSignal::new(false);
    let navigate = use_navigate();
    let campaigns = LocalResource::new(move || api::campaigns(show_archived.get()));
    let list = loaded(campaigns, |list: Vec<CampaignSummary>| {
        if list.is_empty() {
            return view! { <Empty message="No campaigns yet." /> }.into_any();
        }
        view! {
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
                        <Button on_click=Callback::new(move |_| creating.set(true))>"New campaign"</Button>
                    </Group>
                }
                .into_any()
            })
        />
        {list}
        <CampaignDialog
            open=creating
            on_saved=Callback::new(move |c: CampaignSummary| {
                navigate(&format!("/campaigns/{}", c.id), Default::default())
            })
        />
    }
}
