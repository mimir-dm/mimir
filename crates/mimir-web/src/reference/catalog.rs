//! The catalog browser (MIMIR-T-0723): the kinds of entry by group, a
//! name search, the kind's filters, the result table; a row opens the
//! detail drawer. The kind is in the address (`?category=`).

use std::time::Duration;

use aurora_leptos::components::*;
use aurora_leptos::data::SortState;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};
use mimir_render::summary::CATEGORIES;
use mimir_wire::{CatalogPage, CatalogRow};

use super::detail::{Reference, Target};
use crate::api;

/// A filter of a kind: (query parameter, label, options (value, label)).
pub type Filter = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
);

const YES: &[(&str, &str)] = &[("true", "Yes"), ("false", "No")];

/// The filters of a kind.
pub fn filters(category: &str) -> &'static [Filter] {
    match category {
        "spells" => &[
            (
                "level",
                "Level",
                &[
                    ("0", "Cantrip"),
                    ("1", "1st"),
                    ("2", "2nd"),
                    ("3", "3rd"),
                    ("4", "4th"),
                    ("5", "5th"),
                    ("6", "6th"),
                    ("7", "7th"),
                    ("8", "8th"),
                    ("9", "9th"),
                ],
            ),
            (
                "school",
                "School",
                &[
                    ("A", "Abjuration"),
                    ("C", "Conjuration"),
                    ("D", "Divination"),
                    ("E", "Enchantment"),
                    ("V", "Evocation"),
                    ("I", "Illusion"),
                    ("N", "Necromancy"),
                    ("T", "Transmutation"),
                ],
            ),
            ("ritual", "Ritual", YES),
            ("concentration", "Concentration", YES),
        ],
        "monsters" => &[
            (
                "creature_type",
                "Type",
                &[
                    ("aberration", "Aberration"),
                    ("beast", "Beast"),
                    ("celestial", "Celestial"),
                    ("construct", "Construct"),
                    ("dragon", "Dragon"),
                    ("elemental", "Elemental"),
                    ("fey", "Fey"),
                    ("fiend", "Fiend"),
                    ("giant", "Giant"),
                    ("humanoid", "Humanoid"),
                    ("monstrosity", "Monstrosity"),
                    ("ooze", "Ooze"),
                    ("plant", "Plant"),
                    ("undead", "Undead"),
                ],
            ),
            (
                "size",
                "Size",
                &[
                    ("T", "Tiny"),
                    ("S", "Small"),
                    ("M", "Medium"),
                    ("L", "Large"),
                    ("H", "Huge"),
                    ("G", "Gargantuan"),
                ],
            ),
            (
                "cr",
                "CR",
                &[
                    ("0", "0"),
                    ("1/8", "1/8"),
                    ("1/4", "1/4"),
                    ("1/2", "1/2"),
                    ("1", "1"),
                    ("2", "2"),
                    ("3", "3"),
                    ("4", "4"),
                    ("5", "5"),
                    ("6", "6"),
                    ("7", "7"),
                    ("8", "8"),
                    ("9", "9"),
                    ("10", "10"),
                    ("11", "11"),
                    ("12", "12"),
                    ("13", "13"),
                    ("14", "14"),
                    ("15", "15"),
                    ("16", "16"),
                    ("17", "17"),
                    ("18", "18"),
                    ("19", "19"),
                    ("20", "20"),
                    ("21", "21"),
                    ("22", "22"),
                    ("23", "23"),
                    ("24", "24"),
                    ("30", "30"),
                ],
            ),
        ],
        "items" => &[
            (
                "rarity",
                "Rarity",
                &[
                    ("none", "Mundane"),
                    ("common", "Common"),
                    ("uncommon", "Uncommon"),
                    ("rare", "Rare"),
                    ("very rare", "Very rare"),
                    ("legendary", "Legendary"),
                    ("artifact", "Artifact"),
                ],
            ),
            (
                "item_type",
                "Type",
                &[
                    ("M", "Melee weapon"),
                    ("R", "Ranged weapon"),
                    ("A", "Ammunition"),
                    ("LA", "Light armor"),
                    ("MA", "Medium armor"),
                    ("HA", "Heavy armor"),
                    ("S", "Shield"),
                    ("G", "Adventuring gear"),
                    ("P", "Potion"),
                    ("RG", "Ring"),
                    ("RD", "Rod"),
                    ("SC", "Scroll"),
                    ("ST", "Staff"),
                    ("WD", "Wand"),
                    ("INS", "Instrument"),
                    ("AT", "Artisan's tools"),
                    ("T", "Tools"),
                    ("GS", "Gaming set"),
                    ("SCF", "Spellcasting focus"),
                    ("MNT", "Mount"),
                    ("TAH", "Tack and harness"),
                    ("FD", "Food and drink"),
                ],
            ),
        ],
        _ => &[],
    }
}

/// The search query of a page: `?search=…&level=…&limit=…`.
pub fn query_string(search: &str, chosen: &[(&str, String)], limit: usize) -> String {
    let mut parts = vec![format!("limit={limit}")];
    if !search.trim().is_empty() {
        parts.push(format!("search={}", enc(search.trim())));
    }
    for (k, v) in chosen {
        if !v.is_empty() {
            parts.push(format!("{k}={}", enc(v)));
        }
    }
    parts.join("&")
}

fn enc(s: &str) -> String {
    js_sys::encode_uri_component(s).into()
}

/// The kind of a `?category=` value; spells when absent or unknown.
pub fn category_of(value: Option<&str>) -> &'static str {
    CATEGORIES
        .iter()
        .map(|c| c.0)
        .find(|c| Some(*c) == value)
        .unwrap_or("spells")
}

const PAGE: usize = 100;
const MOST: usize = 500;

/// Compare two cells: as numbers when both are (a challenge rating "1/4"
/// too), else as text without case.
pub fn cell_order(a: &str, b: &str) -> std::cmp::Ordering {
    fn num(s: &str) -> Option<f64> {
        let t = s.trim();
        if let Some((n, d)) = t.split_once('/') {
            let (n, d) = (n.trim().parse::<f64>().ok()?, d.trim().parse::<f64>().ok()?);
            return (d != 0.0).then(|| n / d);
        }
        t.parse::<f64>().ok()
    }
    match (num(a), num(b)) {
        (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
        _ => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}

/// Sort the rows by a column key ("name", "source", or "c<i>" for the
/// `i`th cell); no key: as the server sent them (MIMIR-T-0734).
pub fn sort_rows(rows: &mut [CatalogRow], sort: &SortState) {
    let Some(key) = sort.key.as_deref() else {
        return;
    };
    let value = |r: &CatalogRow| -> String {
        match key {
            "name" => r.name.clone(),
            "source" => r.source.clone(),
            k => k
                .strip_prefix('c')
                .and_then(|i| i.parse::<usize>().ok())
                .and_then(|i| r.cells.get(i).cloned())
                .unwrap_or_default(),
        }
    };
    rows.sort_by(|a, b| sort.order(cell_order(&value(a), &value(b))));
}

#[component]
pub fn CatalogView() -> impl IntoView {
    let query = use_query_map();
    let category = Memo::new(move |_| category_of(query.get().get("category").as_deref()));
    let reference = Reference::get();
    let navigate = use_navigate();
    let go = StoredValue::new(move |to: String| navigate(&to, Default::default()));

    let text = RwSignal::new(String::new());
    let search = RwSignal::new(String::new());
    let timer = StoredValue::new_local(None::<TimeoutHandle>);
    Effect::new(move |_| {
        let t = text.get();
        timer.update_value(|h| {
            if let Some(h) = h.take() {
                h.clear();
            }
        });
        if let Ok(h) = set_timeout_with_handle(move || search.set(t), Duration::from_millis(300)) {
            timer.set_value(Some(h));
        }
    });
    // One signal for each filter a kind can have (at most four).
    let chosen: [RwSignal<String>; 4] = std::array::from_fn(|_| RwSignal::new(String::new()));
    let limit = RwSignal::new(PAGE);
    // The column the results sort by (client side, MIMIR-T-0734).
    let sort = RwSignal::new(SortState::default());
    // A new kind: its filters start empty.
    Effect::new(move |_| {
        category.track();
        for c in chosen {
            c.set(String::new());
        }
        limit.set(PAGE);
    });
    let page = LocalResource::new(move || {
        let cat = category.get();
        let f = filters(cat);
        let picked: Vec<(&str, String)> = f
            .iter()
            .zip(chosen.iter())
            .map(|((k, _, _), s)| (*k, s.get()))
            .collect();
        let q = query_string(&search.get(), &picked, limit.get());
        async move { api::get::<CatalogPage>(format!("/catalog/search/{cat}?{q}")).await }
    });

    let groups = ["Reference", "Player", "Rules", "DM tools"];
    view! {
        <div class="mimir-split mimir-catalog">
            <nav class="mimir-split__side mimir-doclist mimir-catalog__nav" aria-label="Catalog kinds">
                {groups.iter().map(|g| view! {
                    <SectionLabel label=*g />
                    {CATEGORIES.iter().filter(|c| c.2 == *g).map(|(key, label, _)| {
                        let key = *key;
                        let active = move || category.get() == key;
                        view! {
                            <a
                                class="cl-navlink"
                                class:cl-navlink--active=active
                                aria-current=move || active().then_some("page")
                                href=format!("?mode=catalog&category={key}")
                            >
                                <span class="cl-navlink__label">{*label}</span>
                            </a>
                        }
                    }).collect_view()}
                }).collect_view()}
            </nav>
            <div class="mimir-split__main">
                // On a narrow screen the kinds are a select (the list is hidden).
                <label class="mimir-file mimir-catalog__kinds">
                    <span>"Kind"</span>
                    <select
                        class="mimir-wizard__select"
                        aria-label="Kind"
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            go.with_value(|g| g(format!("?mode=catalog&category={v}")));
                        }
                    >
                        {CATEGORIES.iter().map(|(key, label, _)| view! {
                            <option value=*key selected=move || category.get() == *key>{*label}</option>
                        }).collect_view()}
                    </select>
                </label>
                <div class="mimir-catalog__bar">
                    <TextInput label="Name" value=text placeholder="Search by name" />
                    {move || {
                        filters(category.get()).iter().zip(chosen.iter()).map(|((_, label, opts), s)| {
                            let pairs: Vec<(String, String)> = opts.iter().map(|(v, l)| (v.to_string(), l.to_string())).collect();
                            view! { <Select label=*label placeholder="Any" option_pairs=pairs value=*s /> }
                        }).collect_view()
                    }}
                </div>
                {move || match page.get() {
                    None => view! { <Loading /> }.into_any(),
                    Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                    Some(Ok(mut p)) => {
                        let cat = category.get_untracked();
                        let n = p.rows.len();
                        let headers = p.headers.clone();
                        sort_rows(&mut p.rows, &sort.get());
                        view! {
                            <Table label="Results" min_width="640px">
                                <thead>
                                    <tr>
                                        <SortHeader label="Name" key="name" sort=sort />
                                        {headers.iter().enumerate().map(|(i, h)| view! {
                                            <SortHeader label=h.clone() key=format!("c{i}") sort=sort />
                                        }).collect_view()}
                                        <SortHeader label="Source" key="source" sort=sort />
                                    </tr>
                                </thead>
                                <tbody>
                                    {if n == 0 {
                                        view! { <TableEmpty message="Nothing found." colspan=headers.len() as u32 + 2 /> }.into_any()
                                    } else {
                                        p.rows.into_iter().map(|row| {
                                            let t = Target::new(cat, &row.source, &row.name);
                                            let name = row.name.clone();
                                            view! {
                                                <tr class="mimir-catalog__row">
                                                    <td>
                                                        <button
                                                            type="button"
                                                            class="mimir-linkbtn"
                                                            on:click=move |_| if let Some(r) = reference { r.open(t.clone()) }
                                                        >{name}</button>
                                                    </td>
                                                    {row.cells.into_iter().map(|c| view! { <td>{c}</td> }).collect_view()}
                                                    <td>{row.source}</td>
                                                </tr>
                                            }
                                        }).collect_view().into_any()
                                    }}
                                </tbody>
                            </Table>
                            {p.more.then(|| if limit.get_untracked() < MOST {
                                view! {
                                    <Group>
                                        <Button variant="light" on_click=Callback::new(move |_| limit.update(|l| *l = (*l + PAGE).min(MOST)))>"Load more"</Button>
                                    </Group>
                                }.into_any()
                            } else {
                                view! { <Text size="sm" dimmed=true>"More than 500: search by name or filter."</Text> }.into_any()
                            })}
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_from_the_address() {
        assert_eq!(category_of(None), "spells");
        assert_eq!(category_of(Some("monsters")), "monsters");
        assert_eq!(category_of(Some("nonsense")), "spells");
    }

    #[test]
    fn at_most_four_filters() {
        for (k, _, _) in CATEGORIES {
            assert!(filters(k).len() <= 4, "{k}");
        }
    }

    #[test]
    fn rows_sort_by_column() {
        use aurora_leptos::data::SortDir;
        let row = |n: &str, cr: &str, src: &str| CatalogRow {
            name: n.into(),
            source: src.into(),
            cells: vec![cr.into()],
        };
        let mut rows = vec![
            row("Ogre", "2", "MM"),
            row("Goblin", "1/4", "MM"),
            row("ankheg", "2", "MM"),
            row("Lich", "21", "MM"),
        ];
        sort_rows(&mut rows, &SortState::by("c0", SortDir::Asc));
        assert_eq!(
            rows.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
            vec!["Goblin", "Ogre", "ankheg", "Lich"]
        );
        sort_rows(&mut rows, &SortState::by("name", SortDir::Desc));
        assert_eq!(rows[0].name, "Ogre");
        assert_eq!(rows[3].name, "ankheg", "no case: a < g < l < o");
        assert_eq!(
            cell_order("10", "9"),
            std::cmp::Ordering::Greater,
            "numbers, not text"
        );
    }
}
