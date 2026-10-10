//! `/modules/:id`: a module, read-only: its documents, NPCs, maps and
//! dangers (monsters).

use aurora_leptos::components::*;
use aurora_leptos::data::SectionLabel;
use aurora_leptos::frame::{PageHeader, TabItem, TabPanel, Tabs};
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};
use mimir_wire::{MapSummary, ModuleMonsterSummary, ModuleNpcSummary};

use crate::api;
use crate::components::{loaded, or_dash, DocumentBrowser, ListResource};

/// The sections of the module page: (value, label).
const SECTIONS: [(&str, &str); 4] = [
    ("documents", "Documents"),
    ("npcs", "NPCs"),
    ("maps", "Maps"),
    ("dangers", "Dangers"),
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
    let module = LocalResource::new(move || api::module(id.get()));
    let docs: ListResource<_> = LocalResource::new(move || api::module_list(id.get(), "documents"));
    let npcs: ListResource<ModuleNpcSummary> =
        LocalResource::new(move || api::module_list(id.get(), "npcs"));
    let maps: ListResource<MapSummary> =
        LocalResource::new(move || api::module_list(id.get(), "maps"));
    let monsters: ListResource<ModuleMonsterSummary> =
        LocalResource::new(move || api::module_list(id.get(), "monsters"));

    let header = loaded(module, |m| {
        view! {
            <PageHeader
                title=m.name
                sub=format!("Module {}", m.module_number)
                back_href=format!("/campaigns/{}/modules", m.campaign_id)
                back_label="Modules"
            />
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
                <DocumentBrowser docs=docs />
            </TabPanel>
            <TabPanel value="npcs">{loaded(npcs, npc_view)}</TabPanel>
            <TabPanel value="maps">{loaded(maps, map_view)}</TabPanel>
            <TabPanel value="dangers">{loaded(monsters, danger_view)}</TabPanel>
        </Tabs>
    }
}

fn npc_view(list: Vec<ModuleNpcSummary>) -> impl IntoView {
    let n = list.len();
    view! {
        <SectionLabel label="NPCs" count=n />
        <Table label="NPCs" min_width="480px">
            <thead><tr><th>"Name"</th><th>"Role"</th><th>"Description"</th></tr></thead>
            <tbody>
                {if list.is_empty() {
                    view! { <TableEmpty message="No NPCs in this module." colspan=3 /> }.into_any()
                } else {
                    list.into_iter().map(|n| view! {
                        <tr><td>{n.name}</td><td>{or_dash(&n.role)}</td><td>{or_dash(&n.description)}</td></tr>
                    }).collect_view().into_any()
                }}
            </tbody>
        </Table>
    }
}

fn map_view(list: Vec<MapSummary>) -> impl IntoView {
    let n = list.len();
    view! {
        <SectionLabel label="Maps" count=n />
        <Table label="Maps" min_width="480px">
            <thead><tr><th>"Map"</th><th>"Lighting"</th><th>"Fog of war"</th></tr></thead>
            <tbody>
                {if list.is_empty() {
                    view! { <TableEmpty message="No maps in this module." colspan=3 /> }.into_any()
                } else {
                    list.into_iter().map(|m| view! {
                        <tr>
                            <td><a href=format!("/maps/{}", m.id)>{m.name}</a></td>
                            <td>{m.lighting_mode}</td>
                            <td>{if m.fog_enabled { "On" } else { "Off" }}</td>
                        </tr>
                    }).collect_view().into_any()
                }}
            </tbody>
        </Table>
    }
}

fn danger_view(list: Vec<ModuleMonsterSummary>) -> impl IntoView {
    let n = list.len();
    view! {
        <SectionLabel label="Dangers" count=n />
        <Table label="Dangers" min_width="480px">
            <thead><tr><th>"Monster"</th><th>"Count"</th><th>"Source"</th></tr></thead>
            <tbody>
                {if list.is_empty() {
                    view! { <TableEmpty message="No monsters in this module." colspan=3 /> }.into_any()
                } else {
                    list.into_iter().map(|m| {
                        let source = if m.homebrew_monster_id.is_some() {
                            "Homebrew".to_string()
                        } else {
                            or_dash(&m.monster_source)
                        };
                        view! { <tr><td>{m.name}</td><td>{m.quantity}</td><td>{source}</td></tr> }
                    }).collect_view().into_any()
                }}
            </tbody>
        </Table>
    }
}

#[cfg(test)]
mod tests {
    use super::section_of;

    #[test]
    fn the_section_comes_from_the_address() {
        assert_eq!(section_of(None), "documents");
        assert_eq!(section_of(Some("maps")), "maps");
        assert_eq!(section_of(Some("dangers")), "dangers");
        assert_eq!(section_of(Some("nonsense")), "documents");
    }
}
