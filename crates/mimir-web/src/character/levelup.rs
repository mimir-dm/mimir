//! The level-up dialog (MIMIR-T-0717): class (or a new class), subclass,
//! hit points, ASI or feat, spells, class features, review. One request at
//! the end; the server checks it all and changes nothing on an error. The
//! rules of what each step asks are pure functions here (tested); the
//! counts come from the class's catalog progressions (the desktop app used
//! fixed tables, and its class lookup failed, so its spell step never
//! showed).

use aurora_leptos::components::*;
use aurora_leptos::frame::Modal;
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_wire::{
    Abilities, AsiOrFeat, CharacterSheet, Choice, ClassLevelInfo, FeatureChoices, HpMethod,
    InvocationChoices, LevelUpRequest, LevelUpResult, ManeuverChoices, Ref, SpellChanges,
};
use serde_json::Value;

use super::stats::{self, ABILITIES};
use crate::api;

// ---- Rules (host-tested) ------------------------------------------------------

fn score_of(a: &Abilities, key: &str) -> i32 {
    match key {
        "str" => a.strength,
        "dex" => a.dexterity,
        "con" => a.constitution,
        "int" => a.intelligence,
        "wis" => a.wisdom,
        "cha" => a.charisma,
        _ => 0,
    }
}

/// Does a character meet a class's multiclass requirements (`{"str": 13}`,
/// all of them; or `{"or": [{...}, {...}]}`, any one)?
pub fn meets_requirements(reqs: &Value, a: &Abilities) -> bool {
    match reqs {
        Value::Object(o) => {
            if let Some(Value::Array(any)) = o.get("or") {
                return any.iter().any(|r| meets_requirements(r, a));
            }
            o.iter()
                .all(|(k, v)| v.as_i64().is_none_or(|n| i64::from(score_of(a, k)) >= n))
        }
        _ => true,
    }
}

/// "STR 13 and CHA 13", "STR 13 or DEX 13"; `None` for none.
pub fn requirement_line(reqs: &Value) -> Option<String> {
    let part = |o: &serde_json::Map<String, Value>| {
        o.iter()
            .filter_map(|(k, v)| v.as_i64().map(|n| format!("{} {n}", k.to_uppercase())))
            .collect::<Vec<_>>()
            .join(" and ")
    };
    match reqs {
        Value::Object(o) => match o.get("or") {
            Some(Value::Array(any)) => Some(
                any.iter()
                    .filter_map(Value::as_object)
                    .map(part)
                    .collect::<Vec<_>>()
                    .join(" or "),
            ),
            _ => Some(part(o)).filter(|s| !s.is_empty()),
        },
        _ => None,
    }
}

/// The steps of the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Class,
    Subclass,
    HitPoints,
    Asi,
    Spells,
    Features,
    Review,
}

impl Step {
    pub fn label(self) -> &'static str {
        match self {
            Step::Class => "Class",
            Step::Subclass => "Subclass",
            Step::HitPoints => "Hit points",
            Step::Asi => "Ability scores",
            Step::Spells => "Spells",
            Step::Features => "Features",
            Step::Review => "Review",
        }
    }
}

/// The class level after this level-up (1 for a new class).
pub fn new_class_level(sheet: &CharacterSheet, class_name: &str) -> i32 {
    sheet
        .classes
        .iter()
        .find(|k| k.class_name.eq_ignore_ascii_case(class_name))
        .map(|k| k.level + 1)
        .unwrap_or(1)
}

fn has_subclass(sheet: &CharacterSheet, class_name: &str) -> bool {
    sheet
        .classes
        .iter()
        .any(|k| k.class_name.eq_ignore_ascii_case(class_name) && k.subclass_name.is_some())
}

/// The steps for a chosen class.
pub fn steps(sheet: &CharacterSheet, info: Option<&ClassLevelInfo>) -> Vec<Step> {
    let mut out = vec![Step::Class];
    let Some(info) = info else {
        return out;
    };
    let level = new_class_level(sheet, &info.name);
    if level >= info.subclass_level && !has_subclass(sheet, &info.name) {
        out.push(Step::Subclass);
    }
    out.push(Step::HitPoints);
    if info.asi_levels.contains(&level) {
        out.push(Step::Asi);
    }
    let (cantrips, spells) = to_learn(info, level);
    if cantrips + spells > 0 {
        out.push(Step::Spells);
    }
    let subclass = sheet
        .classes
        .iter()
        .find(|k| k.class_name.eq_ignore_ascii_case(&info.name))
        .and_then(|k| k.subclass_name.clone());
    if feature_slots(&info.name, level, subclass.as_deref(), info).any() {
        out.push(Step::Features);
    }
    out.push(Step::Review);
    out
}

/// The hit points gained: the average (die/2 + 1), a roll or a number,
/// plus CON; at least 1.
pub fn hp_gain(die: i32, con_mod: i32, method: &HpMethod) -> i32 {
    let base = match method {
        HpMethod::Average => die / 2 + 1,
        HpMethod::Roll(n) | HpMethod::Manual(n) => *n,
    };
    (base + con_mod).max(1)
}

/// Why an ASI is not valid, if it is not. `two` = +1 to two abilities.
pub fn asi_problem(a: &Abilities, first: &str, second: Option<&str>) -> Option<String> {
    let get = |name: &str| {
        ABILITIES
            .contains(&name)
            .then(|| stats::score_of_abilities(a, name))
    };
    let Some(s1) = get(first) else {
        return Some("Choose an ability.".into());
    };
    match second {
        None if s1 + 2 > 20 => Some("A score cannot go above 20.".into()),
        None => None,
        Some(other) if other == first => Some("Choose two different abilities.".into()),
        Some(other) => match get(other) {
            None => Some("Choose a second ability.".into()),
            Some(s2) if s1 + 1 > 20 || s2 + 1 > 20 => Some("A score cannot go above 20.".into()),
            Some(_) => None,
        },
    }
}

/// The highest spell level of a caster type at a class level.
pub fn max_spell_level(caster: &str, level: i32) -> i32 {
    match caster {
        "full" => ((level + 1) / 2).min(9),
        "pact" => ((level + 1) / 2).min(5),
        "half" if level >= 2 => (level - 1) / 4 + 1,
        "third" if level >= 3 => (level - 1) / 6 + 1,
        _ => 0,
    }
}

fn at(list: &[i32], level: i32) -> i32 {
    if level < 1 {
        return 0;
    }
    list.get((level - 1) as usize).copied().unwrap_or(0)
}

/// (cantrips, spells) to learn on reaching `level`: the growth of the
/// known counts, or the spells a book gains (the Wizard).
pub fn to_learn(info: &ClassLevelInfo, level: i32) -> (i32, i32) {
    if info.caster.is_none() {
        return (0, 0);
    }
    let cantrips = (at(&info.cantrips_known, level) - at(&info.cantrips_known, level - 1)).max(0);
    let spells = if !info.spells_added.is_empty() {
        at(&info.spells_added, level)
    } else {
        (at(&info.spells_known, level) - at(&info.spells_known, level - 1)).max(0)
    };
    (cantrips, spells)
}

/// What the Features step asks for.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FeatureSlots {
    pub fighting_style: bool,
    pub metamagic: i32,
    pub maneuvers: i32,
    pub invocations: i32,
    pub pact_boon: bool,
    pub expertise: i32,
}

impl FeatureSlots {
    pub fn any(&self) -> bool {
        self.fighting_style
            || self.pact_boon
            || self.metamagic + self.maneuvers + self.invocations + self.expertise > 0
    }
}

/// The growth of an optional-feature progression ("Eldritch Invocations")
/// at a level, from the class data.
fn progression_gain(info: &ClassLevelInfo, name_part: &str, level: i32) -> Option<i32> {
    let list = info.optional_features.as_array()?;
    let p = list.iter().find(|p| {
        p.get("name")
            .and_then(Value::as_str)
            .is_some_and(|n| n.to_lowercase().contains(name_part))
    })?;
    let prog = p.get("progression")?;
    let count_at = |l: i32| -> i32 {
        match prog {
            Value::Array(a) => a
                .get((l - 1).max(0) as usize)
                .and_then(Value::as_i64)
                .unwrap_or(0) as i32,
            Value::Object(o) => o
                .iter()
                .filter_map(|(k, v)| Some((k.parse::<i32>().ok()?, v.as_i64()? as i32)))
                .filter(|(k, _)| *k <= l)
                .max_by_key(|(k, _)| *k)
                .map(|(_, v)| v)
                .unwrap_or(0),
            _ => 0,
        }
    };
    if level < 1 {
        return None;
    }
    Some((count_at(level) - if level > 1 { count_at(level - 1) } else { 0 }).max(0))
}

/// What a class asks at a level (5e PHB; counts from the class data when
/// it has them).
pub fn feature_slots(
    class: &str,
    level: i32,
    subclass: Option<&str>,
    info: &ClassLevelInfo,
) -> FeatureSlots {
    let c = class.to_lowercase();
    let mut s = FeatureSlots::default();
    match c.as_str() {
        "fighter" => {
            s.fighting_style = level == 1;
            let battle_master =
                subclass.is_some_and(|n| n.to_lowercase().contains("battle master"));
            if battle_master || level == 3 {
                s.maneuvers = progression_gain(info, "maneuver", level).unwrap_or(match level {
                    3 if battle_master => 3,
                    7 | 10 | 15 if battle_master => 2,
                    _ => 0,
                });
            }
        }
        "paladin" | "ranger" => s.fighting_style = level == 2,
        "sorcerer" => {
            s.metamagic = progression_gain(info, "metamagic", level).unwrap_or(match level {
                3 => 2,
                10 | 17 => 1,
                _ => 0,
            })
        }
        "warlock" => {
            s.invocations = progression_gain(info, "invocation", level).unwrap_or(match level {
                2 => 2,
                5 | 7 | 9 | 12 | 15 | 18 => 1,
                _ => 0,
            });
            s.pact_boon = level == 3;
        }
        "rogue" => s.expertise = if level == 1 || level == 6 { 2 } else { 0 },
        "bard" => s.expertise = if level == 3 || level == 10 { 2 } else { 0 },
        _ => {}
    }
    s
}

/// Names typed by hand ("A, B") as references (source PHB when none is
/// given as "Name|SRC").
pub fn typed_refs(text: &str) -> Vec<Ref> {
    text.split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(|t| match t.split_once('|') {
            Some((n, s)) => Ref {
                name: n.trim().into(),
                source: s.trim().into(),
            },
            None => Ref {
                name: t.into(),
                source: "PHB".into(),
            },
        })
        .collect()
}

/// "Name|SRC" → a reference.
pub fn pair_ref(v: &str) -> Option<Ref> {
    let (n, s) = v.split_once('|')?;
    (!n.is_empty()).then(|| Ref {
        name: n.into(),
        source: s.into(),
    })
}

// ---- The dialog ------------------------------------------------------------------

const PACT_BOONS: [&str; 3] = ["Pact of the Chain", "Pact of the Blade", "Pact of the Tome"];

#[component]
pub fn LevelUpDialog(
    sheet: CharacterSheet,
    on_close: Callback<()>,
    on_done: Callback<()>,
) -> impl IntoView {
    let open = RwSignal::new(true);
    Effect::new(move |_| {
        if !open.get() {
            on_close.run(());
        }
    });
    let sheet = StoredValue::new(sheet);
    let classes = LocalResource::new(|| api::get::<Vec<Choice>>("/catalog/classes"));

    // Choices.
    let class_pick = RwSignal::new({
        sheet.with_value(|s| {
            s.classes
                .first()
                .map(|k| format!("{}|{}", k.class_name, k.class_source))
                .unwrap_or_default()
        })
    });
    let info = LocalResource::new(move || {
        let v = class_pick.get();
        async move {
            let (n, s) = v.split_once('|')?;
            api::get::<ClassLevelInfo>(format!("/catalog/classes/{n}/{s}/level-info"))
                .await
                .ok()
        }
    });
    let info_now = move || info.get().flatten();
    let subclass_pick = RwSignal::new(String::new());
    let subclass_typed = RwSignal::new(String::new());
    let hp_mode = RwSignal::new("average".to_string());
    let hp_value = RwSignal::new(String::new());
    let asi_mode = RwSignal::new("+2 to one".to_string());
    let asi_first = RwSignal::new(String::new());
    let asi_second = RwSignal::new(String::new());
    let feat_pick = RwSignal::new(String::new());
    let feat_typed = RwSignal::new(String::new());
    let cantrips = RwSignal::new(Vec::<Ref>::new());
    let spells = RwSignal::new(Vec::<Ref>::new());
    let style_pick = RwSignal::new(String::new());
    let style_typed = RwSignal::new(String::new());
    let metamagic_typed = RwSignal::new(String::new());
    let invocations_typed = RwSignal::new(String::new());
    let maneuvers_typed = RwSignal::new(String::new());
    let pact_pick = RwSignal::new(String::new());
    let expertise = RwSignal::new(Vec::<String>::new());
    let step = RwSignal::new(0usize);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    let result = RwSignal::new(None::<LevelUpResult>);

    let level = move || {
        info_now()
            .map(|i| sheet.with_value(|s| new_class_level(s, &i.name)))
            .unwrap_or(1)
    };
    let plan = move || sheet.with_value(|s| steps(s, info_now().as_ref()));
    let current = move || plan().get(step.get()).copied().unwrap_or(Step::Class);
    Effect::new(move |_| {
        class_pick.track();
        step.set(0);
    });

    let subclasses = LocalResource::new(move || {
        let v = class_pick.get();
        async move {
            let (n, _) = v.split_once('|')?;
            api::get::<Vec<Choice>>(format!("/catalog/classes/{n}/subclasses"))
                .await
                .ok()
        }
    });
    let feats = LocalResource::new(|| api::get::<Vec<Choice>>("/catalog/feats"));
    let spell_choices = LocalResource::new(move || {
        let i = info.get().flatten();
        async move {
            let i = i?;
            let lvl = sheet.with_value(|s| new_class_level(s, &i.name));
            let max = max_spell_level(i.caster.as_deref().unwrap_or(""), lvl);
            api::get::<Vec<Choice>>(format!("/catalog/spells?class={}&max_level={max}", i.name))
                .await
                .ok()
        }
    });
    let styles =
        LocalResource::new(|| api::get::<Vec<Choice>>("/catalog/optional-features?type=FS"));

    let con_mod = sheet.with_value(|s| stats::modifier(s.abilities.constitution));
    let hp_method = move || -> Option<HpMethod> {
        let die = info_now().map(|i| i.hit_die).unwrap_or(8);
        match hp_mode.get().as_str() {
            "average" => Some(HpMethod::Average),
            "roll" => hp_value
                .get()
                .trim()
                .parse()
                .ok()
                .filter(|n| (1..=die).contains(n))
                .map(HpMethod::Roll),
            _ => hp_value
                .get()
                .trim()
                .parse()
                .ok()
                .filter(|n| *n >= 1)
                .map(HpMethod::Manual),
        }
    };
    let asi_or_feat = move || -> Option<AsiOrFeat> {
        match asi_mode.get().as_str() {
            "feat" => {
                let r = pair_ref(&feat_pick.get())
                    .or_else(|| typed_refs(&feat_typed.get()).into_iter().next())?;
                Some(AsiOrFeat::Feat {
                    name: r.name,
                    source: r.source,
                })
            }
            mode => {
                let first = asi_first.get();
                let second = (mode == "+1 to two").then(|| asi_second.get());
                if sheet
                    .with_value(|s| asi_problem(&s.abilities, &first, second.as_deref()))
                    .is_some()
                {
                    return None;
                }
                Some(AsiOrFeat::AbilityScoreImprovement {
                    ability1: first,
                    increase1: if second.is_some() { 1 } else { 2 },
                    increase2: second.as_ref().map(|_| 1),
                    ability2: second,
                })
            }
        }
    };
    let slots = move || {
        info_now()
            .map(|i| {
                let sub = sheet.with_value(|s| {
                    s.classes
                        .iter()
                        .find(|k| k.class_name.eq_ignore_ascii_case(&i.name))
                        .and_then(|k| k.subclass_name.clone())
                });
                feature_slots(&i.name, level(), sub.as_deref(), &i)
            })
            .unwrap_or_default()
    };
    let learn = move || info_now().map(|i| to_learn(&i, level())).unwrap_or((0, 0));

    let step_done = move || -> bool {
        match current() {
            Step::Class => info_now().is_some_and(|i| {
                let new = sheet.with_value(|s| {
                    !s.classes
                        .iter()
                        .any(|k| k.class_name.eq_ignore_ascii_case(&i.name))
                });
                !new || sheet
                    .with_value(|s| meets_requirements(&i.multiclass_requirements, &s.abilities))
            }),
            Step::Subclass => {
                !subclass_pick.get().is_empty() || !subclass_typed.get().trim().is_empty()
            }
            Step::HitPoints => hp_method().is_some(),
            Step::Asi => asi_or_feat().is_some(),
            Step::Spells => {
                let (c, s) = learn();
                cantrips.with(|v| v.len() as i32 <= c) && spells.with(|v| v.len() as i32 <= s)
            }
            Step::Features | Step::Review => true,
        }
    };

    let request = move || -> Option<LevelUpRequest> {
        let i = info_now()?;
        let has = |s: Step| plan().contains(&s);
        let subclass = has(Step::Subclass).then(|| {
            pair_ref(&subclass_pick.get())
                .or_else(|| typed_refs(&subclass_typed.get()).into_iter().next())
        });
        let sl = slots();
        let features = has(Step::Features).then(|| FeatureChoices {
            fighting_style: sl
                .fighting_style
                .then(|| {
                    pair_ref(&style_pick.get())
                        .or_else(|| typed_refs(&style_typed.get()).into_iter().next())
                })
                .flatten(),
            metamagic: (sl.metamagic > 0)
                .then(|| typed_refs(&metamagic_typed.get()))
                .filter(|v| !v.is_empty()),
            maneuvers: (sl.maneuvers > 0)
                .then(|| ManeuverChoices {
                    new_maneuvers: typed_refs(&maneuvers_typed.get()),
                })
                .filter(|m| !m.new_maneuvers.is_empty()),
            invocations: (sl.invocations > 0)
                .then(|| InvocationChoices {
                    new_invocations: typed_refs(&invocations_typed.get()),
                })
                .filter(|m| !m.new_invocations.is_empty()),
            pact_boon: (sl.pact_boon && !pact_pick.get().is_empty()).then(|| Ref {
                name: pact_pick.get(),
                source: "PHB".into(),
            }),
            expertise_skills: (sl.expertise > 0)
                .then(|| expertise.get())
                .filter(|v| !v.is_empty()),
        });
        Some(LevelUpRequest {
            class_name: i.name.clone(),
            class_source: i.source.clone(),
            hit_points_method: hp_method()?,
            subclass: subclass.flatten(),
            asi_or_feat: if has(Step::Asi) {
                Some(asi_or_feat()?)
            } else {
                None
            },
            spell_changes: has(Step::Spells).then(|| SpellChanges {
                new_cantrips: cantrips.get(),
                new_spells: spells.get(),
                ..SpellChanges::default()
            }),
            feature_choices: features,
        })
    };

    let confirm = move |_| {
        let Some(req) = request() else {
            error.set("A step is not complete.".into());
            return;
        };
        let id = sheet.with_value(|s| s.id.clone());
        busy.set(true);
        spawn_local(async move {
            match api::send::<_, LevelUpResult>("POST", &format!("/characters/{id}/level-up"), &req)
                .await
            {
                Ok(r) => result.set(Some(r)),
                Err(e) => error.set(api::message(e)),
            }
            busy.set(false);
        });
    };

    let toggle = move |list: RwSignal<Vec<Ref>>, r: Ref, limit: i32| {
        list.update(|v| {
            if let Some(i) = v.iter().position(|x| *x == r) {
                v.remove(i);
            } else if (v.len() as i32) < limit {
                v.push(r);
            }
        })
    };

    let body = move || {
        if let Some(r) = result.get() {
            return view! {
                <Stack>
                    <Alert color=token::OK title="Level up done">
                        {format!("Now level {}. Hit points gained: {}.", r.new_total_level, r.hp_gained)}
                    </Alert>
                    <Button on_click=Callback::new(move |_| on_done.run(()))>"Done"</Button>
                </Stack>
            }
            .into_any();
        }
        let progress = plan()
            .iter()
            .enumerate()
            .map(|(n, s)| {
                let here = n == step.get();
                view! { <li class:mimir-levelup__here=here aria-current=here.then_some("step")>{s.label()}</li> }
            })
            .collect_view();
        let content = match current() {
            Step::Class => {
                let opts: Vec<(String, String)> = {
                    let mine: Vec<(String, String)> = sheet.with_value(|s| {
                        s.classes
                            .iter()
                            .map(|k| {
                                (
                                    format!("{}|{}", k.class_name, k.class_source),
                                    format!("{} {} → {}", k.class_name, k.level, k.level + 1),
                                )
                            })
                            .collect()
                    });
                    let others: Vec<(String, String)> = classes
                        .get()
                        .and_then(Result::ok)
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|c| {
                            !mine
                                .iter()
                                .any(|(v, _)| v.starts_with(&format!("{}|", c.name)))
                        })
                        .map(|c| {
                            (
                                format!("{}|{}", c.name, c.source),
                                format!("{} ({}) — new class", c.name, c.source),
                            )
                        })
                        .collect();
                    mine.into_iter().chain(others).collect()
                };
                let req = info_now().and_then(|i| {
                    let new = sheet.with_value(|s| {
                        !s.classes
                            .iter()
                            .any(|k| k.class_name.eq_ignore_ascii_case(&i.name))
                    });
                    if !new {
                        return None;
                    }
                    let line = requirement_line(&i.multiclass_requirements)?;
                    let ok = sheet.with_value(|s| {
                        meets_requirements(&i.multiclass_requirements, &s.abilities)
                    });
                    Some((line, ok))
                });
                view! {
                    <Select label="Class" option_pairs=opts value=class_pick />
                    {move || info_now().map(|i| view! {
                        <Text size="sm" dimmed=true>{format!("Hit die d{} · class level {}", i.hit_die, level())}</Text>
                    })}
                    {req.map(|(line, ok)| view! {
                        <Alert color=if ok { token::OK } else { token::BAD } title="Multiclass requirement">
                            {format!("{line}{}", if ok { " — met." } else { " — not met." })}
                        </Alert>
                    })}
                }
                .into_any()
            }
            Step::Subclass => {
                let opts: Vec<(String, String)> = subclasses
                    .get()
                    .flatten()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|c| {
                        (
                            format!("{}|{}", c.name, c.source),
                            format!("{} ({})", c.name, c.source),
                        )
                    })
                    .collect();
                view! {
                    <Select label="Subclass" placeholder="Choose…" option_pairs=opts value=subclass_pick />
                    <TextInput label="Or type one (Name|SRC)" value=subclass_typed />
                }
                .into_any()
            }
            Step::HitPoints => {
                let die = info_now().map(|i| i.hit_die).unwrap_or(8);
                let gain = hp_method().map(|m| hp_gain(die, con_mod, &m));
                view! {
                    <SegmentedControl options=vec!["average".into(), "roll".into(), "manual".into()] value=hp_mode />
                    {move || (hp_mode.get() != "average").then(|| view! {
                        <TextInput
                            label=if hp_mode.get() == "roll" { format!("Your roll (1–{die})") } else { "Hit points before CON".to_string() }
                            value=hp_value
                            input_type="number"
                        />
                    })}
                    <Text>{match gain {
                        Some(g) => format!("Hit points gained: {g} (CON {}).", stats::signed(con_mod)),
                        None => "Enter a valid number.".to_string(),
                    }}</Text>
                }
                .into_any()
            }
            Step::Asi => {
                let abilities: Vec<(String, String)> = ABILITIES
                    .iter()
                    .map(|a| {
                        let s = sheet.with_value(|s| stats::score(s, a));
                        (a.to_string(), format!("{} ({s})", stats::abbrev(a)))
                    })
                    .collect();
                let feat_opts: Vec<(String, String)> = feats
                    .get()
                    .and_then(Result::ok)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|c| {
                        let label = match &c.detail {
                            Some(d) => format!("{} ({d})", c.name),
                            None => c.name.clone(),
                        };
                        (format!("{}|{}", c.name, c.source), label)
                    })
                    .collect();
                let problem = move || {
                    let mode = asi_mode.get();
                    if mode == "feat" {
                        return None;
                    }
                    let second = (mode == "+1 to two").then(|| asi_second.get());
                    sheet.with_value(|s| {
                        asi_problem(&s.abilities, &asi_first.get(), second.as_deref())
                    })
                };
                let a2 = abilities.clone();
                view! {
                    <SegmentedControl options=vec!["+2 to one".into(), "+1 to two".into(), "feat".into()] value=asi_mode />
                    {move || match asi_mode.get().as_str() {
                        "feat" => view! {
                            <Select label="Feat" placeholder="Choose…" option_pairs=feat_opts.clone() value=feat_pick />
                            <TextInput label="Or type one (Name|SRC)" value=feat_typed />
                        }.into_any(),
                        mode => view! {
                            <Select label="Ability" placeholder="Choose…" option_pairs=abilities.clone() value=asi_first />
                            {(mode == "+1 to two").then(|| view! {
                                <Select label="Second ability" placeholder="Choose…" option_pairs=a2.clone() value=asi_second />
                            })}
                        }.into_any(),
                    }}
                    {move || problem().map(|p| view! { <Text size="sm">{p}</Text> })}
                }
                .into_any()
            }
            Step::Spells => {
                let (nc, ns) = learn();
                let known: Vec<String> = sheet.with_value(|s| {
                    s.spells
                        .iter()
                        .map(|k| k.spell_name.to_lowercase())
                        .collect()
                });
                let list: Vec<Choice> = spell_choices
                    .get()
                    .flatten()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|c| !known.contains(&c.name.to_lowercase()))
                    .collect();
                let (cantrip_list, spell_list): (Vec<Choice>, Vec<Choice>) =
                    list.into_iter().partition(|c| c.level == Some(0));
                let pick = move |title: String,
                                 items: Vec<Choice>,
                                 target: RwSignal<Vec<Ref>>,
                                 limit: i32| {
                    view! {
                        <fieldset class="mimir-levelup__picks">
                            <legend>{move || format!("{title}: {} of {limit}", target.with(Vec::len))}</legend>
                            {items.into_iter().map(|c| {
                                let r = Ref { name: c.name.clone(), source: c.source.clone() };
                                let r2 = r.clone();
                                view! {
                                    <label class="mimir-levelup__pick">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || target.with(|v| v.contains(&r))
                                            on:change=move |_| toggle(target, r2.clone(), limit)
                                        />
                                        {format!("{} — {}", c.name, c.detail.clone().unwrap_or_default())}
                                    </label>
                                }
                            }).collect_view()}
                        </fieldset>
                    }
                };
                view! {
                    {(nc > 0).then(|| pick("Cantrips".into(), cantrip_list.clone(), cantrips, nc))}
                    {(ns > 0).then(|| pick("Spells".into(), spell_list.clone(), spells, ns))}
                    <Text size="sm" dimmed=true>"You can also add spells later on the Spells tab."</Text>
                }
                .into_any()
            }
            Step::Features => {
                let sl = slots();
                let style_opts: Vec<(String, String)> = styles
                    .get()
                    .and_then(Result::ok)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|c| (format!("{}|{}", c.name, c.source), c.name))
                    .collect();
                let skills: Vec<String> = sheet.with_value(|s| {
                    s.proficiencies
                        .iter()
                        .filter(|p| p.kind == "skill" && !p.expertise)
                        .map(|p| p.name.clone())
                        .collect()
                });
                view! {
                    {sl.fighting_style.then(|| view! {
                        <Select label="Fighting style" placeholder="Choose…" option_pairs=style_opts.clone() value=style_pick />
                        <TextInput label="Or type one (Name|SRC)" value=style_typed />
                    })}
                    {(sl.metamagic > 0).then(|| view! {
                        <TextInput label=format!("Metamagic ({}; names, separated by commas)", sl.metamagic) value=metamagic_typed />
                    })}
                    {(sl.maneuvers > 0).then(|| view! {
                        <TextInput label=format!("Maneuvers ({}; names, separated by commas)", sl.maneuvers) value=maneuvers_typed />
                    })}
                    {(sl.invocations > 0).then(|| view! {
                        <TextInput label=format!("Eldritch invocations ({}; names, separated by commas)", sl.invocations) value=invocations_typed />
                    })}
                    {sl.pact_boon.then(|| view! {
                        <Select label="Pact boon" placeholder="Choose…" options=PACT_BOONS.iter().map(|b| b.to_string()).collect() value=pact_pick />
                    })}
                    {(sl.expertise > 0).then(|| {
                        let limit = sl.expertise;
                        view! {
                            <fieldset class="mimir-levelup__picks">
                                <legend>{move || format!("Expertise: {} of {limit}", expertise.with(Vec::len))}</legend>
                                {skills.iter().map(|sk| {
                                    let (a, b) = (sk.clone(), sk.clone());
                                    view! {
                                        <label class="mimir-levelup__pick">
                                            <input type="checkbox"
                                                prop:checked=move || expertise.with(|v| v.contains(&a))
                                                on:change=move |_| expertise.update(|v| {
                                                    if let Some(i) = v.iter().position(|x| *x == b) { v.remove(i); }
                                                    else if (v.len() as i32) < limit { v.push(b.clone()); }
                                                }) />
                                            {sk.clone()}
                                        </label>
                                    }
                                }).collect_view()}
                            </fieldset>
                        }
                    })}
                }
                .into_any()
            }
            Step::Review => {
                let r = request();
                view! {
                    {match r {
                        None => view! { <Alert color=token::BAD>"A step is not complete."</Alert> }.into_any(),
                        Some(r) => {
                            let die = info_now().map(|i| i.hit_die).unwrap_or(8);
                            let mut lines = vec![
                                format!("{} → level {}", r.class_name, level()),
                                format!("Hit points: +{}", hp_gain(die, con_mod, &r.hit_points_method)),
                            ];
                            if let Some(s) = &r.subclass { lines.push(format!("Subclass: {}", s.name)); }
                            match &r.asi_or_feat {
                                Some(AsiOrFeat::Feat { name, .. }) => lines.push(format!("Feat: {name}")),
                                Some(AsiOrFeat::AbilityScoreImprovement { ability1, increase1, ability2, .. }) => lines.push(match ability2 {
                                    Some(a2) => format!("+1 {} and +1 {}", stats::abbrev(ability1), stats::abbrev(a2)),
                                    None => format!("+{increase1} {}", stats::abbrev(ability1)),
                                }),
                                None => {}
                            }
                            if let Some(s) = &r.spell_changes {
                                let names: Vec<String> = s.new_cantrips.iter().chain(&s.new_spells).map(|x| x.name.clone()).collect();
                                if !names.is_empty() { lines.push(format!("Spells: {}", names.join(", "))); }
                            }
                            if let Some(f) = &r.feature_choices {
                                if let Some(x) = &f.fighting_style { lines.push(format!("Fighting style: {}", x.name)); }
                                if let Some(x) = &f.pact_boon { lines.push(format!("Pact boon: {}", x.name)); }
                                if let Some(x) = &f.expertise_skills { lines.push(format!("Expertise: {}", x.join(", "))); }
                            }
                            view! { <ul class="mimir-sheet__list">{lines.into_iter().map(|l| view! { <li>{l}</li> }).collect_view()}</ul> }.into_any()
                        }
                    }}
                }
                .into_any()
            }
        };
        // Until the class facts arrive the plan has one step: not the last.
        let last = info_now().is_some() && step.get() + 1 >= plan().len();
        view! {
            <ol class="mimir-levelup__steps" aria-label="Steps">{progress}</ol>
            <div class="mimir-levelup__body">{content}</div>
            {move || {
                let e = error.get();
                (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
            }}
            <Group justify="between">
                <Button variant="subtle" disabled=Signal::derive(move || step.get() == 0) on_click=Callback::new(move |_| step.update(|s| *s = s.saturating_sub(1)))>"Back"</Button>
                {if last {
                    view! { <Button loading=busy disabled=Signal::derive(move || request().is_none()) on_click=Callback::new(confirm)>"Level up"</Button> }.into_any()
                } else {
                    view! { <Button disabled=Signal::derive(move || !step_done()) on_click=Callback::new(move |_| step.update(|s| *s += 1))>"Next"</Button> }.into_any()
                }}
            </Group>
        }
        .into_any()
    };

    let title = sheet.with_value(|s| format!("Level up {}", s.name));
    view! {
        <Modal open=open title=title size="lg" locked=busy>
            {body}
        </Modal>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::stats::tests::{class, sheet};
    use serde_json::json;

    fn info(name: &str, caster: Option<&str>) -> ClassLevelInfo {
        ClassLevelInfo {
            name: name.into(),
            source: "PHB".into(),
            hit_die: 8,
            subclass_level: 3,
            asi_levels: vec![4, 8, 12, 16, 19],
            multiclass_requirements: Value::Null,
            caster: caster.map(String::from),
            spellcasting_ability: None,
            cantrips_known: vec![],
            spells_known: vec![],
            spells_added: vec![],
            optional_features: Value::Null,
        }
    }

    #[test]
    fn multiclass_requirements() {
        let a = sheet().abilities; // STR 8 DEX 14 CON 12 INT 17 WIS 13 CHA 10
        assert!(meets_requirements(&json!({"int": 13}), &a));
        assert!(!meets_requirements(&json!({"str": 13}), &a));
        assert!(meets_requirements(
            &json!({"or": [{"str": 13}, {"dex": 13}]}),
            &a
        ));
        assert!(
            !meets_requirements(&json!({"dex": 13, "wis": 14}), &a),
            "all of them"
        );
        assert!(meets_requirements(&Value::Null, &a));
        assert_eq!(
            requirement_line(&json!({"or": [{"str": 13}, {"dex": 13}]})).as_deref(),
            Some("STR 13 or DEX 13")
        );
        assert_eq!(requirement_line(&Value::Null), None);
    }

    #[test]
    fn the_steps_follow_the_class_and_level() {
        let mut s = sheet(); // Wizard 5
        assert_eq!(steps(&s, None), vec![Step::Class]);
        let mut w = info("Wizard", Some("full"));
        w.subclass_level = 2;
        w.asi_levels = vec![4, 8, 12, 16, 19];
        w.spells_added = vec![6, 2, 2, 2, 2, 2];
        // Level 6, no subclass yet: subclass, HP, spells.
        assert_eq!(
            steps(&s, Some(&w)),
            vec![
                Step::Class,
                Step::Subclass,
                Step::HitPoints,
                Step::Spells,
                Step::Review
            ]
        );
        s.classes[0].subclass_name = Some("Evocation".into());
        s.classes[0].level = 7; // → 8: ASI
        assert_eq!(
            steps(&s, Some(&w)),
            vec![Step::Class, Step::HitPoints, Step::Asi, Step::Review]
        );
        // A new class at 1: a fighter picks a fighting style.
        let f = info("Fighter", None);
        assert_eq!(
            steps(&s, Some(&f)),
            vec![Step::Class, Step::HitPoints, Step::Features, Step::Review]
        );
    }

    #[test]
    fn hit_points_and_asi_rules() {
        assert_eq!(hp_gain(8, 2, &HpMethod::Average), 7);
        assert_eq!(hp_gain(10, -1, &HpMethod::Roll(1)), 1, "at least 1");
        assert_eq!(hp_gain(6, 3, &HpMethod::Manual(4)), 7, "manual adds CON");
        let a = sheet().abilities;
        assert_eq!(asi_problem(&a, "intelligence", None), None);
        let mut high = a;
        high.intelligence = 19;
        assert!(
            asi_problem(&high, "intelligence", None).is_some(),
            "19 + 2 > 20"
        );
        assert_eq!(
            asi_problem(&high, "intelligence", Some("wisdom")),
            None,
            "19 + 1 is 20"
        );
        assert!(asi_problem(&a, "wisdom", Some("wisdom")).is_some());
        assert!(asi_problem(&a, "", None).is_some());
    }

    #[test]
    fn spell_levels_and_counts() {
        assert_eq!(max_spell_level("full", 5), 3);
        assert_eq!(max_spell_level("full", 20), 9);
        assert_eq!(max_spell_level("half", 1), 0);
        assert_eq!(max_spell_level("half", 5), 2);
        assert_eq!(max_spell_level("third", 7), 2);
        assert_eq!(max_spell_level("pact", 11), 5);
        let mut bard = info("Bard", Some("full"));
        bard.cantrips_known = vec![2, 2, 2, 3];
        bard.spells_known = vec![4, 5, 6, 7];
        assert_eq!(to_learn(&bard, 1), (2, 4));
        assert_eq!(to_learn(&bard, 4), (1, 1));
        let mut wizard = info("Wizard", Some("full"));
        wizard.cantrips_known = vec![3, 3, 3, 4];
        wizard.spells_added = vec![6, 2, 2, 2];
        assert_eq!(to_learn(&wizard, 2), (0, 2), "the book gains 2");
        let cleric = info("Cleric", Some("full"));
        assert_eq!(to_learn(&cleric, 2), (0, 0), "prepared casters learn none");
        assert_eq!(to_learn(&info("Fighter", None), 3), (0, 0));
    }

    #[test]
    fn feature_slots_per_class() {
        let none = info("X", None);
        assert!(feature_slots("Fighter", 1, None, &none).fighting_style);
        assert_eq!(
            feature_slots("Fighter", 3, Some("Battle Master"), &none).maneuvers,
            3
        );
        assert!(!feature_slots("Fighter", 3, Some("Champion"), &none).any());
        assert!(feature_slots("Paladin", 2, None, &none).fighting_style);
        assert_eq!(feature_slots("Sorcerer", 3, None, &none).metamagic, 2);
        let w = feature_slots("Warlock", 3, None, &none);
        assert!(w.pact_boon);
        assert_eq!(feature_slots("Warlock", 2, None, &none).invocations, 2);
        assert_eq!(feature_slots("Rogue", 1, None, &none).expertise, 2);
        assert_eq!(feature_slots("Bard", 10, None, &none).expertise, 2);
        // From the class data when it has the progression.
        let mut lock = info("Warlock", Some("pact"));
        lock.optional_features =
            json!([{"name": "Eldritch Invocations", "progression": [0, 2, 2, 2, 3]}]);
        assert_eq!(feature_slots("Warlock", 5, None, &lock).invocations, 1);
        assert_eq!(feature_slots("Warlock", 2, None, &lock).invocations, 2);
    }

    #[test]
    fn typed_and_paired_references() {
        assert_eq!(
            typed_refs("Agonizing Blast, Mask of Many Faces|XPHB, "),
            vec![
                Ref {
                    name: "Agonizing Blast".into(),
                    source: "PHB".into()
                },
                Ref {
                    name: "Mask of Many Faces".into(),
                    source: "XPHB".into()
                },
            ]
        );
        assert_eq!(
            pair_ref("Evocation|PHB"),
            Some(Ref {
                name: "Evocation".into(),
                source: "PHB".into()
            })
        );
        assert_eq!(pair_ref(""), None);
        let _ = class("Wizard", 1);
    }
}
