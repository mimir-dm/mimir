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
    let link_header = (!npc).then(|| view! { <th>"Link"</th> });
    let empty = if npc {
        "No NPCs yet."
    } else {
        "No player characters yet."
    };
    view! {
        <Table label=if npc { "NPCs" } else { "Player characters" } min_width="560px">
            <thead>
                <tr>{headers.map(|h| view! { <th>{h}</th> }).collect_view()}{link_header}</tr>
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
                            let open = RwSignal::new(false);
                            let id = c.id.clone();
                            let name = c.name.clone();
                            view! {
                                <tr>
                                    <td><a href=format!("/characters/{}", c.id)>{c.name}</a></td>
                                    {cells.map(|t| view! { <td>{t}</td> }).collect_view()}
                                    {(!npc).then(|| view! {
                                        <td>
                                            <Button variant="subtle" on_click=Callback::new(move |_| open.update(|o| *o = !*o))>
                                                {move || if open.get() { "Close" } else { "Player link" }}
                                            </Button>
                                        </td>
                                    })}
                                </tr>
                                {move || open.get().then(|| view! {
                                    <tr class="mimir-link-row">
                                        <td colspan="5">
                                            <LinkPanel character_id=id.clone() name=name.clone() />
                                        </td>
                                    </tr>
                                })}
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

/// The full address of a player link path.
pub fn link_url(origin: &str, path: &str) -> String {
    format!("{}{}", origin.trim_end_matches('/'), path)
}

/// A QR code of text, as SVG (black on white: scanners need the contrast).
pub fn qr_svg(text: &str) -> Option<String> {
    use qrcode::render::svg;
    let code = qrcode::QrCode::new(text.as_bytes()).ok()?;
    Some(
        code.render::<svg::Color>()
            .min_dimensions(180, 180)
            .dark_color(svg::Color("#000000"))
            .light_color(svg::Color("#ffffff"))
            .build(),
    )
}

/// The player link of a player character (MIMIR-T-0716): status, a new
/// link (shown once, with Copy and a QR code), revoke.
#[component]
fn LinkPanel(character_id: String, name: String) -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let path = format!("/characters/{character_id}/link");
    let status = {
        let path = path.clone();
        LocalResource::new(move || {
            refresh.track();
            api::get::<mimir_wire::LinkStatus>(path.clone())
        })
    };
    let fresh = RwSignal::new(None::<String>);
    let copied = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    let origin = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default();

    let issue = {
        let path = path.clone();
        move |_| {
            let active = status
                .get_untracked()
                .and_then(Result::ok)
                .is_some_and(|s| s.active);
            if active
                && !web_sys::window()
                    .and_then(|w| {
                        w.confirm_with_message("Make a new link? The old link stops working.")
                            .ok()
                    })
                    .unwrap_or(false)
            {
                return;
            }
            let path = path.clone();
            let origin = origin.clone();
            leptos::task::spawn_local(async move {
                match api::send::<_, mimir_wire::NewLink>("POST", &path, &serde_json::json!({}))
                    .await
                {
                    Ok(l) => {
                        fresh.set(Some(link_url(&origin, &l.path)));
                        copied.set(false);
                        refresh.update(|v| *v += 1);
                    }
                    Err(e) => error.set(api::message(e)),
                }
            });
        }
    };
    let revoke = {
        let path = path.clone();
        move |_| {
            let path = path.clone();
            leptos::task::spawn_local(async move {
                match api::delete(&path).await {
                    Ok(()) => {
                        fresh.set(None);
                        refresh.update(|v| *v += 1);
                    }
                    Err(e) => error.set(api::message(e)),
                }
            });
        }
    };
    let copy = move |_| {
        if let (Some(url), Some(w)) = (fresh.get_untracked(), web_sys::window()) {
            let _ = w.navigator().clipboard().write_text(&url);
            copied.set(true);
        }
    };

    view! {
        <div class="mimir-link" aria-label=format!("Player link of {name}")>
            {move || match status.get() {
                Some(Ok(s)) if s.active => view! {
                    <Text size="sm">{format!("A link is active (made {}).", s.created_at.unwrap_or_default())}</Text>
                }.into_any(),
                Some(Ok(_)) => view! { <Text size="sm" dimmed=true>"No link yet."</Text> }.into_any(),
                Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                None => view! { <Loading /> }.into_any(),
            }}
            {move || fresh.get().map(|url| {
                let qr = qr_svg(&url).unwrap_or_default();
                view! {
                    <div class="mimir-link__new">
                        <Text size="sm">"Open this on the player's device. It shows only now."</Text>
                        <code class="mimir-link__url">{url.clone()}</code>
                        <div class="mimir-link__qr" inner_html=qr></div>
                        <Button variant="light" on_click=Callback::new(copy)>
                            {move || if copied.get() { "Copied" } else { "Copy link" }}
                        </Button>
                    </div>
                }
            })}
            <Group>
                <Button on_click=Callback::new(issue)>
                    {move || {
                        let active = status.get().and_then(Result::ok).is_some_and(|s| s.active);
                        if active { "New link" } else { "Make a link" }
                    }}
                </Button>
                {move || status.get().and_then(Result::ok).is_some_and(|s| s.active).then(|| {
                    let revoke = revoke.clone();
                    view! { <Button variant="light" bad=true on_click=Callback::new(revoke)>"Revoke"</Button> }
                })}
            </Group>
            {move || {
                let e = error.get();
                (!e.is_empty()).then(|| view! { <Text size="sm">{e}</Text> })
            }}
        </div>
    }
}

#[cfg(test)]
mod link_tests {
    use super::*;

    #[test]
    fn link_urls_and_qr_codes() {
        assert_eq!(
            link_url("https://mimir.example/", "/play/abc"),
            "https://mimir.example/play/abc"
        );
        let svg = qr_svg("https://mimir.example/play/abc").unwrap();
        assert!(svg.starts_with("<?xml") || svg.contains("<svg"));
        assert!(svg.contains("#000000"));
    }
}
