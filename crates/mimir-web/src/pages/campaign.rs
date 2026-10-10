//! `/campaigns/:id/…`: the campaign dashboard. One header row holds the
//! name and the tabs (Campaign, Modules, NPCs, PCs); the tab's page is the
//! router outlet under it.

use aurora_leptos::components::*;
use aurora_leptos::frame::{TabItem, Tabs};
use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::hooks::{use_location, use_params_map};
use mimir_wire::{CharacterSummary, ModuleSummary};

use crate::api;
use crate::components::{class_label, loaded, or_dash, DocumentBrowser, ListResource};

/// The tab of a dashboard path.
pub fn tab_of(path: &str) -> &'static str {
    match path.trim_end_matches('/').rsplit('/').next() {
        Some("modules") => "modules",
        Some("npcs") => "npcs",
        Some("pcs") => "pcs",
        _ => "campaign",
    }
}

/// The campaign id of the route.
fn campaign_id() -> Memo<String> {
    let params = use_params_map();
    Memo::new(move |_| params.get().get("id").unwrap_or_default())
}

#[component]
pub fn CampaignDashboard() -> impl IntoView {
    let id = campaign_id();
    let campaign = LocalResource::new(move || api::campaign(id.get()));
    let location = use_location();
    let tab = RwSignal::new(tab_of(&location.pathname.get_untracked()).to_string());
    Effect::new(move |_| tab.set(tab_of(&location.pathname.get()).to_string()));

    let tabs = move || {
        let base = format!("/campaigns/{}", id.get());
        vec![
            TabItem::new("campaign", "Campaign").href(base.clone()),
            TabItem::new("modules", "Modules").href(format!("{base}/modules")),
            TabItem::new("npcs", "NPCs").href(format!("{base}/npcs")),
            TabItem::new("pcs", "PCs").href(format!("{base}/pcs")),
        ]
    };
    let name = move || match campaign.get() {
        Some(Ok(c)) => c.name,
        _ => String::new(),
    };

    view! {
        {move || match campaign.get() {
            Some(Err(error)) => view! { <ErrorState error=error /> }.into_any(),
            _ => {
                view! {
                    <div class="mimir-dash">
                        <header class="mimir-dash__header">
                            <h1 class="mimir-dash__title">{name}</h1>
                            <Tabs tabs=tabs() value=tab label="Campaign sections" />
                        </header>
                        <Outlet />
                    </div>
                }
                    .into_any()
            }
        }}
    }
}

/// The Campaign tab: the campaign-level documents.
#[component]
pub fn CampaignTab() -> impl IntoView {
    let id = campaign_id();
    let docs: ListResource<_> =
        LocalResource::new(move || api::campaign_list(id.get(), "documents"));
    view! { <DocumentBrowser docs=docs /> }
}

#[component]
pub fn ModulesTab() -> impl IntoView {
    let id = campaign_id();
    let modules: ListResource<ModuleSummary> =
        LocalResource::new(move || api::campaign_list(id.get(), "modules"));
    let table = loaded(modules, |list: Vec<ModuleSummary>| {
        view! {
            <Table label="Modules" widths=vec!["4rem".into(), "30%".into(), "auto".into()]>
                <thead>
                    <tr>
                        <th>"#"</th>
                        <th>"Module"</th>
                        <th>"Description"</th>
                    </tr>
                </thead>
                <tbody>
                    {if list.is_empty() {
                        view! { <TableEmpty message="No modules yet." colspan=3 /> }.into_any()
                    } else {
                        list.into_iter()
                            .map(|m| {
                                view! {
                                    <tr>
                                        <td>{m.module_number}</td>
                                        <td>
                                            <a href=format!("/modules/{}", m.id)>{m.name}</a>
                                        </td>
                                        <td>{or_dash(&m.description)}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }}
                </tbody>
            </Table>
        }
    });
    view! { {table} }
}

fn characters_table(list: Vec<CharacterSummary>, npc: bool) -> impl IntoView {
    let headers = if npc {
        ["Name", "Role", "Location", "Faction"]
    } else {
        ["Name", "Player", "Race", "Class"]
    };
    let empty = if npc {
        "No NPCs yet."
    } else {
        "No player characters yet."
    };
    view! {
        <Table label=if npc { "NPCs" } else { "Player characters" } min_width="560px">
            <thead>
                <tr>{headers.map(|h| view! { <th>{h}</th> }).collect_view()}</tr>
            </thead>
            <tbody>
                {if list.is_empty() {
                    view! { <TableEmpty message=empty colspan=4 /> }.into_any()
                } else {
                    list.into_iter()
                        .map(|c| {
                            let cells = if npc {
                                [or_dash(&c.role), or_dash(&c.location), or_dash(&c.faction)]
                            } else {
                                [or_dash(&c.player_name), or_dash(&c.race_name), class_label(&c.classes)]
                            };
                            view! {
                                <tr>
                                    <td>{c.name}</td>
                                    {cells.map(|t| view! { <td>{t}</td> }).collect_view()}
                                </tr>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </tbody>
        </Table>
    }
}

#[component]
pub fn NpcsTab() -> impl IntoView {
    let id = campaign_id();
    let npcs: ListResource<CharacterSummary> =
        LocalResource::new(move || api::campaign_list(id.get(), "npcs"));
    let table = loaded(npcs, |list| characters_table(list, true));
    view! { {table} }
}

#[component]
pub fn PcsTab() -> impl IntoView {
    let id = campaign_id();
    let pcs: ListResource<CharacterSummary> =
        LocalResource::new(move || api::campaign_list(id.get(), "pcs"));
    let table = loaded(pcs, |list| characters_table(list, false));
    view! { {table} }
}

#[cfg(test)]
mod tests {
    use super::tab_of;

    #[test]
    fn the_tab_comes_from_the_path() {
        assert_eq!(tab_of("/campaigns/c1"), "campaign");
        assert_eq!(tab_of("/campaigns/c1/"), "campaign");
        assert_eq!(tab_of("/campaigns/c1/modules"), "modules");
        assert_eq!(tab_of("/campaigns/c1/npcs/"), "npcs");
        assert_eq!(tab_of("/campaigns/c1/pcs"), "pcs");
    }
}
