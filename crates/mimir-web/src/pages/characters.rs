//! `/characters` (MIMIR-T-0721): every character, by campaign, and those
//! in no campaign. Create one with the wizard, move one to another
//! campaign (or out of any), delete one.

use aurora_leptos::components::*;
use aurora_leptos::data::SectionLabel;
use aurora_leptos::frame::{ConfirmDialog, PageHeader};
use aurora_leptos::tokens::{token, ApiError};
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_wire::{CampaignAssignment, CampaignSummary, CharacterSheet, CharacterSummary};

use crate::api;
use crate::authoring::wizard::CharacterWizard;
use crate::components::{class_label, or_dash};

/// A group of the page: a campaign (none: no campaign) and its characters.
type Group = (Option<CampaignSummary>, Vec<CharacterSummary>);

/// PCs first, then by name.
pub fn sort_characters(list: &mut [CharacterSummary]) {
    list.sort_by(|a, b| a.is_npc.cmp(&b.is_npc).then(a.name.cmp(&b.name)));
}

async fn load() -> Result<(Vec<CampaignSummary>, Vec<Group>), ApiError> {
    let campaigns = api::campaigns(false).await?;
    let mut groups = Vec::new();
    let mut none: Vec<CharacterSummary> = api::get("/characters/unassigned").await?;
    sort_characters(&mut none);
    groups.push((None, none));
    for c in &campaigns {
        let mut list: Vec<CharacterSummary> = api::campaign_list(c.id.clone(), "pcs").await?;
        list.extend(api::campaign_list::<CharacterSummary>(c.id.clone(), "npcs").await?);
        sort_characters(&mut list);
        groups.push((Some(c.clone()), list));
    }
    Ok((campaigns, groups))
}

#[component]
pub fn CharactersPage() -> impl IntoView {
    let version = RwSignal::new(0u32);
    let data = LocalResource::new(move || {
        version.track();
        load()
    });
    let reload = move || version.update(|v| *v += 1);
    let creating = RwSignal::new(false);
    let removing = RwSignal::new(None::<CharacterSummary>);
    let confirm_open = RwSignal::new(false);
    Effect::new(move |_| confirm_open.set(removing.get().is_some()));
    let error = RwSignal::new(String::new());

    let move_to = move |id: String, campaign: String| {
        spawn_local(async move {
            let body = CampaignAssignment {
                campaign_id: (!campaign.is_empty()).then_some(campaign),
            };
            match api::send::<_, CharacterSheet>(
                "PUT",
                &format!("/characters/{id}/campaign"),
                &body,
            )
            .await
            {
                Ok(_) => reload(),
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    let remove = move |_| {
        let Some(c) = removing.get_untracked() else {
            return;
        };
        spawn_local(async move {
            match api::delete(&format!("/characters/{}", c.id)).await {
                Ok(()) => {
                    removing.set(None);
                    reload();
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };

    let group_view = move |(campaign, list): Group, campaigns: Vec<CampaignSummary>| {
        let (title, here) = match &campaign {
            Some(c) => (c.name.clone(), c.id.clone()),
            None => ("No campaign".to_string(), String::new()),
        };
        let n = list.len();
        let (aria, label) = (title.clone(), title.clone());
        view! {
            <section class="mimir-chars__group" aria-label=aria>
                <SectionLabel label=label count=n />
                <Table label=title min_width="720px">
                    <thead><tr><th>"Name"</th><th>"Kind"</th><th>"Details"</th><th>"Player"</th><th>"Campaign"</th><th></th></tr></thead>
                    <tbody>
                        {if list.is_empty() {
                            view! { <TableEmpty message="No characters." colspan=6 /> }.into_any()
                        } else {
                            list.into_iter().map(|c| {
                                let details = [class_label(&c.classes), or_dash(&c.race_name)]
                                    .into_iter()
                                    .filter(|t| t != "—")
                                    .collect::<Vec<_>>()
                                    .join(" · ");
                                let (id, del) = (c.id.clone(), c.clone());
                                let name = c.name.clone();
                                let here = here.clone();
                                view! {
                                    <tr>
                                        <td><a href=format!("/characters/{}", c.id)>{c.name.clone()}</a></td>
                                        <td>{if c.is_npc { "NPC" } else { "PC" }}</td>
                                        <td>{if details.is_empty() { "—".to_string() } else { details }}</td>
                                        <td>{or_dash(&c.player_name)}</td>
                                        <td>
                                            <select
                                                class="mimir-wizard__select"
                                                aria-label=format!("Campaign of {name}")
                                                on:change=move |ev| move_to(id.clone(), event_target_value(&ev))
                                            >
                                                <option value="" selected=here.is_empty()>"No campaign"</option>
                                                {campaigns.iter().map(|k| view! {
                                                    <option value=k.id.clone() selected=k.id == here>{k.name.clone()}</option>
                                                }).collect_view()}
                                            </select>
                                        </td>
                                        <td class="mimir-row-actions">
                                            <ActionIcon title=format!("Delete {name}") on_click=Callback::new(move |_| removing.set(Some(del.clone())))>"×"</ActionIcon>
                                        </td>
                                    </tr>
                                }
                            }).collect_view().into_any()
                        }}
                    </tbody>
                </Table>
            </section>
        }
    };

    view! {
        <PageHeader
            title="Characters"
            actions=Box::new(move || view! {
                <Button on_click=Callback::new(move |_| creating.set(true))>"New character"</Button>
            }.into_any())
        />
        {move || {
            let e = error.get();
            (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
        }}
        {move || match data.get() {
            None => view! { <Loading /> }.into_any(),
            Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
            Some(Ok((campaigns, groups))) => groups
                .into_iter()
                .map(|g| group_view(g, campaigns.clone()))
                .collect_view()
                .into_any(),
        }}
        {move || creating.get().then(|| view! {
            <CharacterWizard
                on_created=Callback::new(move |_| reload())
                on_close=Callback::new(move |_| creating.set(false))
            />
        })}
        <ConfirmDialog
            open=confirm_open
            title=Signal::derive(move || format!("Delete {}?", removing.get().map(|c| c.name).unwrap_or_default()))
            message="The character, its sheet, notes and player link go for good."
            confirm_label="Delete"
            on_confirm=Callback::new(remove)
            on_cancel=Callback::new(move |_| removing.set(None))
        />
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(name: &str, npc: bool) -> CharacterSummary {
        CharacterSummary {
            id: name.into(),
            campaign_id: None,
            name: name.into(),
            is_npc: npc,
            player_name: None,
            race_name: None,
            background_name: None,
            classes: Vec::new(),
            level: 0,
            role: None,
            location: None,
            faction: None,
            updated_at: String::new(),
        }
    }

    #[test]
    fn pcs_first_then_by_name() {
        let mut l = vec![c("Zed", false), c("Ann", true), c("Bo", false)];
        sort_characters(&mut l);
        let names: Vec<_> = l.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(names, ["Bo", "Zed", "Ann"]);
    }
}
