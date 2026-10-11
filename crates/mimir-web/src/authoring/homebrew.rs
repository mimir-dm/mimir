//! The campaign's Homebrew tab (MIMIR-T-0722): items, monsters, spells.
//! A list and the selected entry's preview (mimir-render); New (a
//! template), Copy from the catalog; the editor: name, the JSON with a
//! live preview, and for items a form that writes into the JSON; delete
//! with what uses the entry. The kind and the entry are in the address
//! (`?kind=items&hb=<id>`).

use aurora_leptos::components::*;
use aurora_leptos::frame::{ConfirmDialog, Modal, TabItem, Tabs};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};
use mimir_wire::{
    CatalogPage, HomebrewEntry, HomebrewPatch, HomebrewSummary, HomebrewUsage, NewHomebrew,
    HOMEBREW_KINDS,
};
use serde_json::Value;

use super::homebrew_form::{self as form, ItemForm};
use crate::api;
use crate::reference::detail::Rendered;

/// The kind of a `?kind=` value; items when absent or unknown.
pub fn kind_of(value: Option<&str>) -> &'static str {
    HOMEBREW_KINDS
        .iter()
        .map(|k| k.0)
        .find(|k| Some(*k) == value)
        .unwrap_or("items")
}

fn singular(kind: &str) -> &str {
    match kind {
        "items" => "item",
        "monsters" => "monster",
        _ => "spell",
    }
}

/// The preview of data (with its name, source HB).
pub fn preview(kind: &str, name: &str, data: &Value) -> String {
    let mut d = data.clone();
    if let Value::Object(m) = &mut d {
        m.insert("name".into(), Value::String(name.to_string()));
        m.insert("source".into(), Value::String("HB".into()));
    }
    mimir_render::detail::detail(kind, &d)
}

#[component]
pub fn HomebrewTab() -> impl IntoView {
    let params = use_params_map();
    let campaign = Memo::new(move |_| params.get().get("id").unwrap_or_default());
    let query = use_query_map();
    let kind = Memo::new(move |_| kind_of(query.get().get("kind").as_deref()));
    let selected = Memo::new(move |_| query.get().get("hb"));
    let navigate = use_navigate();
    let go = StoredValue::new(move |to: String| navigate(&to, Default::default()));

    let version = RwSignal::new(0u32);
    let list = LocalResource::new(move || {
        version.track();
        let (c, k) = (campaign.get(), kind.get());
        async move { api::get::<Vec<HomebrewSummary>>(format!("/campaigns/{c}/homebrew/{k}")).await }
    });
    let entry = LocalResource::new(move || {
        version.track();
        let (k, id) = (kind.get(), selected.get());
        async move {
            match id {
                Some(id) => api::get::<HomebrewEntry>(format!("/homebrew/{k}/{id}"))
                    .await
                    .map(Some),
                None => Ok(None),
            }
        }
    });
    let editing = RwSignal::new(None::<Option<HomebrewEntry>>); // Some(None): a new one
    let copying = RwSignal::new(false);
    let deleting = RwSignal::new(None::<(HomebrewEntry, Vec<String>)>);
    let confirm_open = RwSignal::new(false);
    Effect::new(move |_| confirm_open.set(deleting.get().is_some()));
    let error = RwSignal::new(String::new());

    let open = move |id: String| {
        go.with_value(|g| g(format!("?kind={}&hb={id}", kind.get_untracked())));
    };
    let saved = Callback::new(move |e: HomebrewEntry| {
        version.update(|v| *v += 1);
        open(e.id);
    });
    let ask_delete = move |e: HomebrewEntry| {
        let k = kind.get_untracked();
        spawn_local(async move {
            match api::get::<HomebrewUsage>(format!("/homebrew/{k}/{}/usage", e.id)).await {
                Ok(u) => deleting.set(Some((e, u.uses))),
                Err(err) => error.set(api::message(err)),
            }
        });
    };
    let remove = move |_| {
        let Some((e, _)) = deleting.get_untracked() else {
            return;
        };
        let k = kind.get_untracked();
        spawn_local(async move {
            match api::delete(&format!("/homebrew/{k}/{}", e.id)).await {
                Ok(()) => {
                    deleting.set(None);
                    version.update(|v| *v += 1);
                    go.with_value(|g| g(format!("?kind={k}")));
                }
                Err(err) => error.set(api::message(err)),
            }
        });
    };

    let tabs = HOMEBREW_KINDS
        .iter()
        .map(|(k, l)| TabItem::new(*k, *l).href(format!("?kind={k}")))
        .collect::<Vec<_>>();
    let tab = RwSignal::new(kind.get_untracked().to_string());
    Effect::new(move |_| tab.set(kind.get().to_string()));

    view! {
        <Tabs tabs=tabs value=tab label="Homebrew kinds" />
        <div class="mimir-split mimir-homebrew">
            <div class="mimir-split__side">
                <Group>
                    <Button variant="light" on_click=Callback::new(move |_| editing.set(Some(None)))>"New"</Button>
                    <Button variant="subtle" on_click=Callback::new(move |_| copying.set(true))>"Copy from the catalog"</Button>
                </Group>
                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                }}
                {move || match list.get() {
                    None => view! { <Loading /> }.into_any(),
                    Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                    Some(Ok(l)) if l.is_empty() => view! {
                        <Text size="sm" dimmed=true>{format!("No homebrew {}s yet.", singular(kind.get()))}</Text>
                    }.into_any(),
                    Some(Ok(l)) => view! {
                        <nav class="mimir-doclist" aria-label="Homebrew">
                            {l.into_iter().map(|h| {
                                let id = h.id.clone();
                                let active = move || selected.get().as_deref() == Some(id.as_str());
                                let based = h.cloned_from_name.clone().map(|n| format!("Copy of {n}"));
                                view! {
                                    <a
                                        class="cl-navlink mimir-homebrew__item"
                                        class:cl-navlink--active=active.clone()
                                        aria-current=move || active().then_some("page")
                                        href=format!("?kind={}&hb={}", kind.get_untracked(), h.id)
                                    >
                                        <span class="cl-navlink__label">{h.name.clone()}</span>
                                        <span class="mimir-homebrew__line">{[Some(h.detail.clone()).filter(|d| !d.is_empty()), based].into_iter().flatten().collect::<Vec<_>>().join(" · ")}</span>
                                    </a>
                                }
                            }).collect_view()}
                        </nav>
                    }.into_any(),
                }}
            </div>
            <div class="mimir-split__main">
                {move || match entry.get() {
                    None | Some(Ok(None)) => ().into_any(),
                    Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                    Some(Ok(Some(e))) => {
                        let html = preview(kind.get_untracked(), &e.name, &e.data);
                        let (edit, del) = (e.clone(), e.clone());
                        view! {
                            <div class="mimir-doc__head">
                                <h2 class="mimir-doc__title">{e.name.clone()}</h2>
                                <Group>
                                    <Button variant="light" on_click=Callback::new(move |_| editing.set(Some(Some(edit.clone()))))>"Edit"</Button>
                                    <ActionIcon title=format!("Delete {}", e.name) on_click=Callback::new(move |_| ask_delete(del.clone()))>"×"</ActionIcon>
                                </Group>
                            </div>
                            {e.cloned_from_name.clone().map(|n| view! {
                                <Text size="sm" dimmed=true>{format!("Copy of {n} ({})", e.cloned_from_source.clone().unwrap_or_default())}</Text>
                            })}
                            <Rendered html=html />
                        }.into_any()
                    }
                }}
            </div>
        </div>
        {move || editing.get().map(|existing| view! {
            <HomebrewEditor
                kind=kind.get_untracked()
                campaign=campaign.get_untracked()
                existing=existing
                on_saved=saved
                on_close=Callback::new(move |_| editing.set(None))
            />
        })}
        {move || copying.get().then(|| view! {
            <CopyDialog
                kind=kind.get_untracked()
                campaign=campaign.get_untracked()
                on_created=saved
                on_close=Callback::new(move |_| copying.set(false))
            />
        })}
        <ConfirmDialog
            open=confirm_open
            title=Signal::derive(move || format!("Delete {}?", deleting.get().map(|d| d.0.name).unwrap_or_default()))
            message=match kind.get_untracked() {
                "monsters" => "Modules that use it lose it too.",
                _ => "Sheets that hold it keep its name.",
            }
            impacts=Signal::derive(move || deleting.get().map(|d| d.1).unwrap_or_default())
            impacts_label="In use:"
            confirm_label="Delete"
            on_confirm=Callback::new(remove)
            on_cancel=Callback::new(move |_| deleting.set(None))
        />
    }
}

/// The item form's fields, one signal each.
#[derive(Clone, Copy)]
struct Fields {
    kind: RwSignal<String>,
    rarity: RwSignal<String>,
    weight: RwSignal<String>,
    value_gp: RwSignal<String>,
    attune: RwSignal<bool>,
    attune_text: RwSignal<String>,
    description: RwSignal<String>,
    rich: RwSignal<bool>,
    weapon_category: RwSignal<String>,
    bonus_weapon: RwSignal<String>,
    dmg1: RwSignal<String>,
    dmg_type: RwSignal<String>,
    dmg2: RwSignal<String>,
    range: RwSignal<String>,
    properties: RwSignal<Vec<String>>,
    armor_type: RwSignal<String>,
    ac: RwSignal<String>,
    bonus_ac: RwSignal<String>,
    strength: RwSignal<String>,
    stealth: RwSignal<bool>,
}

impl Fields {
    fn new() -> Self {
        let s = || RwSignal::new(String::new());
        Fields {
            kind: s(),
            rarity: s(),
            weight: s(),
            value_gp: s(),
            attune: RwSignal::new(false),
            attune_text: s(),
            description: s(),
            rich: RwSignal::new(false),
            weapon_category: s(),
            bonus_weapon: s(),
            dmg1: s(),
            dmg_type: s(),
            dmg2: s(),
            range: s(),
            properties: RwSignal::new(Vec::new()),
            armor_type: s(),
            ac: s(),
            bonus_ac: s(),
            strength: s(),
            stealth: RwSignal::new(false),
        }
    }

    fn load(&self, f: ItemForm) {
        self.kind.set(f.kind);
        self.rarity.set(f.rarity);
        self.weight.set(f.weight);
        self.value_gp.set(f.value_gp);
        self.attune.set(f.attune);
        self.attune_text.set(f.attune_text);
        self.description.set(f.description);
        self.rich.set(f.rich_entries);
        self.weapon_category.set(f.weapon_category);
        self.bonus_weapon.set(f.bonus_weapon);
        self.dmg1.set(f.dmg1);
        self.dmg_type.set(f.dmg_type);
        self.dmg2.set(f.dmg2);
        self.range.set(f.range);
        self.properties.set(f.properties);
        self.armor_type.set(f.armor_type);
        self.ac.set(f.ac);
        self.bonus_ac.set(f.bonus_ac);
        self.strength.set(f.strength);
        self.stealth.set(f.stealth);
    }

    /// Read every field (tracked).
    fn read(&self) -> ItemForm {
        ItemForm {
            kind: self.kind.get(),
            rarity: self.rarity.get(),
            weight: self.weight.get(),
            value_gp: self.value_gp.get(),
            attune: self.attune.get(),
            attune_text: self.attune_text.get(),
            description: self.description.get(),
            rich_entries: self.rich.get(),
            weapon_category: self.weapon_category.get(),
            bonus_weapon: self.bonus_weapon.get(),
            dmg1: self.dmg1.get(),
            dmg_type: self.dmg_type.get(),
            dmg2: self.dmg2.get(),
            range: self.range.get(),
            properties: self.properties.get(),
            armor_type: self.armor_type.get(),
            ac: self.ac.get(),
            bonus_ac: self.bonus_ac.get(),
            strength: self.strength.get(),
            stealth: self.stealth.get(),
        }
    }
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(k, l)| (k.to_string(), l.to_string()))
        .collect()
}

/// The editor: `existing` none for a new entry (the kind's template).
#[component]
fn HomebrewEditor(
    kind: &'static str,
    campaign: String,
    existing: Option<HomebrewEntry>,
    on_saved: Callback<HomebrewEntry>,
    on_close: Callback<()>,
) -> impl IntoView {
    let open = RwSignal::new(true);
    Effect::new(move |prev: Option<bool>| {
        let now = open.get();
        if prev == Some(true) && !now {
            on_close.run(());
        }
        now
    });
    let is_new = existing.is_none();
    let id = StoredValue::new(existing.as_ref().map(|e| e.id.clone()));
    let campaign = StoredValue::new(campaign);
    let name = RwSignal::new(
        existing
            .as_ref()
            .map(|e| e.name.clone())
            .unwrap_or_default(),
    );
    let start = existing
        .as_ref()
        .map(|e| {
            let mut d = e.data.clone();
            if let Value::Object(m) = &mut d {
                m.remove("name");
                m.remove("source");
            }
            d
        })
        .unwrap_or_else(|| form::template(kind));
    let json_text = RwSignal::new(serde_json::to_string_pretty(&start).unwrap_or_default());
    let parsed = Memo::new(move |_| {
        serde_json::from_str::<Value>(&json_text.get()).map_err(|e| e.to_string())
    });
    // Items: the form, or the JSON.
    let mode = RwSignal::new(if kind == "items" { "form" } else { "json" }.to_string());
    let fields = Fields::new();
    fields.load(form::form_of(&start));
    // Back to the form: read the JSON again.
    Effect::new(move |prev: Option<String>| {
        let m = mode.get();
        if prev.as_deref() == Some("json") && m == "form" {
            if let Ok(v) = parsed.get_untracked() {
                fields.load(form::form_of(&v));
            }
        }
        m
    });
    // On the form: each change writes into the JSON.
    Effect::new(move |_| {
        let f = fields.read();
        if mode.get_untracked() != "form" {
            return;
        }
        let mut v = parsed
            .get_untracked()
            .unwrap_or_else(|_| serde_json::json!({}));
        form::apply(&f, &mut v);
        let text = serde_json::to_string_pretty(&v).unwrap_or_default();
        if text != json_text.get_untracked() {
            json_text.set(text);
        }
    });
    let preview_html = Signal::derive(move || match parsed.get() {
        Ok(v) => preview(kind, &name.get(), &v),
        Err(_) => String::new(),
    });
    let error = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let save = move |_| {
        let Ok(data) = parsed.get_untracked() else {
            return;
        };
        busy.set(true);
        let n = name.get_untracked();
        spawn_local(async move {
            let r = match id.get_value() {
                None => {
                    let body = NewHomebrew {
                        name: n,
                        data: Some(data),
                        ..NewHomebrew::default()
                    };
                    let path = format!("/campaigns/{}/homebrew/{kind}", campaign.get_value());
                    api::send::<_, HomebrewEntry>("POST", &path, &body).await
                }
                Some(id) => {
                    let body = HomebrewPatch {
                        name: Some(n),
                        data: Some(data),
                    };
                    api::send::<_, HomebrewEntry>("PATCH", &format!("/homebrew/{kind}/{id}"), &body)
                        .await
                }
            };
            busy.set(false);
            match r {
                Ok(e) => {
                    on_saved.run(e);
                    open.set(false);
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };

    let item_form = move || {
        let f = fields;
        view! {
            <div class="mimir-homebrew__form">
                <Select label="Kind" option_pairs=pairs(&form::KINDS) value=f.kind />
                <Select label="Rarity" option_pairs=pairs(&form::RARITIES) value=f.rarity />
                <TextInput label="Weight (lb.)" value=f.weight input_type="number" />
                <TextInput label="Value (gp)" value=f.value_gp input_type="number" />
                <Switch label="Requires attunement" checked=f.attune />
                {move || f.attune.get().then(|| view! { <TextInput label="Attunement by (optional)" value=f.attune_text placeholder="by a wizard" /> })}
                {move || (f.kind.get() == "weapon").then(|| view! {
                    <fieldset class="mimir-homebrew__group">
                        <legend>"Weapon"</legend>
                        <Select label="Category" option_pairs=pairs(&[("simple", "Simple"), ("martial", "Martial")]) value=f.weapon_category />
                        <TextInput label="Damage" value=f.dmg1 placeholder="1d8" />
                        <Select label="Damage type" placeholder="—" option_pairs=pairs(&form::DAMAGE) value=f.dmg_type />
                        <TextInput label="Versatile damage" value=f.dmg2 placeholder="1d10" />
                        <TextInput label="Range" value=f.range placeholder="80/320" />
                        <TextInput label="Weapon bonus" value=f.bonus_weapon placeholder="+1" />
                        <div class="mimir-wizard__skills" role="group" aria-label="Properties">
                            {form::PROPERTIES.iter().map(|(code, label)| {
                                let code = code.to_string();
                                let c2 = code.clone();
                                view! {
                                    <label class="mimir-wizard__skill">
                                        <input
                                            type="checkbox"
                                            checked=move || f.properties.get().contains(&code)
                                            on:change=move |_| f.properties.update(|p| {
                                                if let Some(i) = p.iter().position(|x| x == &c2) { p.remove(i); } else { p.push(c2.clone()); }
                                            })
                                        />
                                        <span>{*label}</span>
                                    </label>
                                }
                            }).collect_view()}
                        </div>
                    </fieldset>
                })}
                {move || (f.kind.get() == "armor").then(|| view! {
                    <fieldset class="mimir-homebrew__group">
                        <legend>"Armor"</legend>
                        <Select label="Armor" option_pairs=pairs(&form::ARMOR) value=f.armor_type />
                        <TextInput label="Armor class" value=f.ac input_type="number" />
                        <TextInput label="AC bonus" value=f.bonus_ac placeholder="+1" />
                        <TextInput label="Strength needed" value=f.strength />
                        <Switch label="Disadvantage on Stealth" checked=f.stealth />
                    </fieldset>
                })}
                {move || if f.rich.get() {
                    view! { <Text size="sm" dimmed=true>"Its text has lists or tables: edit it in JSON."</Text> }.into_any()
                } else {
                    view! { <Textarea label="Description (a blank line between paragraphs)" value=f.description /> }.into_any()
                }}
            </div>
        }
    };

    let modes = vec![TabItem::new("form", "Form"), TabItem::new("json", "JSON")];
    view! {
        <Modal open=open title=if is_new { format!("New homebrew {}", singular(kind)) } else { "Edit".to_string() } size="xl" locked=busy>
            <div class="mimir-homebrew__editor">
                <div class="mimir-homebrew__inputs">
                    <TextInput label="Name" value=name />
                    {(kind == "items").then(|| view! { <Tabs tabs=modes.clone() value=mode label="Edit as" /> })}
                    {move || if mode.get() == "form" {
                        item_form().into_any()
                    } else {
                        view! {
                            <label class="mimir-file">
                                <span>"Data (5etools JSON)"</span>
                                <textarea
                                    class="mimir-homebrew__json"
                                    aria-label="Data (JSON)"
                                    spellcheck="false"
                                    prop:value=move || json_text.get()
                                    on:input=move |ev| json_text.set(event_target_value(&ev))
                                ></textarea>
                            </label>
                            {move || parsed.get().err().map(|e| view! { <Alert color=token::BAD>{format!("Not valid JSON: {e}")}</Alert> })}
                        }.into_any()
                    }}
                </div>
                <div class="mimir-homebrew__preview" aria-label="Preview">
                    <Rendered html=preview_html />
                </div>
            </div>
            {move || {
                let e = error.get();
                (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
            }}
            <Group>
                <Button
                    disabled=Signal::derive(move || busy.get() || name.get().trim().is_empty() || parsed.get().is_err())
                    on_click=Callback::new(save)
                >"Save"</Button>
                <Button variant="subtle" on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
            </Group>
        </Modal>
    }
}

/// Copy a catalog entry: search, pick, name the copy.
#[component]
fn CopyDialog(
    kind: &'static str,
    campaign: String,
    on_created: Callback<HomebrewEntry>,
    on_close: Callback<()>,
) -> impl IntoView {
    let open = RwSignal::new(true);
    Effect::new(move |prev: Option<bool>| {
        let now = open.get();
        if prev == Some(true) && !now {
            on_close.run(());
        }
        now
    });
    let campaign = StoredValue::new(campaign);
    let search = RwSignal::new(String::new());
    let results = RwSignal::new(None::<Result<CatalogPage, String>>);
    let picked = RwSignal::new(None::<(String, String)>);
    let name = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());
    let find = move || {
        let q = search.get_untracked();
        spawn_local(async move {
            let path = format!(
                "/catalog/search/{kind}?limit=20&search={}",
                js_sys::encode_uri_component(q.trim())
            );
            results.set(Some(
                api::get::<CatalogPage>(path).await.map_err(api::message),
            ));
        });
    };
    let create = move |_| {
        let Some((n, src)) = picked.get_untracked() else {
            return;
        };
        let body = NewHomebrew {
            name: name.get_untracked(),
            data: None,
            cloned_from_name: Some(n),
            cloned_from_source: Some(src),
        };
        spawn_local(async move {
            let path = format!("/campaigns/{}/homebrew/{kind}", campaign.get_value());
            match api::send::<_, HomebrewEntry>("POST", &path, &body).await {
                Ok(e) => {
                    on_created.run(e);
                    open.set(false);
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    view! {
        <Modal open=open title=format!("Copy a {} from the catalog", singular(kind)) size="lg">
            <Stack gap="sm">
                <form class="mimir-inv__search" on:submit=move |ev| { ev.prevent_default(); find(); }>
                    <TextInput label="Catalog name" value=search />
                    <Button variant="light" button_type="submit">"Search"</Button>
                </form>
                {move || results.get().map(|r| match r {
                    Err(e) => view! { <Alert color=token::BAD>{e}</Alert> }.into_any(),
                    Ok(p) if p.rows.is_empty() => view! { <Text size="sm" dimmed=true>"Nothing found."</Text> }.into_any(),
                    Ok(p) => view! {
                        <ul class="mimir-inv__results" aria-label="Catalog entries">
                            {p.rows.into_iter().map(|row| {
                                let (n, s) = (row.name.clone(), row.source.clone());
                                let here = {
                                    let (n, s) = (n.clone(), s.clone());
                                    move || picked.get() == Some((n.clone(), s.clone()))
                                };
                                view! {
                                    <li class:mimir-homebrew__picked=here>
                                        <div class="mimir-inv__name">
                                            <Text bold=true>{row.name.clone()}</Text>
                                            <Text size="xs" dimmed=true>{format!("{} · {}", row.source, row.cells.join(" · "))}</Text>
                                        </div>
                                        <Button variant="light" on_click=Callback::new(move |_| {
                                            name.set(n.clone());
                                            picked.set(Some((n.clone(), s.clone())));
                                        })>"Pick"</Button>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    }.into_any(),
                })}
                {move || picked.get().map(|(n, s)| view! {
                    <Text size="sm">{format!("Copy of {n} ({s}). Name the copy:")}</Text>
                    <TextInput label="Name" value=name />
                })}
                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                }}
                <Group>
                    <Button
                        disabled=Signal::derive(move || picked.get().is_none() || name.get().trim().is_empty())
                        on_click=Callback::new(create)
                    >"Copy"</Button>
                    <Button variant="subtle" on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                </Group>
            </Stack>
        </Modal>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn kinds() {
        assert_eq!(kind_of(None), "items");
        assert_eq!(kind_of(Some("spells")), "spells");
        assert_eq!(kind_of(Some("vehicles")), "items");
    }

    #[test]
    fn previews_carry_the_name() {
        let html = preview("spells", "Ember Lance", &json!({"level": 3, "school": "V"}));
        assert!(html.contains("3rd-level evocation") && html.contains("Source: HB"));
    }
}
