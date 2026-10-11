//! Token setup on the DM map (MIMIR-T-0730): the placement form (name,
//! colour, seen by players, a catalog monster or trap), the point of
//! interest dialog, and the lists of what is on the map. The desktop's
//! token setup window, in the page.

use aurora_leptos::components::*;
use aurora_leptos::frame::Modal;
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_wire::{CatalogPage, Choice, Light, Poi, Token, Trap};
use serde_json::json;

use crate::api;

/// The icons of a point of interest: value, emoji, label (the desktop's).
pub const POI_ICONS: &[(&str, &str, &str)] = &[
    ("pin", "📍", "Pin"),
    ("star", "⭐", "Star"),
    ("skull", "💀", "Skull"),
    ("chest", "📦", "Chest"),
    ("door", "🚪", "Door"),
    ("secret", "🔮", "Secret"),
    ("question", "❓", "Question"),
    ("exclamation", "❗", "Exclamation"),
];

/// The preset colours of a point of interest.
pub const POI_COLORS: &[(&str, &str)] = &[
    ("#3b82f6", "Blue"),
    ("#22c55e", "Green"),
    ("#ef4444", "Red"),
    ("#f59e0b", "Amber"),
    ("#8b5cf6", "Purple"),
    ("#ec4899", "Pink"),
    ("#06b6d4", "Cyan"),
    ("#6b7280", "Grey"),
];

/// The emoji of a POI icon (a pin for an unknown one).
pub fn poi_emoji(icon: &str) -> &'static str {
    POI_ICONS
        .iter()
        .find(|i| i.0 == icon)
        .map(|i| i.1)
        .unwrap_or("📍")
}

/// What a tap on the map places.
#[derive(Debug, Clone, PartialEq)]
pub enum Place {
    Pc,
    Monster(String),
    Npc(String),
    /// A monster from the catalog: it joins the module on placing.
    CatalogMonster,
    Torch,
    Lantern,
    Candle,
    Trap,
    Poi,
}

impl Place {
    /// From the value of the place menu ("pc", "monster:<id>", …).
    pub fn parse(value: &str) -> Option<Place> {
        match value.split_once(':') {
            Some(("monster", id)) => Some(Place::Monster(id.to_string())),
            Some(("npc", id)) => Some(Place::Npc(id.to_string())),
            _ => match value {
                "pc" => Some(Place::Pc),
                "catalog-monster" => Some(Place::CatalogMonster),
                "torch" => Some(Place::Torch),
                "lantern" => Some(Place::Lantern),
                "candle" => Some(Place::Candle),
                "trap" => Some(Place::Trap),
                "poi" => Some(Place::Poi),
                _ => None,
            },
        }
    }

    /// A token (the form has a name, a colour and "Players see it").
    pub fn is_token(&self) -> bool {
        matches!(
            self,
            Place::Pc | Place::Monster(_) | Place::Npc(_) | Place::CatalogMonster
        )
    }

    /// The place menu stays on it after a tap: monsters and NPCs go on
    /// the map more than once.
    pub fn repeats(&self) -> bool {
        matches!(
            self,
            Place::Monster(_) | Place::Npc(_) | Place::CatalogMonster
        )
    }

    /// Seen by the players at first: tokens yes, traps no.
    pub fn seen_at_first(&self) -> bool {
        !matches!(self, Place::Trap)
    }
}

/// The placement form.
#[derive(Clone, Copy)]
pub struct PlaceForm {
    /// The token's or trap's name (a monster's: empty keeps its own).
    pub name: RwSignal<String>,
    /// The token's colour (`#rrggbb`); empty: by its kind.
    pub color: RwSignal<String>,
    pub visible: RwSignal<bool>,
    /// The catalog monster to place.
    pub monster: RwSignal<Option<Choice>>,
}

impl PlaceForm {
    pub fn new() -> Self {
        PlaceForm {
            name: RwSignal::new(String::new()),
            color: RwSignal::new(String::new()),
            visible: RwSignal::new(true),
            monster: RwSignal::new(None),
        }
    }

    /// Start a new placement of this kind.
    pub fn reset(&self, kind: Option<&Place>) {
        self.name.set(String::new());
        self.color.set(String::new());
        self.visible
            .set(kind.map(Place::seen_at_first).unwrap_or(true));
        self.monster.set(None);
    }

    /// The token fields for `POST /maps/{id}/tokens` (no cell, no link).
    pub fn token_fields(&self) -> serde_json::Value {
        let name = self.name.get_untracked().trim().to_string();
        let color = self.color.get_untracked();
        json!({
            "label": (!name.is_empty()).then_some(name),
            "color": (!color.is_empty()).then_some(color),
            "hidden": !self.visible.get_untracked(),
        })
    }
}

impl Default for PlaceForm {
    fn default() -> Self {
        Self::new()
    }
}

/// The fields of the placement form, under the hint.
#[component]
pub fn PlaceOptions(
    #[prop(into)] kind: Signal<Option<Place>>,
    form: PlaceForm,
    #[prop(into)] campaign_id: Signal<String>,
) -> impl IntoView {
    move || {
        let k = kind.get()?;
        let token = k.is_token();
        let name_label = match k {
            Place::Pc => "Name",
            Place::Trap => "Trap name",
            _ => "Name (optional)",
        };
        let has_name = token || k == Place::Trap;
        Some(view! {
            <div class="mimir-place" role="group" aria-label="Placement">
                {(k == Place::CatalogMonster).then(|| view! { <CatalogMonsterPicker form=form campaign_id=campaign_id /> })}
                {(k == Place::Trap).then(|| view! { <CatalogTrapPicker form=form /> })}
                {has_name.then(|| view! { <TextInput label=name_label value=form.name /> })}
                {token.then(|| view! {
                    <label class="mimir-place__color">
                        <span class="cl-text cl-text--sm">"Colour"</span>
                        <input
                            type="color"
                            aria-label="Token colour"
                            prop:value=move || {
                                let c = form.color.get();
                                if c.is_empty() { "#888888".to_string() } else { c }
                            }
                            on:input=move |ev| form.color.set(event_target_value(&ev))
                        />
                    </label>
                })}
                {has_name.then(|| view! { <Switch label="Players see it" checked=form.visible /> })}
                {(k == Place::Poi).then(|| view! {
                    <Text size="sm" dimmed=true>"Tap the map, then fill in the point of interest."</Text>
                })}
            </div>
        })
    }
}

/// Search the catalog (and the campaign's homebrew) for a monster to place.
#[component]
fn CatalogMonsterPicker(
    form: PlaceForm,
    #[prop(into)] campaign_id: Signal<String>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let results = RwSignal::new(None::<Result<Vec<Choice>, String>>);
    let find = move || {
        let q = search.get_untracked();
        spawn_local(async move {
            let path = format!(
                "/catalog/monsters?search={}&campaign={}",
                js_sys::encode_uri_component(q.trim()),
                js_sys::encode_uri_component(&campaign_id.get_untracked())
            );
            results.set(Some(
                api::get::<Vec<Choice>>(path).await.map_err(api::message),
            ));
        });
    };
    view! {
        <form class="mimir-place__search" on:submit=move |ev| { ev.prevent_default(); find(); }>
            <TextInput aria_label="Monster name" placeholder="Monster name" value=search />
            <Button variant="light" button_type="submit">"Search"</Button>
        </form>
        {move || form.monster.get().map(|m| view! {
            <Text size="sm" bold=true>{format!("Placing: {} ({})", m.name, m.source)}</Text>
        })}
        {move || results.get().map(|r| match r {
            Err(e) => view! { <Alert color=token::BAD>{e}</Alert> }.into_any(),
            Ok(list) if list.is_empty() => view! { <Text size="sm" dimmed=true>"No monster by that name."</Text> }.into_any(),
            Ok(list) => view! {
                <ul class="mimir-place__results" aria-label="Monsters found">
                    {list.into_iter().take(8).map(|c| {
                        let pick = c.clone();
                        let label = format!("{} ({})", c.name, c.source);
                        view! {
                            <li>
                                <Button variant="subtle" on_click=Callback::new(move |_| {
                                    form.monster.set(Some(pick.clone()));
                                    results.set(None);
                                })>{label}</Button>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            }.into_any(),
        })}
    }
}

/// Link a catalog trap: its name names the placed trap (a double click
/// then opens it).
#[component]
fn CatalogTrapPicker(form: PlaceForm) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let results = RwSignal::new(None::<Result<CatalogPage, String>>);
    let find = move || {
        let q = search.get_untracked();
        spawn_local(async move {
            let path = format!(
                "/catalog/search/traps?limit=8&search={}",
                js_sys::encode_uri_component(q.trim())
            );
            results.set(Some(
                api::get::<CatalogPage>(path).await.map_err(api::message),
            ));
        });
    };
    view! {
        <form class="mimir-place__search" on:submit=move |ev| { ev.prevent_default(); find(); }>
            <TextInput aria_label="Catalog trap" placeholder="Link a catalog trap" value=search />
            <Button variant="light" button_type="submit">"Search"</Button>
        </form>
        {move || results.get().map(|r| match r {
            Err(e) => view! { <Alert color=token::BAD>{e}</Alert> }.into_any(),
            Ok(page) if page.rows.is_empty() => view! { <Text size="sm" dimmed=true>"No trap by that name."</Text> }.into_any(),
            Ok(page) => view! {
                <ul class="mimir-place__results" aria-label="Traps found">
                    {page.rows.into_iter().map(|row| {
                        let name = row.name.clone();
                        let label = format!("{} ({})", row.name, row.source);
                        view! {
                            <li>
                                <Button variant="subtle" on_click=Callback::new(move |_| {
                                    form.name.set(name.clone());
                                    results.set(None);
                                })>{label}</Button>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            }.into_any(),
        })}
    }
}

/// A point of interest to create (no id) or change.
#[derive(Debug, Clone, PartialEq)]
pub struct PoiEdit {
    pub id: Option<String>,
    pub map_id: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub color: String,
    pub visible: bool,
}

impl PoiEdit {
    pub fn new(map_id: String, grid_x: i32, grid_y: i32) -> Self {
        PoiEdit {
            id: None,
            map_id,
            grid_x,
            grid_y,
            name: String::new(),
            description: String::new(),
            icon: "pin".into(),
            color: POI_COLORS[0].0.into(),
            visible: false,
        }
    }

    pub fn of(p: &Poi) -> Self {
        PoiEdit {
            id: Some(p.id.clone()),
            map_id: p.map_id.clone(),
            grid_x: p.grid_x,
            grid_y: p.grid_y,
            name: p.name.clone(),
            description: p.description.clone().unwrap_or_default(),
            icon: p.icon.clone(),
            color: p.color.clone().unwrap_or_else(|| POI_COLORS[0].0.into()),
            visible: p.visible,
        }
    }
}

/// The point of interest dialog: name, description, icon, colour, and a
/// preview (the desktop's). `edit`: `Some` opens it.
#[component]
pub fn PoiDialog(edit: RwSignal<Option<PoiEdit>>) -> impl IntoView {
    let open = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let icon = RwSignal::new(String::new());
    let color = RwSignal::new(String::new());
    let visible = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    Effect::new(move |_| match edit.get() {
        Some(e) => {
            name.set(e.name);
            description.set(e.description);
            icon.set(e.icon);
            color.set(e.color);
            visible.set(e.visible);
            error.set(String::new());
            open.set(true);
        }
        None => open.set(false),
    });
    Effect::new(move |prev: Option<bool>| {
        let now = open.get();
        if prev == Some(true) && !now {
            edit.set(None);
        }
        now
    });
    let save = move |_| {
        let Some(e) = edit.get_untracked() else {
            return;
        };
        let n = name.get_untracked().trim().to_string();
        if n.is_empty() {
            error.set("Give the point of interest a name.".into());
            return;
        }
        let body = json!({
            "name": n,
            "description": description.get_untracked(),
            "icon": icon.get_untracked(),
            "color": color.get_untracked(),
            "visible": visible.get_untracked(),
            "grid_x": e.grid_x,
            "grid_y": e.grid_y,
        });
        spawn_local(async move {
            let r = match &e.id {
                Some(id) => api::send::<_, Poi>("PATCH", &format!("/pois/{id}"), &body).await,
                None => {
                    api::send::<_, Poi>("POST", &format!("/maps/{}/pois", e.map_id), &body).await
                }
            };
            match r {
                Ok(_) => edit.set(None),
                Err(err) => error.set(api::message(err)),
            }
        });
    };
    let title = Signal::derive(move || {
        if edit.with(|e| e.as_ref().is_some_and(|e| e.id.is_some())) {
            "Edit the point of interest".to_string()
        } else {
            "New point of interest".to_string()
        }
    });
    view! {
        <Modal open=open title=title>
            <Stack gap="sm">
                <TextInput label="Name" value=name />
                <Textarea label="Description" value=description rows=3 />
                <div class="mimir-poi__icons" role="group" aria-label="Icon">
                    {POI_ICONS.iter().map(|(value, emoji, label)| {
                        let v = value.to_string();
                        let v2 = v.clone();
                        view! {
                            <button
                                type="button"
                                class="mimir-poi__icon"
                                title=*label
                                aria-label=*label
                                aria-pressed=move || if icon.get() == v { "true" } else { "false" }
                                on:click=move |_| icon.set(v2.clone())
                            >
                                {*emoji}
                            </button>
                        }
                    }).collect_view()}
                </div>
                <div class="mimir-poi__colors" role="group" aria-label="Colour">
                    {POI_COLORS.iter().map(|(hex, label)| {
                        let h = hex.to_string();
                        let h2 = h.clone();
                        view! {
                            <button
                                type="button"
                                class="mimir-poi__swatch"
                                style=format!("background:{hex}")
                                title=*label
                                aria-label=*label
                                aria-pressed=move || if color.get().eq_ignore_ascii_case(&h) { "true" } else { "false" }
                                on:click=move |_| color.set(h2.clone())
                            />
                        }
                    }).collect_view()}
                    <input
                        type="color"
                        aria-label="Another colour"
                        prop:value=move || color.get()
                        on:input=move |ev| color.set(event_target_value(&ev))
                    />
                </div>
                <div class="mimir-poi__preview" aria-label="Preview">
                    <span class="mimir-poi__badge" style=move || format!("background:{}", color.get())>
                        {move || poi_emoji(&icon.get())}
                    </span>
                    <Text>{move || name.get()}</Text>
                </div>
                <Switch label="Players see it" checked=visible />
                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                }}
                <Group>
                    <Button on_click=Callback::new(save)>"Save"</Button>
                    <Button variant="subtle" on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                </Group>
            </Stack>
        </Modal>
    }
}

/// What is on the map, in lists: a name selects it; a switch shows or
/// hides it (a light: on or off); × removes it.
#[component]
pub fn PlacedLists(
    #[prop(into)] tokens: Signal<Vec<Token>>,
    #[prop(into)] lights: Signal<Vec<Light>>,
    #[prop(into)] traps: Signal<Vec<Trap>>,
    #[prop(into)] pois: Signal<Vec<Poi>>,
    on_select: Callback<(String, String)>,
    error: RwSignal<String>,
) -> impl IntoView {
    // (kind, id, name, on, path, the field the switch changes)
    let row = move |kind: &'static str,
                    id: String,
                    name: String,
                    on: bool,
                    path: String,
                    field: &'static str,
                    invert: bool| {
        let shown = RwSignal::new(on);
        let (p1, p2) = (path.clone(), path);
        let (sel_kind, sel_id) = (kind.to_string(), id);
        let switch_label = if field == "active" { "Lit" } else { "Shown" };
        let n2 = name.clone();
        view! {
            <li class="mimir-placed__row">
                <button type="button" class="mimir-placed__name" on:click=move |_| on_select.run((sel_kind.clone(), sel_id.clone()))>
                    {name.clone()}
                </button>
                <Switch label=switch_label checked=shown on_change=Callback::new(move |v: bool| {
                    let body = json!({ field: if invert { !v } else { v } });
                    let p = p1.clone();
                    spawn_local(async move {
                        if let Err(e) = api::send::<_, serde_json::Value>("PATCH", &p, &body).await {
                            error.set(api::message(e));
                        }
                    });
                }) />
                <ActionIcon title=format!("Remove {n2}") on_click=Callback::new(move |_| {
                    let p = p2.clone();
                    spawn_local(async move {
                        if let Err(e) = api::delete(&p).await {
                            error.set(api::message(e));
                        }
                    });
                })>"×"</ActionIcon>
            </li>
        }
    };
    view! {
        <div class="mimir-placed" aria-label="On the map">
            <SectionLabel label="Tokens" count=Signal::derive(move || tokens.get().len()) />
            <ul>
                {move || tokens.get().into_iter().map(|t| {
                    row("token", t.id.clone(), t.name.clone(), t.visible_to_players, format!("/tokens/{}", t.id), "hidden", true)
                }).collect_view()}
            </ul>
            <SectionLabel label="Lights" count=Signal::derive(move || lights.get().len()) />
            <ul>
                {move || lights.get().into_iter().map(|l| {
                    let name = l.name.clone().unwrap_or_else(|| "Light".into());
                    row("light", l.id.clone(), name, l.active, format!("/lights/{}", l.id), "active", false)
                }).collect_view()}
            </ul>
            <SectionLabel label="Traps" count=Signal::derive(move || traps.get().len()) />
            <ul>
                {move || traps.get().into_iter().map(|t| {
                    row("trap", t.id.clone(), t.name.clone(), t.visible, format!("/traps/{}", t.id), "visible", false)
                }).collect_view()}
            </ul>
            <SectionLabel label="Points of interest" count=Signal::derive(move || pois.get().len()) />
            <ul>
                {move || pois.get().into_iter().map(|p| {
                    row("poi", p.id.clone(), p.name.clone(), p.visible, format!("/pois/{}", p.id), "visible", false)
                }).collect_view()}
            </ul>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_menu_values() {
        assert_eq!(Place::parse("pc"), Some(Place::Pc));
        assert_eq!(
            Place::parse("monster:m1"),
            Some(Place::Monster("m1".into()))
        );
        assert_eq!(Place::parse("npc:n1"), Some(Place::Npc("n1".into())));
        assert_eq!(Place::parse("torch"), Some(Place::Torch));
        assert_eq!(Place::parse("candle"), Some(Place::Candle));
        assert_eq!(Place::parse("catalog-monster"), Some(Place::CatalogMonster));
        assert_eq!(Place::parse(""), None);
    }

    #[test]
    fn what_a_kind_takes() {
        assert!(Place::Pc.is_token() && !Place::Pc.repeats());
        assert!(Place::CatalogMonster.is_token() && Place::CatalogMonster.repeats());
        assert!(!Place::Trap.is_token() && !Place::Trap.seen_at_first());
        assert!(Place::Monster("m".into()).seen_at_first());
        assert!(!Place::Candle.is_token());
    }

    #[test]
    fn poi_icons_and_colours() {
        assert_eq!(POI_ICONS.len(), 8);
        assert_eq!(POI_COLORS.len(), 8);
        assert_eq!(poi_emoji("skull"), "💀");
        assert_eq!(poi_emoji("nope"), "📍");
    }
}
