//! `/adventures` (MIMIR-T-0745): the adventures to run, a card each, with
//! a search, a level filter and a choice of grouping; each card opens the
//! adventure or makes a campaign of it. `/adventures/<id>`: the adventure
//! in the reader.

use aurora_leptos::components::*;
use aurora_leptos::frame::{Card, PageHeader};
use aurora_leptos::tokens::token;
use std::collections::HashMap;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use mimir_wire::BookInfo;

use crate::api;
use crate::reference::adventure::AdventureCampaignDialog;
use crate::reference::reader::{adventure_href, adventure_line, ReaderView};

/// The group of an adventure without a storyline.
const NO_STORYLINE: &str = "Other adventures";

/// How the list is grouped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupBy {
    Storyline,
    Name,
}

impl GroupBy {
    pub fn parse(v: &str) -> Self {
        match v {
            "name" => Self::Name,
            _ => Self::Storyline,
        }
    }
}

/// An adventure that the search and the level take: the search in its
/// name, storyline or id; the level within its range.
pub fn matches(b: &BookInfo, search: &str, level: Option<i64>) -> bool {
    let q = search.trim().to_lowercase();
    let found = q.is_empty()
        || b.name.to_lowercase().contains(&q)
        || b.source.to_lowercase().contains(&q)
        || b.storyline
            .as_deref()
            .is_some_and(|s| s.to_lowercase().contains(&q));
    let fits = match level {
        None => true,
        Some(l) => match (b.level_start, b.level_end) {
            (Some(a), z) => a <= l && l <= z.unwrap_or(a),
            _ => false,
        },
    };
    found && fits
}

/// The adventures in groups: by storyline (the groups by name, "Other
/// adventures" last; in a group, by first level, then name), or one group
/// by name.
pub fn groups(list: Vec<BookInfo>, by: GroupBy) -> Vec<(String, Vec<BookInfo>)> {
    let by_name = |a: &BookInfo, b: &BookInfo| a.name.to_lowercase().cmp(&b.name.to_lowercase());
    match by {
        GroupBy::Name => {
            let mut all = list;
            all.sort_by(by_name);
            if all.is_empty() {
                Vec::new()
            } else {
                vec![(String::new(), all)]
            }
        }
        GroupBy::Storyline => {
            let mut map: std::collections::BTreeMap<(bool, String), Vec<BookInfo>> =
                Default::default();
            for b in list {
                let story = b
                    .storyline
                    .clone()
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.trim().to_string());
                let key = match story {
                    Some(s) => (false, s),
                    None => (true, NO_STORYLINE.to_string()),
                };
                map.entry(key).or_default().push(b);
            }
            map.into_iter()
                .map(|((_, name), mut l)| {
                    l.sort_by(|a, b| {
                        a.level_start
                            .unwrap_or(i64::MAX)
                            .cmp(&b.level_start.unwrap_or(i64::MAX))
                            .then_with(|| by_name(a, b))
                    });
                    (name, l)
                })
                .collect()
        }
    }
}

#[component]
pub fn AdventuresPage() -> impl IntoView {
    let all = LocalResource::new(|| api::get::<Vec<BookInfo>>("/books"));
    let search = RwSignal::new(String::new());
    let level = RwSignal::new(String::new());
    let group = RwSignal::new("storyline".to_string());
    // The quick "Make a campaign…" of a card.
    let making = RwSignal::new(false);
    let pick = RwSignal::new(String::new());
    // The covers, loaded once: a search draws the cards again.
    let covers: Covers = RwSignal::new(HashMap::new());
    let levels: Vec<(String, String)> = std::iter::once((String::new(), "Any level".to_string()))
        .chain((1..=20).map(|l| (l.to_string(), format!("Level {l}"))))
        .collect();
    let groupings = vec![
        ("storyline".to_string(), "Storyline".to_string()),
        ("name".to_string(), "None (by name)".to_string()),
    ];
    let shown = move || {
        let adventures: Vec<BookInfo> = all
            .get()
            .and_then(Result::ok)
            .unwrap_or_default()
            .into_iter()
            .filter(|b| b.kind == "adventure")
            .collect();
        let total = adventures.len();
        let l = level.get().parse::<i64>().ok();
        let q = search.get();
        let list: Vec<BookInfo> = adventures
            .into_iter()
            .filter(|b| matches(b, &q, l))
            .collect();
        (total, groups(list, GroupBy::parse(&group.get())))
    };
    view! {
        <PageHeader title="Adventures" />
        <AdventureCampaignDialog open=making source=Signal::derive(move || pick.get()) />
        <div class="mimir-adventures__filters">
            <div class="mimir-adventures__search">
                <TextInput label="Search" placeholder="Name or storyline" value=search />
            </div>
            <Select label="Level" option_pairs=levels value=level />
            <Select label="Group by" option_pairs=groupings value=group />
        </div>
        {move || match all.get() {
            None => view! { <Loading /> }.into_any(),
            Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
            Some(Ok(_)) => {
                let (total, groups) = shown();
                if total == 0 {
                    return view! {
                        <Empty message="No adventures yet. Import the 5etools data or a homebrew file (Settings)." />
                    }.into_any();
                }
                if groups.is_empty() {
                    return view! { <Empty message="No adventure matches." /> }.into_any();
                }
                groups.into_iter().map(|(name, list)| {
                    let heading = (!name.is_empty()).then(|| view! { <h2 class="mimir-adventures__group">{name}</h2> });
                    view! {
                        {heading}
                        <div class="mimir-cards">
                            {list.into_iter().map(|b| view! {
                                <AdventureCard
                                    book=b
                                    covers=covers
                                    on_make=Callback::new(move |id: String| {
                                        pick.set(id);
                                        making.set(true);
                                    })
                                />
                            }).collect_view()}
                        </div>
                    }
                }).collect_view().into_any()
            }
        }}
    }
}

#[component]
fn AdventureCard(book: BookInfo, covers: Covers, on_make: Callback<String>) -> impl IntoView {
    let open = adventure_href(&book.source, 0, None);
    let line = adventure_line(&book);
    let id = book.source.clone();
    let name = book.name.clone();
    let label = book.name.clone();
    view! {
        <section class="mimir-adventure" aria-label=label>
            <Card>
                {book.cover.clone().map(|c| view! { <Cover path=c covers=covers /> })}
                <Stack gap="xs">
                    <a class="mimir-adventure__name" href=open.clone()>{name}</a>
                    {(!line.is_empty()).then(|| view! { <Text size="sm" dimmed=true>{line}</Text> })}
                    {book.homebrew.then(|| view! { <div class="mimir-adventure__marks"><Pill color=token::VIOLET>"Homebrew"</Pill></div> })}
                    <Group>
                        <a class="cl-btn cl-btn--subtle" href=open>"Read"</a>
                        <Button variant="light" on_click=Callback::new(move |_| on_make.run(id.clone()))>
                            "Make a campaign…"
                        </Button>
                    </Group>
                </Stack>
            </Card>
        </section>
    }
}

/// The covers of the page: the path, and its image (`None` while it
/// loads, or when the catalog has no such image).
type Covers = RwSignal<HashMap<String, Option<String>>>;

/// A cover from the catalog images; nothing when the images are not in.
#[component]
fn Cover(path: String, covers: Covers) -> impl IntoView {
    if !covers.with_untracked(|c| c.contains_key(&path)) {
        covers.update(|c| {
            c.insert(path.clone(), None);
        });
        let url = path
            .split('/')
            .map(crate::character::inventory::encode)
            .collect::<Vec<_>>()
            .join("/");
        let key = path.clone();
        spawn_local(async move {
            if let Ok(Some(s)) = api::image_url(format!("/book-images/{url}")).await {
                covers.update(|c| {
                    c.insert(key, Some(s));
                });
            }
        });
    }
    move || {
        covers.with(|c| c.get(&path).cloned().flatten()).map(
            |s| view! { <img class="mimir-adventure__cover" src=s alt="Cover" loading="lazy" /> },
        )
    }
}

/// `/adventures/<id>`: the adventure in the reader.
#[component]
pub fn AdventurePage() -> impl IntoView {
    let params = use_params_map();
    let id = Signal::derive(move || params.get().get("id").unwrap_or_default());
    let all = LocalResource::new(|| api::get::<Vec<BookInfo>>("/books"));
    let title = move || {
        let id = id.get();
        all.get()
            .and_then(Result::ok)
            .and_then(|l| l.into_iter().find(|b| b.source == id))
            .map(|b| b.name)
            .unwrap_or_default()
    };
    view! {
        {move || view! { <PageHeader title=title() back_href="/adventures" back_label="Adventures" /> }}
        <ReaderView adventure=id />
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adv(name: &str, story: Option<&str>, levels: (Option<i64>, Option<i64>)) -> BookInfo {
        BookInfo {
            source: name.to_uppercase(),
            name: name.into(),
            kind: "adventure".into(),
            level_start: levels.0,
            level_end: levels.1,
            storyline: story.map(str::to_string),
            cover: None,
            homebrew: false,
        }
    }

    #[test]
    fn search_and_level() {
        let lmop = adv("Lost Mine", Some("Starter Set"), (Some(1), Some(5)));
        assert!(matches(&lmop, "", None));
        assert!(matches(&lmop, "lost", None));
        assert!(matches(&lmop, "starter", None));
        assert!(!matches(&lmop, "dragon", None));
        assert!(matches(&lmop, "", Some(1)));
        assert!(matches(&lmop, "", Some(5)));
        assert!(!matches(&lmop, "", Some(6)));
        // One level only, and none.
        assert!(matches(&adv("One", None, (Some(3), None)), "", Some(3)));
        assert!(!matches(&adv("One", None, (Some(3), None)), "", Some(4)));
        assert!(!matches(&adv("Any", None, (None, None)), "", Some(1)));
    }

    #[test]
    fn grouping() {
        let list = vec![
            adv("b", Some("Tales"), (Some(5), Some(6))),
            adv("a", None, (Some(1), Some(2))),
            adv("c", Some("Tales"), (Some(1), Some(2))),
            adv("d", Some("Starter Set"), (Some(1), Some(5))),
        ];
        let names = |g: &[(String, Vec<BookInfo>)]| {
            g.iter()
                .map(|(n, l)| {
                    (
                        n.clone(),
                        l.iter().map(|b| b.name.clone()).collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            names(&groups(list.clone(), GroupBy::Storyline)),
            vec![
                ("Starter Set".to_string(), vec!["d".to_string()]),
                ("Tales".to_string(), vec!["c".to_string(), "b".to_string()]),
                (NO_STORYLINE.to_string(), vec!["a".to_string()]),
            ]
        );
        assert_eq!(
            names(&groups(list, GroupBy::Name)),
            vec![(
                String::new(),
                vec!["a".into(), "b".into(), "c".into(), "d".into()]
            )]
        );
        assert!(groups(Vec::new(), GroupBy::Name).is_empty());
        assert_eq!(GroupBy::parse("name"), GroupBy::Name);
        assert_eq!(GroupBy::parse("x"), GroupBy::Storyline);
    }
}
