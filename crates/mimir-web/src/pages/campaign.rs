//! `/campaigns/:id/…`: the campaign dashboard. One header row holds the
//! name and the tabs (Campaign, Modules, NPCs, PCs); the tab's page is the
//! router outlet under it.

use aurora_leptos::components::*;
use aurora_leptos::frame::{TabItem, Tabs};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_navigate;
use leptos_router::hooks::{use_location, use_params_map};
use mimir_wire::{CharacterSummary, Direction, ModuleSummary, Move};

use crate::api;
use crate::authoring::campaigns::CampaignActions;
use crate::authoring::documents::DocumentWorkspace;
use crate::authoring::documents::OrderButtons;
use crate::authoring::maps::MapsPanel;
use crate::authoring::modules::ModuleDialog;
use crate::authoring::wizard::CharacterWizard;
use crate::components::{class_label, loaded, or_dash, ListResource};

/// The tab of a dashboard path.
pub fn tab_of(path: &str) -> &'static str {
    match path.trim_end_matches('/').rsplit('/').next() {
        Some("modules") => "modules",
        Some("npcs") => "npcs",
        Some("pcs") => "pcs",
        Some("homebrew") => "homebrew",
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
    let version = RwSignal::new(0u32);
    let campaign = LocalResource::new(move || {
        version.track();
        api::campaign(id.get())
    });
    // Remember it: the header's selector and Home's "Continue".
    Effect::new(move |_| {
        if let (Some(Ok(c)), Some(current)) =
            (campaign.get(), crate::current::CurrentCampaign::get())
        {
            current.set(&c.id);
        }
    });
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
            TabItem::new("homebrew", "Homebrew").href(format!("{base}/homebrew")),
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
                            <div class="mimir-dash__titlebar">
                                <h1 class="mimir-dash__title">{name}</h1>
                                {move || match campaign.get() {
                                    Some(Ok(c)) => Some(view! {
                                        <CampaignActions campaign=c on_changed=Callback::new(move |_| version.update(|v| *v += 1)) />
                                    }),
                                    _ => None,
                                }}
                            </div>
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
    view! {
        <DocumentWorkspace base=Signal::derive(move || format!("/campaigns/{}", id.get())) />
        <section class="mimir-dash__maps" aria-label="Campaign maps">
            <MapsPanel campaign_id=id />
        </section>
    }
}

#[component]
pub fn ModulesTab() -> impl IntoView {
    let id = campaign_id();
    let version = RwSignal::new(0u32);
    let modules: ListResource<ModuleSummary> = LocalResource::new(move || {
        version.track();
        api::campaign_list(id.get(), "modules")
    });
    let creating = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    let navigate = use_navigate();
    let move_module = move |module: String, direction: Direction| {
        spawn_local(async move {
            let path = format!("/modules/{module}/move");
            match api::send::<_, Vec<ModuleSummary>>("POST", &path, &Move { direction }).await {
                Ok(_) => version.update(|v| *v += 1),
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    let table = loaded(modules, move |list: Vec<ModuleSummary>| {
        let n = list.len();
        view! {
            <Table label="Modules" widths=vec!["4rem".into(), "30%".into(), "auto".into(), "6rem".into()]>
                <thead>
                    <tr>
                        <th>"#"</th>
                        <th>"Module"</th>
                        <th>"Description"</th>
                        <th>"Order"</th>
                    </tr>
                </thead>
                <tbody>
                    {if list.is_empty() {
                        view! { <TableEmpty message="No modules yet." colspan=4 /> }.into_any()
                    } else {
                        list.into_iter()
                            .enumerate()
                            .map(|(i, m)| {
                                let mid = m.id.clone();
                                view! {
                                    <tr>
                                        <td>{m.module_number}</td>
                                        <td>
                                            <a href=format!("/modules/{}", m.id)>{m.name.clone()}</a>
                                        </td>
                                        <td>{or_dash(&m.description)}</td>
                                        <td>
                                            <OrderButtons
                                                name=m.name
                                                first=i == 0
                                                last=i + 1 == n
                                                on_move=Callback::new(move |d| move_module(mid.clone(), d))
                                            />
                                        </td>
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
    view! {
        <Group>
            <Button on_click=Callback::new(move |_| creating.set(true))>"New module"</Button>
        </Group>
        {move || {
            let e = error.get();
            (!e.is_empty()).then(|| view! { <Alert color=aurora_leptos::tokens::token::BAD>{e}</Alert> })
        }}
        {table}
        {move || view! {
            <ModuleDialog
                open=creating
                campaign_id=id.get()
                on_saved=Callback::new({
                    let navigate = navigate.clone();
                    move |m: ModuleSummary| navigate(&format!("/modules/{}", m.id), Default::default())
                })
            />
        }}
    }
}

/// The lines of a character's card: an NPC's role, place and faction; a
/// PC's player, race and class. Empty ones are left out.
pub fn card_lines(c: &CharacterSummary, npc: bool) -> Vec<String> {
    let lines = if npc {
        [
            c.role.clone().map(|r| format!("Role: {r}")),
            c.location.clone().map(|l| format!("Location: {l}")),
            c.faction.clone().map(|f| format!("Faction: {f}")),
        ]
    } else {
        [
            c.player_name.clone().map(|p| format!("Player: {p}")),
            c.race_name.clone(),
            Some(class_label(&c.classes)).filter(|l| l != "—"),
        ]
    };
    lines
        .into_iter()
        .flatten()
        .filter(|l| !l.trim().is_empty())
        .collect()
}

/// The PCs or NPCs as cards: View, PDF, and a PC's player link
/// (MIMIR-T-0733, the desktop's character cards).
fn characters_cards(
    list: Vec<CharacterSummary>,
    npc: bool,
    printing: RwSignal<Option<String>>,
) -> impl IntoView {
    let label = if npc { "NPCs" } else { "Player characters" };
    if list.is_empty() {
        let empty = if npc {
            "No NPCs yet."
        } else {
            "No player characters yet."
        };
        return view! { <Empty message=empty /> }.into_any();
    }
    view! {
        <div class="mimir-cards mimir-char-cards" role="list" aria-label=label>
            {list.into_iter().map(|c| {
                let lines = card_lines(&c, npc);
                let open = RwSignal::new(false);
                let (id, pdf_id) = (c.id.clone(), c.id.clone());
                let name = c.name.clone();
                let href = format!("/characters/{}", c.id);
                let view_label = format!("View {}", c.name);
                let (aria, title) = (c.name.clone(), c.name.clone());
                view! {
                    <article class="cl-card mimir-char-card" role="listitem" aria-label=aria>
                        <a class="cl-card__title mimir-char-card__name" href=href.clone()>{title}</a>
                        {lines.into_iter().map(|l| view! { <Text size="sm" dimmed=true>{l}</Text> }).collect_view()}
                        <Group>
                            <a class="cl-btn cl-btn--light" href=href aria-label=view_label>"View"</a>
                            <Button variant="subtle" on_click=Callback::new(move |_| printing.set(Some(pdf_id.clone())))>"PDF…"</Button>
                            {(!npc).then(|| view! {
                                <Button variant="subtle" on_click=Callback::new(move |_| open.update(|o| *o = !*o))>
                                    {move || if open.get() { "Close" } else { "Player link" }}
                                </Button>
                            })}
                        </Group>
                        {move || open.get().then(|| view! {
                            <LinkPanel character_id=id.clone() name=name.clone() />
                        })}
                    </article>
                }
            }).collect_view()}
        </div>
    }
    .into_any()
}

/// The PCs or NPCs of the campaign, and New PC / New NPC (the wizard,
/// MIMIR-T-0721).
fn characters_tab(npc: bool) -> impl IntoView {
    let id = campaign_id();
    let version = RwSignal::new(0u32);
    let list: ListResource<CharacterSummary> = LocalResource::new(move || {
        version.track();
        api::campaign_list(id.get(), if npc { "npcs" } else { "pcs" })
    });
    let creating = RwSignal::new(false);
    let printing = RwSignal::new(None::<String>);
    let pdf_open = RwSignal::new(false);
    Effect::new(move |_| pdf_open.set(printing.with(Option::is_some)));
    Effect::new(move |prev: Option<bool>| {
        let now = pdf_open.get();
        if prev == Some(true) && !now {
            printing.set(None);
        }
        now
    });
    let cards = loaded(list, move |l| characters_cards(l, npc, printing));
    view! {
        <Group>
            <Button on_click=Callback::new(move |_| creating.set(true))>{if npc { "New NPC" } else { "New PC" }}</Button>
        </Group>
        {cards}
        <crate::admin::pdf::PdfDialog
            open=pdf_open
            title="PDF of the character"
            path=Signal::derive(move || format!("/characters/{}/pdf", printing.get().unwrap_or_default()))
            options=crate::admin::pdf::CHARACTER
            file="character.pdf"
        />
        {move || creating.get().then(|| view! {
            <CharacterWizard
                campaign_id=id.get()
                npc=npc
                on_created=Callback::new(move |_| version.update(|v| *v += 1))
                on_close=Callback::new(move |_| creating.set(false))
            />
        })}
    }
}

#[component]
pub fn NpcsTab() -> impl IntoView {
    characters_tab(true)
}

#[component]
pub fn PcsTab() -> impl IntoView {
    characters_tab(false)
}

#[cfg(test)]
mod tests {
    use super::tab_of;

    #[test]
    fn card_lines_leave_out_empty_fields() {
        let c: mimir_wire::CharacterSummary = serde_json::from_value(serde_json::json!({
            "id": "c", "campaign_id": null, "name": "Halia", "is_npc": true, "player_name": null,
            "race_name": "Human", "background_name": null, "classes": [], "level": 0,
            "role": "Guild agent", "location": null, "faction": "Zhentarim", "updated_at": ""
        }))
        .unwrap();
        assert_eq!(
            super::card_lines(&c, true),
            vec!["Role: Guild agent", "Faction: Zhentarim"]
        );
        assert_eq!(super::card_lines(&c, false), vec!["Human"]);
    }

    #[test]
    fn the_tab_comes_from_the_path() {
        assert_eq!(tab_of("/campaigns/c1"), "campaign");
        assert_eq!(tab_of("/campaigns/c1/"), "campaign");
        assert_eq!(tab_of("/campaigns/c1/modules"), "modules");
        assert_eq!(tab_of("/campaigns/c1/npcs/"), "npcs");
        assert_eq!(tab_of("/campaigns/c1/pcs"), "pcs");
        assert_eq!(tab_of("/campaigns/c1/homebrew"), "homebrew");
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
