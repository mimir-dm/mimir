//! The character wizard (MIMIR-T-0721). A PC: basics, race, class,
//! background, ability scores, skills, review; the class starts at the
//! first level. An NPC: one form (name, race, role, location, faction).
//! The rules are in [`super::wizard_rules`].

use aurora_leptos::components::*;
use aurora_leptos::frame::Modal;
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_wire::{CampaignSummary, CharacterSheet, Choice, CreationSkills, NewCharacter};

use super::campaigns::optional;
use super::wizard_rules::{self as rules, Draft, Method, Step, ABILITIES};
use crate::api;

fn pairs(list: &[Choice]) -> Vec<(String, String)> {
    list.iter()
        .map(|c| {
            (
                rules::pick_key(&c.name, &c.source),
                format!("{} ({})", c.name, c.source),
            )
        })
        .collect()
}

fn pick_label(key: &str) -> String {
    rules::parse_pick(key)
        .map(|(n, s)| format!("{n} ({s})"))
        .unwrap_or_else(|| "—".into())
}

/// `campaign_id`: the campaign of the new character (none: the DM picks,
/// or no campaign). `npc`: the kind is fixed (none: the DM picks).
#[component]
pub fn CharacterWizard(
    #[prop(optional)] campaign_id: Option<String>,
    #[prop(optional)] npc: Option<bool>,
    on_created: Callback<CharacterSheet>,
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
    let campaign_fixed = campaign_id.is_some();
    let campaign = RwSignal::new(campaign_id.unwrap_or_default());
    let is_npc = RwSignal::new(npc.unwrap_or(false));
    let name = RwSignal::new(String::new());
    let player = RwSignal::new(String::new());
    let race = RwSignal::new(String::new());
    let class = RwSignal::new(String::new());
    let background = RwSignal::new(String::new());
    let method = RwSignal::new(Method::Standard.key().to_string());
    let scores = RwSignal::new(Method::Standard.start());
    let picked = RwSignal::new(Vec::<String>::new());
    let role = RwSignal::new(String::new());
    let location = RwSignal::new(String::new());
    let faction = RwSignal::new(String::new());
    let step = RwSignal::new(0usize);
    let error = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let races = LocalResource::new(|| api::get::<Vec<Choice>>("/catalog/races"));
    let classes = LocalResource::new(|| api::get::<Vec<Choice>>("/catalog/classes"));
    let backgrounds = LocalResource::new(|| api::get::<Vec<Choice>>("/catalog/backgrounds"));
    let campaigns = LocalResource::new(|| api::campaigns(false));
    // The skills of the class and background picked.
    let skills = LocalResource::new(move || {
        let (c, b) = (class.get(), background.get());
        async move {
            let mut q = Vec::new();
            if let Some((n, s)) = rules::parse_pick(&c) {
                q.push(format!("class={}&class_source={}", enc(&n), enc(&s)));
            }
            if let Some((n, s)) = rules::parse_pick(&b) {
                q.push(format!(
                    "background={}&background_source={}",
                    enc(&n),
                    enc(&s)
                ));
            }
            api::get::<CreationSkills>(format!("/catalog/creation-skills?{}", q.join("&")))
                .await
                .unwrap_or_default()
        }
    });
    let skills_now = move || skills.get().unwrap_or_default();
    // A new class or background: pick again.
    Effect::new(move |_| {
        class.track();
        background.track();
        picked.set(Vec::new());
    });
    // A new method: its starting scores.
    Effect::new(move |prev: Option<String>| {
        let m = method.get();
        if prev.is_some_and(|p| p != m) {
            scores.set(Method::from_key(&m).start());
        }
        m
    });

    let draft = move || Draft {
        name: name.get(),
        player_name: player.get(),
        race: rules::parse_pick(&race.get()),
        class: rules::parse_pick(&class.get()),
        background: rules::parse_pick(&background.get()),
        method: Some(Method::from_key(&method.get())),
        scores: scores.get(),
        choose: skills_now().choose,
        picked: picked.get().len(),
    };
    let current = move || Step::PC[step.get().min(Step::PC.len() - 1)];
    let can_go = move || rules::can_proceed(current(), &draft());
    let last = move || step.get() + 1 >= Step::PC.len();

    let create = move |_| {
        let npc_now = is_npc.get_untracked();
        let r = rules::parse_pick(&race.get_untracked());
        let (c, b) = if npc_now {
            (None, None)
        } else {
            (
                rules::parse_pick(&class.get_untracked()),
                rules::parse_pick(&background.get_untracked()),
            )
        };
        let mut selected = if npc_now {
            Vec::new()
        } else {
            skills_now().background
        };
        if !npc_now {
            selected.extend(picked.get_untracked());
        }
        let body = NewCharacter {
            campaign_id: optional(&campaign.get_untracked()),
            is_npc: npc_now,
            name: name.get_untracked(),
            player_name: (!npc_now).then(|| player.get_untracked()),
            race_name: r.as_ref().map(|x| x.0.clone()),
            race_source: r.map(|x| x.1),
            background_name: b.as_ref().map(|x| x.0.clone()),
            background_source: b.map(|x| x.1),
            class_name: c.as_ref().map(|x| x.0.clone()),
            class_source: c.map(|x| x.1),
            ability_scores: (!npc_now).then(|| scores.get_untracked()),
            selected_skills: selected,
            role: npc_now.then(|| role.get_untracked()),
            location: npc_now.then(|| location.get_untracked()),
            faction: npc_now.then(|| faction.get_untracked()),
        };
        busy.set(true);
        spawn_local(async move {
            let r = api::send::<_, CharacterSheet>("POST", "/characters", &body).await;
            busy.set(false);
            match r {
                Ok(s) => {
                    on_created.run(s);
                    open.set(false);
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };

    let campaign_field = move || {
        (!campaign_fixed).then(|| {
            let opts = move || {
                let mut o = vec![(String::new(), "No campaign".to_string())];
                if let Some(Ok(list)) = campaigns.get() {
                    o.extend(list.into_iter().map(|c: CampaignSummary| (c.id, c.name)));
                }
                o
            };
            view! { {move || view! { <Select label="Campaign" option_pairs=opts() value=campaign /> }} }
        })
    };
    // The kind, when the DM picks it: one signal for the whole wizard (a
    // signal made in the form would be made again on each render).
    let kind = RwSignal::new(if is_npc.get_untracked() { "npc" } else { "pc" }.to_string());
    Effect::new(move |_| {
        let npc_now = kind.get() == "npc";
        if is_npc.get_untracked() != npc_now {
            is_npc.set(npc_now);
        }
    });
    let kind_field = move || {
        npc.is_none().then(|| {
            view! {
                <Select
                    label="Kind"
                    option_pairs=vec![("pc".to_string(), "Player character".to_string()), ("npc".to_string(), "NPC".to_string())]
                    value=kind
                />
            }
        })
    };
    let catalog_select = move |label: &'static str,
                               list: LocalResource<
        Result<Vec<Choice>, aurora_leptos::tokens::ApiError>,
    >,
                               value: RwSignal<String>,
                               placeholder: &'static str| {
        move || {
            match list.get() {
            None => view! { <Loading /> }.into_any(),
            Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
            Some(Ok(l)) => view! { <Select label=label placeholder=placeholder option_pairs=pairs(&l) value=value /> }.into_any(),
        }
        }
    };

    let npc_form = move || {
        view! {
            <Stack gap="sm">
                {kind_field()}
                <TextInput label="Name" value=name />
                {catalog_select("Race", races, race, "No race")}
                <TextInput label="Role" value=role />
                <TextInput label="Location" value=location />
                <TextInput label="Faction" value=faction />
                {campaign_field()}
            </Stack>
        }
    };

    let methods: Vec<(String, String)> = Method::ALL
        .iter()
        .map(|(_, k, n)| (k.to_string(), n.to_string()))
        .collect();
    let methods = StoredValue::new(methods);
    let abilities = move || {
        let m = Method::from_key(&method.get());
        let s = scores.get();
        view! {
            <Select
                label="Method"
                option_pairs=methods.get_value()
                value=method
            />
            {(m == Method::PointBuy).then(|| view! {
                <Text size="sm">{format!("Points left: {}", rules::points_left(&s))}</Text>
            })}
            <table class="mimir-wizard__scores" aria-label="Ability scores">
                <tbody>
                    {ABILITIES.iter().enumerate().map(|(i, a)| {
                        let control = if m == Method::Standard {
                            let opts = rules::standard_options(&s, i);
                            view! {
                                <select
                                    class="mimir-wizard__select"
                                    aria-label=*a
                                    on:change=move |ev| {
                                        let v = event_target_value(&ev).parse().unwrap_or(0);
                                        scores.update(|x| x[i] = v);
                                    }
                                >
                                    <option value="0" selected=s[i] == 0>"—"</option>
                                    {opts.into_iter().map(|v| view! { <option value=v.to_string() selected=s[i] == v>{v}</option> }).collect_view()}
                                </select>
                            }.into_any()
                        } else {
                            let (up, down) = (rules::can_raise(m, &s, i), rules::can_lower(m, &s, i));
                            view! {
                                <span class="mimir-wizard__stepper">
                                    <button type="button" class="mimir-order__btn" aria-label=format!("Lower {a}") disabled=!down
                                        on:click=move |_| scores.update(|x| x[i] -= 1)>"−"</button>
                                    <span class="mimir-wizard__score" aria-label=format!("{a} score")>{s[i]}</span>
                                    <button type="button" class="mimir-order__btn" aria-label=format!("Raise {a}") disabled=!up
                                        on:click=move |_| scores.update(|x| x[i] += 1)>"+"</button>
                                </span>
                            }.into_any()
                        };
                        let modifier = if s[i] > 0 { rules::signed(rules::modifier(s[i])) } else { "—".into() };
                        let cost = (m == Method::PointBuy).then(|| rules::point_cost(s[i]).map(|c| format!("{c} pt")).unwrap_or_default());
                        view! {
                            <tr>
                                <th scope="row">{*a}</th>
                                <td>{control}</td>
                                <td class="mimir-sheet__num">{modifier}</td>
                                <td class="mimir-sheet__num">{cost}</td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        }
    };

    let skills_step = move || {
        let s = skills_now();
        let chosen = picked.get();
        let full = chosen.len() >= s.choose;
        view! {
            <SectionLabel label="From the background" />
            <Group>
                {if s.background.is_empty() {
                    view! { <Text size="sm" dimmed=true>"None."</Text> }.into_any()
                } else {
                    s.background.clone().into_iter().map(|b| view! { <Pill color=token::TEAL>{b}</Pill> }).collect_view().into_any()
                }}
            </Group>
            <SectionLabel label=format!("From the class: pick {} ({} picked)", s.choose, chosen.len()) />
            <div class="mimir-wizard__skills" role="group" aria-label="Class skills">
                {s.from.clone().into_iter().map(|k| {
                    let k = k.clone();
                    let on = chosen.contains(&k);
                    let key = k.clone();
                    view! {
                        <label class="mimir-wizard__skill">
                            <input
                                type="checkbox"
                                checked=on
                                disabled=full && !on
                                on:change=move |_| picked.update(|p| {
                                    if let Some(i) = p.iter().position(|x| x == &key) {
                                        p.remove(i);
                                    } else {
                                        p.push(key.clone());
                                    }
                                })
                            />
                            <span>{k}</span>
                        </label>
                    }
                }).collect_view()}
            </div>
        }
    };

    let review = move || {
        let s = scores.get();
        let mut skill_list = skills_now().background;
        skill_list.extend(picked.get());
        view! {
            <dl class="mimir-sheet__profs" aria-label="Review">
                <dt>"Name"</dt><dd>{name.get()}</dd>
                <dt>"Player"</dt><dd>{player.get()}</dd>
                <dt>"Race"</dt><dd>{pick_label(&race.get())}</dd>
                <dt>"Class"</dt><dd>{format!("{} 1", pick_label(&class.get()))}</dd>
                <dt>"Background"</dt><dd>{pick_label(&background.get())}</dd>
                <dt>"Scores"</dt>
                <dd>{ABILITIES.iter().zip(s).map(|(a, v)| format!("{} {v}", &a[..3])).collect::<Vec<_>>().join(" · ")}</dd>
                <dt>"Skills"</dt>
                <dd>{if skill_list.is_empty() { "—".to_string() } else { skill_list.join(", ") }}</dd>
            </dl>
        }
    };

    let pc_form = move || {
        let body = match current() {
            Step::Basics => view! {
                <Stack gap="sm">
                    {kind_field()}
                    <TextInput label="Name" value=name />
                    <TextInput label="Player" value=player />
                    {campaign_field()}
                </Stack>
            }
            .into_any(),
            Step::Race => catalog_select("Race", races, race, "Pick a race").into_any(),
            Step::Class => view! {
                {catalog_select("Class", classes, class, "Pick a class")}
                <Text size="sm" dimmed=true>"The character starts at level 1 in this class."</Text>
            }
            .into_any(),
            Step::Background => {
                catalog_select("Background", backgrounds, background, "Pick a background")
                    .into_any()
            }
            Step::Abilities => abilities().into_any(),
            Step::Skills => skills_step().into_any(),
            Step::Review => review().into_any(),
        };
        view! {
            <ol class="mimir-wizard__steps" aria-label="Steps">
                {Step::PC.iter().enumerate().map(|(i, s)| {
                    let here = i == step.get();
                    view! { <li class:mimir-wizard__here=here aria-current=here.then_some("step")>{s.label()}</li> }
                }).collect_view()}
            </ol>
            <div class="mimir-wizard__body">{body}</div>
        }
    };

    view! {
        <Modal open=open title="New character" size="lg" locked=busy>
            <Stack gap="md">
                {move || if is_npc.get() { npc_form().into_any() } else { pc_form().into_any() }}
                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                }}
                <Group>
                    {move || (!is_npc.get() && step.get() > 0).then(|| view! {
                        <Button variant="subtle" on_click=Callback::new(move |_| step.update(|s| *s -= 1))>"Back"</Button>
                    })}
                    {move || if is_npc.get() || last() {
                        view! {
                            <Button
                                disabled=Signal::derive(move || busy.get() || name.get().trim().is_empty())
                                on_click=Callback::new(create)
                            >"Create"</Button>
                        }.into_any()
                    } else {
                        view! {
                            <Button
                                disabled=Signal::derive(move || !can_go())
                                on_click=Callback::new(move |_| step.update(|s| *s += 1))
                            >"Next"</Button>
                        }.into_any()
                    }}
                    <Button variant="subtle" on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                </Group>
            </Stack>
        </Modal>
    }
}

fn enc(s: &str) -> String {
    js_sys::encode_uri_component(s).into()
}
