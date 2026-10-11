//! `/`: the campaigns. A card opens the campaign's dashboard.

use aurora_leptos::components::*;
use aurora_leptos::frame::{Card, PageHeader};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use mimir_wire::CampaignSummary;

use crate::api;
use crate::components::loaded;

#[component]
pub fn Home() -> impl IntoView {
    let show_archived = RwSignal::new(false);
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
                view! { <Switch label="Show archived" checked=show_archived /> }.into_any()
            })
        />
        {list}
    }
}
