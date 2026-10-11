//! The character sheet (MIMIR-T-0717): one screen for the DM (any
//! character, `/characters/:id`) and a player (their own, on the player
//! page). Tabs: Character (the numbers), Equipment (coins and inventory),
//! Spells (casters), Details (personality, proficiencies, feats, features,
//! classes). The server decides what a caller may change; the screen shows
//! the same actions to both. It reads the sheet again when the socket says
//! the character changed (the DM sees a player's edit live).

use aurora_leptos::components::*;
use aurora_leptos::data::SectionLabel;
use aurora_leptos::frame::{PageHeader, TabItem, TabPanel, Tabs};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use mimir_wire::{CharacterSheet, Choice, KnownSpell, ServerMsg};
use serde_json::json;

use super::inventory::InventoryPanel;
use super::levelup::LevelUpDialog;
use super::stats::{self, signed, ABILITIES, SKILLS};
use crate::api;
use crate::live::Live;

/// `/characters/:id` (DM).
#[component]
pub fn CharacterPage() -> impl IntoView {
    let params = use_params_map();
    let id = Memo::new(move |_| params.get().get("id").unwrap_or_default());
    view! { <CharacterSheetView id=id /> }
}

/// The sheet of a character.
#[component]
pub fn CharacterSheetView(#[prop(into)] id: Signal<String>) -> impl IntoView {
    let version = RwSignal::new(0u32);
    let sheet = LocalResource::new(move || {
        version.track();
        api::get::<CharacterSheet>(format!("/characters/{}", id.get()))
    });
    let reload = Callback::new(move |_| version.update(|v| *v += 1));

    // Live: read again when this character changes.
    let live = StoredValue::new_local(None::<Live>);
    let campaign = Memo::new(move |_| {
        sheet
            .get()
            .and_then(Result::ok)
            .and_then(|s| s.campaign_id)
            .unwrap_or_default()
    });
    Effect::new(move |_| {
        let c = campaign.get();
        if c.is_empty() {
            return;
        }
        live.set_value(Some(Live::watch(c, move |msg| {
            if let ServerMsg::CharacterChanged { character_id } = msg {
                if character_id == id.get_untracked() {
                    version.update(|v| *v += 1);
                }
            }
        })));
    });

    let tab = RwSignal::new("character".to_string());
    // The sheet as it was when the level-up opened: the dialog lives
    // outside the part that rebuilds on each reload (the live notice of its
    // own level-up would rebuild it and lose its answer).
    let leveling = RwSignal::new(None::<CharacterSheet>);

    view! {
        {move || match sheet.get() {
            None => view! { <Loading label="Loading the character…" /> }.into_any(),
            Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
            Some(Ok(s)) => {
                let caster = !stats::casting(&s).is_empty() || !s.spells.is_empty();
                let mut tabs = vec![
                    TabItem::new("character", "Character"),
                    TabItem::new("equipment", "Equipment"),
                ];
                if caster {
                    tabs.push(TabItem::new("spells", "Spells"));
                }
                tabs.push(TabItem::new("details", "Details"));
                let sub = format!(
                    "Level {} · {}{}",
                    stats::total_level(&s),
                    s.race_name.clone().map(|r| format!("{r} · ")).unwrap_or_default(),
                    stats::class_line(&s)
                );
                let can_level = stats::total_level(&s) < 20 && !s.is_npc;
                let (s1, s2, s3, s4, s5) = (s.clone(), s.clone(), s.clone(), s.clone(), s.clone());
                let s5 = StoredValue::new(s5);
                view! {
                    <div class="mimir-sheet">
                        <PageHeader
                            title=s.name.clone()
                            sub=sub
                            actions=Box::new(move || {
                                can_level
                                    .then(|| {
                                        view! {
                                            <Button on_click=Callback::new(move |_| leveling.set(Some(s5.get_value())))>
                                                "Level up"
                                            </Button>
                                        }
                                    })
                                    .into_any()
                            })
                        />
                        <Tabs tabs=tabs value=tab label="Character sections">
                            <TabPanel value="character">{stats_tab(s1.clone())}</TabPanel>
                            <TabPanel value="equipment">
                                <InventoryPanel sheet=s2.clone() on_changed=reload />
                            </TabPanel>
                            <TabPanel value="spells">{spells_tab(s3.clone(), reload)}</TabPanel>
                            <TabPanel value="details">{details_tab(s4.clone(), reload)}</TabPanel>
                        </Tabs>
                    </div>
                }
                    .into_any()
            }
        }}
        {move || leveling.get().map(|s| view! {
            <LevelUpDialog
                sheet=s
                on_close=Callback::new(move |_| leveling.set(None))
                on_done=Callback::new(move |_| {
                    leveling.set(None);
                    version.update(|v| *v += 1);
                })
            />
        })}
    }
}

fn stat_block(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="mimir-sheet__stat">
            <span class="mimir-sheet__stat-label">{label}</span>
            <span class="mimir-sheet__stat-value">{value}</span>
        </div>
    }
}

fn stats_tab(s: CharacterSheet) -> impl IntoView {
    let abilities = ABILITIES
        .iter()
        .map(|a| {
            let score = stats::score(&s, a);
            view! {
                <div class="mimir-sheet__ability" aria-label=a.to_string()>
                    <span class="mimir-sheet__stat-label">{stats::abbrev(a)}</span>
                    <span class="mimir-sheet__ability-mod">{signed(stats::modifier(score))}</span>
                    <span class="mimir-sheet__ability-score">{score}</span>
                </div>
            }
        })
        .collect_view();
    let combat = view! {
        <div class="mimir-sheet__stats">
            {stat_block("Armor class", stats::armor_class(&s).to_string())}
            {stat_block("Initiative", signed(stats::initiative(&s)))}
            {stat_block("Speed", format!("{} ft", s.speed))}
            {stat_block("Proficiency", signed(stats::proficiency_bonus(stats::total_level(&s))))}
            {stat_block("Passive Perception", stats::passive_perception(&s).to_string())}
            {stat_block("Hit dice", stats::hit_dice(&s))}
        </div>
    };
    let saves = ABILITIES
        .iter()
        .map(|a| {
            let prof = stats::save_proficient(&s, a);
            view! {
                <li class:mimir-sheet__prof=prof>
                    <span>{stats::abbrev(a)}</span>
                    <span class="mimir-sheet__num">{signed(stats::save_bonus(&s, a))}</span>
                </li>
            }
        })
        .collect_view();
    let skills = SKILLS
        .iter()
        .map(|(skill, ability)| {
            let rank = stats::skill_rank(&s, skill);
            let (proficient, expert) = (rank > 0, rank == 2);
            view! {
                <li class:mimir-sheet__prof=proficient class:mimir-sheet__expert=expert>
                    <span>{format!("{skill} ({})", stats::abbrev(ability))}</span>
                    <span class="mimir-sheet__num">{signed(stats::skill_bonus(&s, skill, ability))}</span>
                </li>
            }
        })
        .collect_view();
    let attacks = stats::attacks(&s);
    view! {
        <div class="mimir-sheet__abilities">{abilities}</div>
        {combat}
        <div class="mimir-sheet__columns">
            <section>
                <SectionLabel label="Saving throws" />
                <ul class="mimir-sheet__list">{saves}</ul>
                <SectionLabel label="Attacks" />
                {if attacks.is_empty() {
                    view! { <Text size="sm" dimmed=true>"No weapon equipped."</Text> }.into_any()
                } else {
                    view! {
                        <ul class="mimir-sheet__list">
                            {attacks.into_iter().map(|a| view! {
                                <li>
                                    <span>{a.name}</span>
                                    <span class="mimir-sheet__num">{format!("{} · {}", signed(a.to_hit), a.damage)}</span>
                                </li>
                            }).collect_view()}
                        </ul>
                    }.into_any()
                }}
            </section>
            <section>
                <SectionLabel label="Skills" />
                <ul class="mimir-sheet__list">{skills}</ul>
            </section>
        </div>
    }
}

fn spells_tab(s: CharacterSheet, reload: Callback<()>) -> impl IntoView {
    let casting = stats::casting(&s);
    let slots = stats::spell_slots(&s);
    let error = RwSignal::new(String::new());
    let id = s.id.clone();
    let classes: Vec<String> = s
        .classes
        .iter()
        .filter(|k| stats::casting_ability(&k.class_name).is_some())
        .map(|k| k.class_name.clone())
        .collect();
    // Spells to learn: the first casting class's list up to its highest
    // slot level.
    let max_level = slots
        .iter()
        .rposition(|n| *n > 0)
        .map(|i| i as i32 + 1)
        .unwrap_or(0);
    let class0 = classes.first().cloned().unwrap_or_default();
    let choices = LocalResource::new({
        let class0 = class0.clone();
        move || {
            let c = class0.clone();
            async move {
                if c.is_empty() {
                    return Vec::new();
                }
                api::get::<Vec<Choice>>(format!("/catalog/spells?class={c}&max_level={max_level}"))
                    .await
                    .unwrap_or_default()
            }
        }
    });
    let pick = RwSignal::new(String::new());
    let learn = {
        let id = id.clone();
        let class0 = class0.clone();
        move |_| {
            let v = pick.get_untracked();
            let Some((name, source)) = v
                .split_once('|')
                .map(|(a, b)| (a.to_string(), b.to_string()))
            else {
                return;
            };
            let path = format!("/characters/{id}/spells");
            let class0 = class0.clone();
            spawn_local(async move {
                match api::send::<_, KnownSpell>(
                    "POST",
                    &path,
                    &json!({"spell_name": name, "spell_source": source, "source_class": class0}),
                )
                .await
                {
                    Ok(_) => {
                        pick.set(String::new());
                        reload.run(());
                    }
                    Err(e) => error.set(api::message(e)),
                }
            });
        }
    };
    let known_names: Vec<String> = s
        .spells
        .iter()
        .map(|k| k.spell_name.to_lowercase())
        .collect();
    let mut known = s.spells.clone();
    known.sort_by(|a, b| a.spell_name.cmp(&b.spell_name));
    view! {
        <div class="mimir-sheet__stats">
            {casting.into_iter().map(|c| view! {
                <div class="mimir-sheet__stat">
                    <span class="mimir-sheet__stat-label">{format!("{} ({})", c.class, stats::abbrev(c.ability))}</span>
                    <span class="mimir-sheet__stat-value">{format!("DC {} · {}", c.save_dc, signed(c.attack))}</span>
                </div>
            }).collect_view()}
        </div>
        <SectionLabel label="Spell slots" />
        <div class="mimir-sheet__slots">
            {slots.iter().enumerate().filter(|(_, n)| **n > 0).map(|(i, n)| view! {
                <span class="mimir-sheet__slot">{format!("{}: {n}", ordinal(i as i32 + 1))}</span>
            }).collect_view()}
        </div>
        <SectionLabel label="Known spells" count=known.len() />
        {if known.is_empty() {
            view! { <Text size="sm" dimmed=true>"No spells recorded."</Text> }.into_any()
        } else {
            view! {
                <ul class="mimir-sheet__list">
                    {known.into_iter().map(|k| {
                        let prepared = RwSignal::new(k.prepared);
                        let path = format!("/characters/{}/spells/{}", id, k.id);
                        let path2 = path.clone();
                        view! {
                            <li>
                                <span>{format!("{} ({})", k.spell_name, k.source_class)}</span>
                                <span class="mimir-sheet__row-actions">
                                    <Switch label="Prepared" checked=prepared on_change=Callback::new(move |on: bool| {
                                        let path = path.clone();
                                        spawn_local(async move {
                                            if let Err(e) = api::send::<_, KnownSpell>("PATCH", &path, &json!({"prepared": on})).await {
                                                error.set(api::message(e));
                                            }
                                        });
                                    }) />
                                    <ActionIcon title=format!("Forget {}", k.spell_name) on_click=Callback::new(move |_| {
                                        let path = path2.clone();
                                        spawn_local(async move {
                                            match api::delete(&path).await {
                                                Ok(()) => reload.run(()),
                                                Err(e) => error.set(api::message(e)),
                                            }
                                        });
                                    })>"×"</ActionIcon>
                                </span>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            }.into_any()
        }}
        {(!class0.is_empty()).then(|| view! {
            <div class="mimir-sheet__add">
                {move || {
                    let opts: Vec<(String, String)> = choices
                        .get()
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|c| !known_names.contains(&c.name.to_lowercase()))
                        .map(|c| (format!("{}|{}", c.name, c.source), format!("{} — {}", c.name, c.detail.unwrap_or_default())))
                        .collect();
                    view! { <Select aria_label="Spell to learn" placeholder="Learn a spell…" option_pairs=opts value=pick /> }
                }}
                <Button variant="light" on_click=Callback::new(learn)>"Learn"</Button>
            </div>
        })}
        {move || {
            let e = error.get();
            (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
        }}
    }
}

fn ordinal(n: i32) -> String {
    let suffix = match n {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

fn details_tab(s: CharacterSheet, reload: Callback<()>) -> impl IntoView {
    let fields: [(&'static str, &'static str, Option<String>); 4] = [
        ("Traits", "traits", s.traits.clone()),
        ("Ideals", "ideals", s.ideals.clone()),
        ("Bonds", "bonds", s.bonds.clone()),
        ("Flaws", "flaws", s.flaws.clone()),
    ];
    let error = RwSignal::new(String::new());
    let saved = RwSignal::new(false);
    let values: Vec<(&'static str, &'static str, RwSignal<String>)> = fields
        .into_iter()
        .map(|(label, key, v)| (label, key, RwSignal::new(v.unwrap_or_default())))
        .collect();
    let save = {
        let values = values.clone();
        let id = s.id.clone();
        move |_| {
            let mut body = serde_json::Map::new();
            for (_, key, v) in &values {
                let t = v.get_untracked().trim().to_string();
                body.insert(
                    key.to_string(),
                    if t.is_empty() {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(t)
                    },
                );
            }
            let path = format!("/characters/{id}");
            spawn_local(async move {
                match api::send::<_, CharacterSheet>(
                    "PATCH",
                    &path,
                    &serde_json::Value::Object(body),
                )
                .await
                {
                    Ok(_) => {
                        saved.set(true);
                        reload.run(());
                    }
                    Err(e) => error.set(api::message(e)),
                }
            });
        }
    };
    let group = |kind: &str| -> Vec<String> {
        s.proficiencies
            .iter()
            .filter(|p| p.kind == kind)
            .map(|p| p.name.clone())
            .collect()
    };
    let profs = [
        ("Armor", group("armor")),
        ("Weapons", group("weapon")),
        ("Tools", group("tool")),
        ("Languages", group("language")),
    ];
    view! {
        <div class="mimir-sheet__columns">
            <section>
                <SectionLabel label="Personality" />
                <Stack gap="sm">
                    {values.iter().map(|(label, _, v)| view! { <Textarea label=*label value=*v /> }).collect_view()}
                    <Group>
                        <Button on_click=Callback::new(save)>"Save"</Button>
                        {move || saved.get().then(|| view! { <Text size="sm" dimmed=true>"Saved."</Text> })}
                    </Group>
                    {move || {
                        let e = error.get();
                        (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                    }}
                </Stack>
            </section>
            <section>
                <SectionLabel label="Classes" />
                <ul class="mimir-sheet__list">
                    {s.classes.iter().map(|k| view! {
                        <li>
                            <span>{match &k.subclass_name { Some(sub) => format!("{} ({sub})", k.class_name), None => k.class_name.clone() }}</span>
                            <span class="mimir-sheet__num">{format!("level {} · {}", k.level, stats::hit_die(&k.class_name))}</span>
                        </li>
                    }).collect_view()}
                </ul>
                <SectionLabel label="Proficiencies" />
                <dl class="mimir-sheet__profs">
                    {profs.into_iter().map(|(label, names)| view! {
                        <dt>{label}</dt>
                        <dd>{if names.is_empty() { "—".to_string() } else { names.join(", ") }}</dd>
                    }).collect_view()}
                </dl>
                <SectionLabel label="Feats" count=s.feats.len() />
                <ul class="mimir-sheet__list">
                    {s.feats.iter().map(|f| view! { <li><span>{f.name.clone()}</span><span class="mimir-sheet__num">{f.source.clone()}</span></li> }).collect_view()}
                </ul>
                <SectionLabel label="Chosen features" count=s.features.len() />
                <ul class="mimir-sheet__list">
                    {s.features.iter().map(|f| view! {
                        <li><span>{f.name.clone()}</span><span class="mimir-sheet__num">{f.kind.replace('_', " ")}</span></li>
                    }).collect_view()}
                </ul>
            </section>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::ordinal;

    #[test]
    fn ordinals() {
        assert_eq!(
            (1..=4).map(ordinal).collect::<Vec<_>>(),
            vec!["1st", "2nd", "3rd", "4th"]
        );
    }
}
