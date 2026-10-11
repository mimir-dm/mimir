//! The traps and points of interest of a module's maps, on its Dangers
//! tab (MIMIR-T-0732). A name opens the details: a trap that names a
//! catalog trap opens it in the detail drawer; otherwise, and for a point
//! of interest, a dialog shows what the map holds.

use aurora_leptos::components::*;
use aurora_leptos::data::SectionLabel;
use aurora_leptos::frame::Modal;
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_wire::{CatalogPage, MapSummary, Poi, Trap};

use crate::api;
use crate::map::setup::poi_emoji;
use crate::reference::detail::{Reference, Target};

/// A marker with the name of its map.
#[derive(Debug, Clone, PartialEq)]
pub enum Marker {
    Trap(Trap, String),
    Poi(Poi, String),
}

/// The traps and POIs of the maps, each with its map's name, in map order.
pub fn markers_of(
    maps: &[MapSummary],
    traps: Vec<Vec<Trap>>,
    pois: Vec<Vec<Poi>>,
) -> (Vec<Marker>, Vec<Marker>) {
    let name = |i: usize| maps.get(i).map(|m| m.name.clone()).unwrap_or_default();
    let t = traps
        .into_iter()
        .enumerate()
        .flat_map(|(i, l)| l.into_iter().map(move |t| (t, i)))
        .map(|(t, i)| Marker::Trap(t, name(i)))
        .collect();
    let p = pois
        .into_iter()
        .enumerate()
        .flat_map(|(i, l)| l.into_iter().map(move |p| (p, i)))
        .map(|(p, i)| Marker::Poi(p, name(i)))
        .collect();
    (t, p)
}

async fn load(
    module_id: String,
) -> Result<(Vec<Marker>, Vec<Marker>), aurora_leptos::tokens::ApiError> {
    let maps: Vec<MapSummary> = api::module_list(module_id, "maps").await?;
    let (mut traps, mut pois) = (Vec::new(), Vec::new());
    for m in &maps {
        traps.push(api::get::<Vec<Trap>>(format!("/maps/{}/traps", m.id)).await?);
        pois.push(api::get::<Vec<Poi>>(format!("/maps/{}/pois", m.id)).await?);
    }
    Ok(markers_of(&maps, traps, pois))
}

#[component]
pub fn MarkersPanel(#[prop(into)] module_id: Signal<String>) -> impl IntoView {
    let markers = LocalResource::new(move || load(module_id.get()));
    let reference = Reference::get();
    let showing = RwSignal::new(None::<Marker>);
    // A trap: its catalog entry when one has its name, else its own details.
    let open = move |m: Marker| {
        if let (Marker::Trap(t, _), Some(r)) = (&m, reference) {
            let name = t.name.clone();
            spawn_local(async move {
                let path = format!(
                    "/catalog/search/traps?limit=20&search={}",
                    String::from(js_sys::encode_uri_component(&name))
                );
                let found = api::get::<CatalogPage>(path).await.ok().and_then(|p| {
                    p.rows
                        .into_iter()
                        .find(|r| r.name.eq_ignore_ascii_case(&name))
                });
                match found {
                    Some(row) => r.open(Target::new("traps", &row.source, &row.name)),
                    None => showing.set(Some(m)),
                }
            });
        } else {
            showing.set(Some(m));
        }
    };
    let table = move |label: &'static str, list: Vec<Marker>| {
        let n = list.len();
        view! {
            <SectionLabel label=label count=n />
            <Table label=label min_width="420px">
                <thead><tr><th>"Name"</th><th>"Map"</th><th>"Players see it"</th></tr></thead>
                <tbody>
                    {if list.is_empty() {
                        view! { <TableEmpty message="None on the module's maps." colspan=3 /> }.into_any()
                    } else {
                        list.into_iter().map(|m| {
                            let (name, map, seen) = match &m {
                                Marker::Trap(t, map) => (t.name.clone(), map.clone(), t.visible),
                                Marker::Poi(p, map) => (format!("{} {}", poi_emoji(&p.icon), p.name), map.clone(), p.visible),
                            };
                            view! {
                                <tr>
                                    <td><button type="button" class="mimir-link-button" on:click=move |_| open(m.clone())>{name}</button></td>
                                    <td>{map}</td>
                                    <td>{if seen { "Yes" } else { "No" }}</td>
                                </tr>
                            }
                        }).collect_view().into_any()
                    }}
                </tbody>
            </Table>
        }
    };
    view! {
        {move || match markers.get() {
            None => view! { <Loading /> }.into_any(),
            Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
            Some(Ok((traps, pois))) => view! {
                {table("Traps", traps)}
                {table("Points of interest", pois)}
            }.into_any(),
        }}
        <MarkerDetails showing=showing />
    }
}

/// The details of a trap or a point of interest, as its map holds them.
#[component]
fn MarkerDetails(showing: RwSignal<Option<Marker>>) -> impl IntoView {
    let open = RwSignal::new(false);
    Effect::new(move |_| open.set(showing.with(Option::is_some)));
    Effect::new(move |prev: Option<bool>| {
        let now = open.get();
        if prev == Some(true) && !now {
            showing.set(None);
        }
        now
    });
    let title = Signal::derive(move || match showing.get() {
        Some(Marker::Trap(t, _)) => t.name,
        Some(Marker::Poi(p, _)) => p.name,
        None => String::new(),
    });
    let rows = move || -> Vec<(&'static str, String)> {
        let dash = |v: Option<String>| {
            v.filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "—".into())
        };
        match showing.get() {
            Some(Marker::Trap(t, map)) => vec![
                ("Map", map),
                (
                    "DC",
                    t.dc.map(|d| d.to_string()).unwrap_or_else(|| "—".into()),
                ),
                ("Trigger", dash(t.trigger_description)),
                ("Effect", dash(t.effect_description)),
                ("Notes", dash(t.description)),
                (
                    "State",
                    if t.triggered {
                        "Triggered".into()
                    } else {
                        "Armed".into()
                    },
                ),
                (
                    "Players see it",
                    if t.visible { "Yes".into() } else { "No".into() },
                ),
            ],
            Some(Marker::Poi(p, map)) => vec![
                ("Map", map),
                ("Icon", format!("{} {}", poi_emoji(&p.icon), p.icon)),
                ("Description", dash(p.description)),
                (
                    "Players see it",
                    if p.visible { "Yes".into() } else { "No".into() },
                ),
            ],
            None => Vec::new(),
        }
    };
    let map_href = move || match showing.get() {
        Some(Marker::Trap(t, _)) => format!("/maps/{}", t.map_id),
        Some(Marker::Poi(p, _)) => format!("/maps/{}", p.map_id),
        None => String::new(),
    };
    view! {
        <Modal open=open title=title>
            <Stack gap="sm">
                <dl class="mimir-details">
                    {move || rows().into_iter().map(|(k, v)| view! { <dt>{k}</dt><dd>{v}</dd> }).collect_view()}
                </dl>
                <Group>
                    <a class="cl-btn cl-btn--light" href=map_href>"Open the map"</a>
                    <Button variant="subtle" on_click=Callback::new(move |_| open.set(false))>"Close"</Button>
                </Group>
            </Stack>
        </Modal>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(id: &str, name: &str) -> MapSummary {
        serde_json::from_value(serde_json::json!({
            "id": id, "campaign_id": "c", "module_id": "m", "name": name,
            "description": null, "sort_order": 0, "lighting_mode": "bright", "fog_enabled": false
        }))
        .unwrap()
    }

    fn trap(name: &str) -> Trap {
        Trap {
            id: name.into(),
            map_id: "m1".into(),
            grid_x: 0,
            grid_y: 0,
            name: name.into(),
            description: None,
            trigger_description: None,
            effect_description: None,
            dc: Some(12),
            triggered: false,
            visible: false,
        }
    }

    #[test]
    fn markers_carry_their_map() {
        let maps = vec![map("m1", "Cave"), map("m2", "Hall")];
        let (traps, pois) = markers_of(
            &maps,
            vec![vec![trap("Pit")], vec![trap("Darts")]],
            vec![vec![], vec![]],
        );
        assert_eq!(traps.len(), 2);
        assert!(matches!(&traps[1], Marker::Trap(t, m) if t.name == "Darts" && m == "Hall"));
        assert!(pois.is_empty());
    }
}
