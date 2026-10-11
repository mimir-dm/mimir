//! The initiative tracker beside the DM map (MIMIR-T-0715): the module's
//! combat in turn order, HP, conditions, concentration, turns, adding
//! combatants, token links and the turn order on the player display. A
//! port of the desktop `InitiativeTracker`; its rules live on the server
//! (CombatService), so this view only sends changes and shows the answer.

use aurora_leptos::components::*;
use aurora_leptos::tokens::{token, ApiError};
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_wire::{
    CharacterSummary, Combat, CombatEntry, Condition, DamageResult, DisplayState, DisplayUpdate,
    ModuleMonsterSummary, ModuleNpcSummary,
};
use serde_json::json;

use crate::api;

/// The SRD conditions (the server takes these names).
pub const CONDITIONS: [&str; 15] = [
    "blinded",
    "charmed",
    "deafened",
    "exhaustion",
    "frightened",
    "grappled",
    "incapacitated",
    "invisible",
    "paralyzed",
    "petrified",
    "poisoned",
    "prone",
    "restrained",
    "stunned",
    "unconscious",
];

// ---- Pure helpers (host-tested) ----------------------------------------------

/// "cur/max", with " +temp" when there is temp HP; "—" with no HP.
pub fn hp_text(e: &CombatEntry) -> String {
    match (e.current_hp, e.max_hp) {
        (Some(c), Some(m)) if e.temp_hp > 0 => format!("{c}/{m} +{}", e.temp_hp),
        (Some(c), Some(m)) => format!("{c}/{m}"),
        _ => "—".to_string(),
    }
}

/// The HP bar: percent of max (0–100).
pub fn hp_percent(e: &CombatEntry) -> f64 {
    match (e.current_hp, e.max_hp) {
        (Some(c), Some(m)) if m > 0 => (f64::from(c.max(0)) / f64::from(m) * 100.0).round(),
        _ => 0.0,
    }
}

/// "healthy" above half, "bloodied" above 0, else "down".
pub fn hp_band(e: &CombatEntry) -> &'static str {
    let p = hp_percent(e);
    if p > 50.0 {
        "healthy"
    } else if p > 0.0 {
        "bloodied"
    } else {
        "down"
    }
}

/// Down: at 0 HP.
pub fn is_down(e: &CombatEntry) -> bool {
    e.current_hp == Some(0)
}

/// The title of a condition pill.
pub fn condition_title(c: &Condition) -> String {
    match c.expires_round {
        Some(r) => format!("{} until the end of round {r}", c.name),
        None => format!("{} until removed", c.name),
    }
}

/// A whole number of at least `min` from a field, else `None`.
pub fn positive_int(text: &str, min: i32) -> Option<i32> {
    text.trim().parse::<i32>().ok().filter(|n| *n >= min)
}

/// What the map marks: the token of the current turn, and the tokens of
/// entries at 0 HP.
pub fn map_marks(c: Option<&Combat>) -> (Option<String>, Vec<String>) {
    let Some(c) = c else {
        return (None, Vec::new());
    };
    let current = c
        .entries
        .iter()
        .find(|e| Some(&e.id) == c.current_entry_id.as_ref())
        .and_then(|e| e.token_id.clone());
    let down = c
        .entries
        .iter()
        .filter(|e| is_down(e))
        .filter_map(|e| e.token_id.clone())
        .collect();
    (current, down)
}

/// The key of a token-link request: the map and the unlinked monster and
/// NPC entries. Each key is asked for once. `None`: nothing to link.
pub fn link_key(c: &Combat, map_id: &str) -> Option<String> {
    let mut ids: Vec<&str> = c
        .entries
        .iter()
        .filter(|e| {
            e.token_id.is_none()
                && matches!(e.source_kind.as_str(), "module_monster" | "module_npc")
        })
        .map(|e| e.id.as_str())
        .collect();
    if ids.is_empty() || map_id.is_empty() {
        return None;
    }
    ids.sort_unstable();
    Some(format!("{map_id}|{}", ids.join(",")))
}

/// The value of the add menu: "monster:<id>", "npc:<id>" or "pc:<id>" →
/// the request body.
pub fn add_body(value: &str) -> Option<serde_json::Value> {
    let (kind, id) = value.split_once(':')?;
    match kind {
        "monster" => Some(json!({"kind": "monster", "module_monster_id": id})),
        "npc" => Some(json!({"kind": "npc", "npc_id": id})),
        "pc" => Some(json!({"kind": "character", "character_id": id})),
        _ => None,
    }
}

// ---- The component -------------------------------------------------------------

/// A boxed request whose error the tracker shows.
type Job = std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ApiError>>>>;

const COLLAPSE_KEY: &str = "mimir.initiativeTracker.collapsed";

fn stored_collapsed() -> bool {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(COLLAPSE_KEY).ok().flatten())
        .is_some_and(|v| v == "1")
}

fn store_collapsed(on: bool) {
    if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = s.set_item(COLLAPSE_KEY, if on { "1" } else { "0" });
    }
}

/// The tracker. `combat` is owned by the map page (it marks tokens from
/// it); `combat_v` bumps when the socket says the combat changed.
#[component]
pub fn CombatTracker(
    module_id: Memo<Option<String>>,
    campaign_id: Memo<String>,
    map_id: Memo<String>,
    combat: RwSignal<Option<Combat>>,
    combat_v: RwSignal<u32>,
    display: RwSignal<DisplayState>,
    selected_token: RwSignal<Option<String>>,
) -> impl IntoView {
    let error = RwSignal::new(String::new());
    let collapsed = RwSignal::new(stored_collapsed());
    let open = RwSignal::new(None::<String>);
    let prompt = RwSignal::new(None::<(String, i32)>);
    let confirm_end = RwSignal::new(false);
    let last_link = StoredValue::new(String::new());

    // Read the combat when the module is known and on each notice.
    Effect::new(move |_| {
        combat_v.track();
        let Some(m) = module_id.get() else { return };
        spawn_local(async move {
            match api::get::<Option<Combat>>(format!("/modules/{m}/combat")).await {
                Ok(c) => combat.set(c),
                Err(e) => error.set(message(e)),
            }
        });
    });

    let run = move |job: Job| {
        spawn_local(async move {
            match job.await {
                Ok(()) => error.set(String::new()),
                Err(e) => error.set(message(e)),
            }
        });
    };
    // A request that answers the whole combat ("DELETE" sends no body).
    let whole = move |method: &'static str, path: String, body: serde_json::Value| {
        run(Box::pin(async move {
            let c: Combat = if method == "DELETE" {
                api::delete_json(&path).await?
            } else {
                api::send(method, &path, &body).await?
            };
            combat.set(Some(c));
            Ok(())
        }));
    };
    // A request that answers one entry: patch it in place.
    let one = move |path: String, body: serde_json::Value| {
        run(Box::pin(async move {
            let e: CombatEntry = api::send("POST", &path, &body).await?;
            combat.update(|c| {
                if let Some(c) = c {
                    if let Some(slot) = c.entries.iter_mut().find(|x| x.id == e.id) {
                        *slot = e;
                    }
                }
            });
            Ok(())
        }));
    };

    // Link unlinked monsters and NPCs to the map's tokens, once per set.
    Effect::new(move |_| {
        let (Some(c), map) = (combat.get(), map_id.get()) else {
            return;
        };
        let Some(key) = link_key(&c, &map) else {
            return;
        };
        if last_link.get_value() == key {
            return;
        }
        last_link.set_value(key);
        whole(
            "POST",
            format!("/combat/{}/link-tokens", c.session.id),
            json!({ "map_id": map }),
        );
    });

    // Choices to add.
    let monsters = LocalResource::new(move || {
        let m = module_id.get();
        async move {
            match m {
                Some(m) => api::module_list::<ModuleMonsterSummary>(m, "monsters")
                    .await
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        }
    });
    let npcs = LocalResource::new(move || {
        let m = module_id.get();
        async move {
            match m {
                Some(m) => api::module_list::<ModuleNpcSummary>(m, "npcs")
                    .await
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        }
    });
    let pcs = LocalResource::new(move || {
        let c = campaign_id.get();
        async move {
            if c.is_empty() {
                return Vec::new();
            }
            api::campaign_list::<CharacterSummary>(c, "pcs")
                .await
                .unwrap_or_default()
        }
    });
    let add_choice = RwSignal::new(String::new());
    let custom = RwSignal::new(String::new());
    let add_options = move || {
        let mut o: Vec<(String, String)> = Vec::new();
        if let Some(list) = monsters.get() {
            o.extend(list.into_iter().map(|m| {
                (
                    format!("monster:{}", m.id),
                    format!("{} ×{}", m.name, m.quantity),
                )
            }));
        }
        if let Some(list) = npcs.get() {
            o.extend(
                list.into_iter()
                    .map(|n| (format!("npc:{}", n.id), format!("NPC: {}", n.name))),
            );
        }
        if let Some(list) = pcs.get() {
            o.extend(
                list.into_iter()
                    .map(|p| (format!("pc:{}", p.id), format!("PC: {}", p.name))),
            );
        }
        o
    };

    let session_id = move || combat.with(|c| c.as_ref().map(|c| c.session.id.clone()));
    let start = move |_| {
        if let Some(m) = module_id.get_untracked() {
            whole("POST", format!("/modules/{m}/combat"), json!({}));
        }
    };
    let turn = move |dir: &'static str| {
        if let Some(id) = session_id() {
            whole("POST", format!("/combat/{id}/{dir}"), json!({}));
        }
    };
    let add = move |_| {
        let (Some(id), Some(body)) = (session_id(), add_body(&add_choice.get_untracked())) else {
            return;
        };
        add_choice.set(String::new());
        whole("POST", format!("/combat/{id}/entries"), body);
    };
    let add_custom = move || {
        let name = custom.get_untracked().trim().to_string();
        let Some(id) = session_id() else { return };
        if name.is_empty() {
            return;
        }
        custom.set(String::new());
        whole(
            "POST",
            format!("/combat/{id}/entries"),
            json!({"kind": "custom", "name": name}),
        );
    };
    let show_order = RwSignal::new(false);
    Effect::new(move |_| show_order.set(display.with(|d| d.show_initiative)));
    let set_show = Callback::new(move |on: bool| {
        let d = display.get_untracked();
        let c = campaign_id.get_untracked();
        run(Box::pin(async move {
            let d: DisplayState = api::send(
                "PUT",
                &format!("/campaigns/{c}/display"),
                &DisplayUpdate {
                    map_id: d.map_id,
                    blackout: d.blackout,
                    show_initiative: on,
                },
            )
            .await?;
            display.set(d);
            Ok(())
        }));
    });
    let end = move |_| {
        confirm_end.set(false);
        let Some(id) = session_id() else { return };
        run(Box::pin(async move {
            api::delete(&format!("/combat/{id}")).await?;
            combat.set(None);
            Ok(())
        }));
    };

    let row = move |e: CombatEntry| {
        let id = e.id.clone();
        let id_cur = id.clone();
        let current = move || {
            combat.with(|c| {
                c.as_ref().and_then(|c| c.current_entry_id.as_deref()) == Some(id_cur.as_str())
            })
        };
        let current2 = current.clone();
        let token = e.token_id.clone();
        let init = RwSignal::new(e.initiative.map(|i| i.to_string()).unwrap_or_default());
        let id_init = id.clone();
        let id_toggle = id.clone();
        let id_open = id.clone();
        let id_remove = id.clone();
        let token_sel = token.clone();
        let down = is_down(&e);
        let conditions = e.conditions.clone();
        view! {
            <li
                class="mimir-tracker__row"
                class:mimir-tracker__row--current=current
                class:mimir-tracker__row--down=down
                class:mimir-tracker__row--selected=move || token_sel.is_some() && selected_token.get() == token_sel
                aria-current=move || current2().then_some("step")
            >
                <div class="mimir-tracker__line">
                    <input
                        class="mimir-tracker__init"
                        type="number"
                        aria-label=format!("Initiative of {}", e.name)
                        prop:value=move || init.get()
                        on:input=move |ev| init.set(event_target_value(&ev))
                        on:change=move |_| {
                            let v = init.get_untracked().trim().parse::<i32>().ok();
                            whole("PATCH", format!("/combat-entries/{id_init}"), json!({ "initiative": v }));
                        }
                    />
                    <button
                        type="button"
                        class="mimir-tracker__name"
                        aria-expanded=move || (open.get().as_deref() == Some(id_open.as_str())).to_string()
                        on:click=move |_| {
                            let opening = open.get_untracked().as_deref() != Some(id_toggle.as_str());
                            open.set(opening.then(|| id_toggle.clone()));
                            if token.is_some() {
                                selected_token.set(if opening { token.clone() } else { None });
                            }
                        }
                    >
                        {e.name.clone()}
                        {e.token_id.is_some().then(|| view! { <span class="mimir-tracker__linked" title="On the map">" ⌖"</span> })}
                    </button>
                    <span class="mimir-tracker__hp">{hp_text(&e)}</span>
                    <ActionIcon
                        title=format!("Remove {}", e.name)
                        on_click=Callback::new(move |_| whole("DELETE", format!("/combat-entries/{id_remove}"), json!({})))
                    >
                        "×"
                    </ActionIcon>
                </div>
                {(e.concentrating || !conditions.is_empty()).then(|| {
                    let cid = e.id.clone();
                    view! {
                        <div class="mimir-tracker__badges">
                            {e.concentrating.then(|| view! { <Pill color=token::VIOLET>"Conc."</Pill> })}
                            {conditions
                                .into_iter()
                                .map(|c| {
                                    let path = format!("/combat-entries/{}/conditions/{}", cid, c.name);
                                    view! {
                                        <button
                                            type="button"
                                            class="mimir-tracker__condition"
                                            title=condition_title(&c)
                                            on:click=move |_| {
                                                let path = path.clone();
                                                run(Box::pin(async move {
                                                    let e: CombatEntry = api::delete_json(&path).await?;
                                                    combat.update(|c| {
                                                        if let Some(c) = c {
                                                            if let Some(s) = c.entries.iter_mut().find(|x| x.id == e.id) {
                                                                *s = e;
                                                            }
                                                        }
                                                    });
                                                    Ok(())
                                                }));
                                            }
                                        >
                                            {c.name.clone()}
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                })}
                {move || (open.get().as_deref() == Some(id.as_str())).then(|| details(e.clone(), whole, one, run, prompt, combat))}
            </li>
        }
    };

    // The body rebuilds only when a combat starts or ends; the rows are a
    // keyed list, so a change rebuilds only the rows that changed (an
    // initiative field being typed in stays).
    let running = Memo::new(move |_| combat.with(Option::is_some));
    let rows = move || {
        combat.with(|c| {
            let Some(c) = c else { return Vec::new() };
            // Keyed by the entry's data: a row is rebuilt only when its
            // own entry changes (the current turn is a reactive class).
            c.entries
                .iter()
                .map(|e| (format!("{}|{:?}", e.id, e), e.clone()))
                .collect::<Vec<_>>()
        })
    };
    let empty = move || combat.with(|c| c.as_ref().is_some_and(|c| c.entries.is_empty()));
    let body = move || {
        if !running.get() {
            return view! {
                <Button on_click=Callback::new(start)>"Start combat"</Button>
            }
            .into_any();
        }
        view! {
            {move || prompt.get().map(|(eid, dc)| {
                let eid2 = eid;
                view! {
                    <div class="mimir-tracker__prompt">
                        <Alert color=token::VIOLET title=format!("Concentration save DC {dc}")>
                            <Group>
                                <Button variant="light" on_click=Callback::new(move |_| prompt.set(None))>"Kept"</Button>
                                <Button variant="light" bad=true on_click=Callback::new(move |_| {
                                    prompt.set(None);
                                    whole("PATCH", format!("/combat-entries/{eid2}"), json!({ "concentrating": false }));
                                })>"Lost"</Button>
                            </Group>
                        </Alert>
                    </div>
                }
            })}
            <ol class="mimir-tracker__list" aria-label="Turn order">
                {move || empty().then(|| view! {
                    <li class="mimir-tracker__empty"><Text size="sm" dimmed=true>"Add creatures below."</Text></li>
                })}
                <For each=rows key=|(key, _)| key.clone() children=move |(_, e)| row(e) />
            </ol>
            <Group>
                <Button variant="light" on_click=Callback::new(move |_| turn("previous"))>"‹ Prev"</Button>
                <Button on_click=Callback::new(move |_| turn("next"))>"Next turn ›"</Button>
            </Group>
            <div class="mimir-tracker__add">
                <Select aria_label="Add to the combat" placeholder="Add a creature…" option_pairs=add_options() value=add_choice />
                <Button variant="light" on_click=Callback::new(add)>"Add"</Button>
            </div>
            <form class="mimir-tracker__add" on:submit=move |ev| {
                ev.prevent_default();
                add_custom();
            }>
                <TextInput placeholder="Custom name" value=custom name="custom" />
                <Button variant="light" button_type="submit">"Add"</Button>
            </form>
            <Switch label="Show order to players" checked=show_order on_change=set_show />
            {move || if confirm_end.get() {
                view! {
                    <Group>
                        <Text size="sm">"End this combat?"</Text>
                        <Button bad=true on_click=Callback::new(end)>"End"</Button>
                        <Button variant="subtle" on_click=Callback::new(move |_| confirm_end.set(false))>"Cancel"</Button>
                    </Group>
                }.into_any()
            } else {
                view! { <Button variant="subtle" bad=true on_click=Callback::new(move |_| confirm_end.set(true))>"End combat"</Button> }.into_any()
            }}
        }
            .into_any()
    };

    view! {
        <aside class="mimir-tracker" class:mimir-tracker--collapsed=move || collapsed.get() aria-label="Initiative">
            <div class="mimir-tracker__head">
                <Text bold=true>"Initiative"</Text>
                {move || combat.with(|c| c.as_ref().map(|c| {
                    let r = c.session.round;
                    view! { <Pill color=token::ICE>{format!("Round {r}")}</Pill> }
                }))}
                <ActionIcon
                    title="Show or hide the tracker"
                    on_click=Callback::new(move |_| {
                        let next = !collapsed.get_untracked();
                        collapsed.set(next);
                        store_collapsed(next);
                    })
                >
                    {move || if collapsed.get() { "‹" } else { "›" }}
                </ActionIcon>
            </div>
            {move || (!collapsed.get()).then(|| view! {
                <div class="mimir-tracker__body">
                    {move || {
                        let e = error.get();
                        (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                    }}
                    {body}
                </div>
            })}
        </aside>
    }
}

fn message(e: ApiError) -> String {
    match e {
        ApiError::Http { message, .. } => message,
        ApiError::Network => "The server did not answer.".into(),
        ApiError::Unknown(m) => m,
    }
}

/// The details of an open entry: HP bar and changes, temp HP, max HP,
/// conditions, concentration and the HP log.
fn details(
    e: CombatEntry,
    whole: impl Fn(&'static str, String, serde_json::Value) + Copy + Send + Sync + 'static,
    one: impl Fn(String, serde_json::Value) + Copy + Send + Sync + 'static,
    run: impl Fn(Job) + Copy + Send + Sync + 'static,
    prompt: RwSignal<Option<(String, i32)>>,
    combat: RwSignal<Option<Combat>>,
) -> impl IntoView {
    let id = e.id.clone();
    let amount = RwSignal::new(String::new());
    let temp = RwSignal::new(e.temp_hp.to_string());
    let max = RwSignal::new(String::new());
    let condition = RwSignal::new(String::new());
    let rounds = RwSignal::new(String::new());
    let concentrating = RwSignal::new(e.concentrating);
    let valid = Signal::derive(move || positive_int(&amount.get(), 0).is_none());
    let (id1, id2, id3, id4, id5, id6) = (
        id.clone(),
        id.clone(),
        id.clone(),
        id.clone(),
        id.clone(),
        id,
    );
    let damage = move || {
        let Some(n) = positive_int(&amount.get_untracked(), 0) else {
            return;
        };
        let path = format!("/combat-entries/{id1}/damage");
        let eid = id1.clone();
        amount.set(String::new());
        run(Box::pin(async move {
            let r: DamageResult = api::send("POST", &path, &json!({ "amount": n })).await?;
            if let Some(dc) = r.concentration_dc {
                prompt.set(Some((eid, dc)));
            }
            combat.update(|c| {
                if let Some(c) = c {
                    if let Some(s) = c.entries.iter_mut().find(|x| x.id == r.entry.id) {
                        *s = r.entry;
                    }
                }
            });
            Ok(())
        }));
    };
    let has_hp = e.current_hp.is_some();
    let percent = hp_percent(&e);
    let band = hp_band(&e);
    let log = e.hp_log.clone();
    view! {
        <div class="mimir-tracker__details">
            {has_hp.then(|| view! {
                <div class=format!("mimir-tracker__bar mimir-tracker__bar--{band}") role="meter"
                    aria-valuenow=percent aria-valuemin="0" aria-valuemax="100" aria-label="HP">
                    <span style=format!("width:{percent}%") />
                </div>
            })}
            {has_hp.then(|| view! {
                <form class="mimir-tracker__add" on:submit=move |ev| {
                    ev.prevent_default();
                    damage();
                }>
                    <TextInput aria_label="Amount" placeholder="Amount" value=amount input_type="number" name="amount" />
                    <Button bad=true button_type="submit" disabled=valid>"Damage"</Button>
                    <Button variant="light" disabled=valid on_click=Callback::new(move |_| {
                        let Some(n) = positive_int(&amount.get_untracked(), 0) else { return };
                        amount.set(String::new());
                        one(format!("/combat-entries/{id2}/heal"), json!({ "amount": n }));
                    })>"Heal"</Button>
                </form>
            })}
            {has_hp.then(|| view! {
                <TextInput label="Temp HP" value=temp input_type="number"
                    on_change=Callback::new(move |v: String| {
                        let n = positive_int(&v, 0).unwrap_or(0);
                        whole("PATCH", format!("/combat-entries/{id3}"), json!({ "temp_hp": n }));
                    }) />
            })}
            {(!has_hp).then(|| view! {
                <form class="mimir-tracker__add" on:submit=move |ev| {
                    ev.prevent_default();
                    if let Some(n) = positive_int(&max.get_untracked(), 1) {
                        whole("PATCH", format!("/combat-entries/{id4}"), json!({ "max_hp": n }));
                    }
                }>
                    <TextInput aria_label="Max HP" placeholder="Set HP" value=max input_type="number" name="max" />
                    <Button variant="light" button_type="submit">"Set HP"</Button>
                </form>
            })}
            <form class="mimir-tracker__add" on:submit=move |ev| {
                ev.prevent_default();
                let name = condition.get_untracked();
                if name.is_empty() {
                    return;
                }
                let rounds_n = positive_int(&rounds.get_untracked(), 1);
                condition.set(String::new());
                rounds.set(String::new());
                one(format!("/combat-entries/{id5}/conditions"), json!({ "name": name, "duration_rounds": rounds_n }));
            }>
                <Select aria_label="Condition" placeholder="Condition…" options=CONDITIONS.iter().map(|c| c.to_string()).collect() value=condition />
                <TextInput aria_label="Rounds" placeholder="Rnds" value=rounds input_type="number" name="rounds" />
                <Button variant="light" button_type="submit">"Add"</Button>
            </form>
            <Switch label="Concentrating" checked=concentrating on_change=Callback::new(move |on: bool| {
                whole("PATCH", format!("/combat-entries/{id6}"), json!({ "concentrating": on }));
            }) />
            {(!log.is_empty()).then(|| view! {
                <ul class="mimir-tracker__log" aria-label="Recent HP changes">
                    {log.into_iter().map(|h| {
                        let sign = if h.kind == "damage" { "−" } else { "+" };
                        view! { <li>{format!("{sign}{} (round {})", h.amount, h.round)}</li> }
                    }).collect_view()}
                </ul>
            })}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mimir_wire::{CombatSession, HpChange};

    fn entry(id: &str, kind: &str) -> CombatEntry {
        CombatEntry {
            id: id.into(),
            source_kind: kind.into(),
            source_id: None,
            token_id: None,
            name: id.into(),
            initiative: None,
            dex_modifier: None,
            max_hp: Some(20),
            current_hp: Some(20),
            temp_hp: 0,
            concentrating: false,
            conditions: vec![],
            hp_log: vec![],
        }
    }

    fn combat(entries: Vec<CombatEntry>, current: Option<&str>) -> Combat {
        Combat {
            session: CombatSession {
                id: "s1".into(),
                module_id: "m1".into(),
                round: 2,
                turn_index: 0,
                status: "active".into(),
            },
            entries,
            current_entry_id: current.map(String::from),
        }
    }

    #[test]
    fn hp_text_bar_and_band() {
        let mut e = entry("a", "custom");
        assert_eq!(hp_text(&e), "20/20");
        e.temp_hp = 5;
        e.current_hp = Some(8);
        assert_eq!(hp_text(&e), "8/20 +5");
        assert_eq!(hp_percent(&e), 40.0);
        assert_eq!(hp_band(&e), "bloodied");
        e.current_hp = Some(15);
        assert_eq!(hp_band(&e), "healthy");
        e.current_hp = Some(0);
        assert_eq!(hp_band(&e), "down");
        assert!(is_down(&e));
        e.current_hp = None;
        e.max_hp = None;
        assert_eq!(hp_text(&e), "—");
        assert_eq!(hp_percent(&e), 0.0);
    }

    #[test]
    fn condition_titles_and_amounts() {
        let c = Condition {
            name: "poisoned".into(),
            expires_round: Some(3),
        };
        assert_eq!(condition_title(&c), "poisoned until the end of round 3");
        let c = Condition {
            name: "prone".into(),
            expires_round: None,
        };
        assert_eq!(condition_title(&c), "prone until removed");
        assert_eq!(positive_int(" 7 ", 0), Some(7));
        assert_eq!(positive_int("0", 1), None);
        assert_eq!(positive_int("-2", 0), None);
        assert_eq!(positive_int("x", 0), None);
        assert_eq!(positive_int("", 0), None);
    }

    // Ported from the desktop mapLink tests (combatMapState).
    #[test]
    fn the_map_marks_the_current_turn_and_the_down_tokens() {
        let mut a = entry("a", "module_monster");
        a.token_id = Some("t-a".into());
        let mut b = entry("b", "module_monster");
        b.token_id = Some("t-b".into());
        b.current_hp = Some(0);
        let c = entry("c", "custom");
        let fight = combat(vec![a, b, c], Some("a"));
        assert_eq!(
            map_marks(Some(&fight)),
            (Some("t-a".into()), vec!["t-b".to_string()])
        );
        assert_eq!(map_marks(None), (None, vec![]));
        let no_token = combat(vec![entry("c", "custom")], Some("c"));
        assert_eq!(map_marks(Some(&no_token)), (None, vec![]));
    }

    #[test]
    fn links_are_asked_once_per_set_of_unlinked_creatures() {
        let mut linked = entry("a", "module_monster");
        linked.token_id = Some("t".into());
        let fight = combat(
            vec![
                linked,
                entry("c", "module_npc"),
                entry("b", "module_monster"),
                entry("pc", "character"),
            ],
            None,
        );
        assert_eq!(link_key(&fight, "map"), Some("map|b,c".into()));
        assert_eq!(link_key(&fight, ""), None, "no map");
        assert_eq!(
            link_key(&combat(vec![entry("x", "custom")], None), "map"),
            None
        );
    }

    #[test]
    fn add_menu_values() {
        assert_eq!(add_body("monster:m1").unwrap()["module_monster_id"], "m1");
        assert_eq!(add_body("npc:n1").unwrap()["kind"], "npc");
        assert_eq!(add_body("pc:p1").unwrap()["character_id"], "p1");
        assert!(add_body("").is_none());
        let _ = HpChange {
            kind: "damage".into(),
            amount: 1,
            round: 1,
        };
    }
}
