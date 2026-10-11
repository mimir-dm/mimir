//! The book reader (MIMIR-T-0723): a book with readable content, its
//! contents, one section at a time. The book, the section and the
//! heading are in the address (`?mode=reader&book=PHB&section=3&at=id`).
//! An adventure reads on its own page (`/adventures/<id>?section=3`,
//! MIMIR-T-0745), with its level range, and its `{@area}` links go to the
//! chapter that holds the place (MIMIR-T-0736). Reference › Books lists
//! books only.

use aurora_leptos::components::*;
use aurora_leptos::frame::ConfirmDialog;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_query_map};
use mimir_render::book;
use mimir_wire::{BookData, BookInfo};

use super::detail::Rendered;
use crate::api;

/// The address of a place in a book.
pub fn href(book: &str, section: usize, at: Option<&str>) -> String {
    let mut h = format!("?mode=reader&book={}&section={section}", enc(book));
    if let Some(a) = at {
        h += &format!("&at={}", enc(a));
    }
    h
}

/// The address of a place in an adventure, on its own page.
pub fn adventure_href(id: &str, section: usize, at: Option<&str>) -> String {
    let mut h = format!("/adventures/{}?section={section}", enc(id));
    if let Some(a) = at {
        h += &format!("&at={}", enc(a));
    }
    h
}

fn enc(s: &str) -> String {
    crate::character::inventory::encode(s)
}

/// "Levels 1–3 · Starter Set" for an adventure; empty for a book.
pub fn adventure_line(b: &BookInfo) -> String {
    if b.kind != "adventure" {
        return String::new();
    }
    let levels = match (b.level_start, b.level_end) {
        (Some(a), Some(z)) if a == z => Some(format!("Level {a}")),
        (Some(a), Some(z)) => Some(format!("Levels {a}–{z}")),
        (Some(a), None) => Some(format!("Level {a}")),
        _ => None,
    };
    levels
        .into_iter()
        .chain(b.storyline.clone().filter(|s| !s.is_empty()))
        .collect::<Vec<_>>()
        .join(" · ")
}

/// The reader. Without `adventure`: the books, one chosen in the address.
/// With it: that adventure only.
#[component]
pub fn ReaderView(#[prop(optional, into)] adventure: Option<Signal<String>>) -> impl IntoView {
    let fixed = adventure.is_some();
    let kind = if fixed { "adventure" } else { "book" };
    let addr = move |b: &str, section: usize, at: Option<&str>| {
        if fixed {
            adventure_href(b, section, at)
        } else {
            href(b, section, at)
        }
    };
    let query = use_query_map();
    let navigate = use_navigate();
    let go = StoredValue::new(move |to: String| navigate(&to, Default::default()));
    let sources_v = RwSignal::new(0u32);
    let sources = LocalResource::new(move || {
        sources_v.track();
        api::get::<Vec<BookInfo>>("/books")
    });
    let books = move || {
        sources
            .get()
            .and_then(Result::ok)
            .unwrap_or_default()
            .into_iter()
            .filter(|b| b.kind == kind)
            .collect::<Vec<_>>()
    };
    let chosen = Memo::new(move |_| match adventure {
        Some(a) => a.get(),
        None => query
            .get()
            .get("book")
            .or_else(|| books().first().map(|b| b.source.clone()))
            .unwrap_or_default(),
    });
    let chosen_info = move || books().into_iter().find(|b| b.source == chosen.get());
    let section = Memo::new(move |_| {
        query
            .get()
            .get("section")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0)
    });
    let at = Memo::new(move |_| query.get().get("at"));
    // Remove the open book from the library (MIMIR-T-0734): its source
    // goes from the catalog, as in Settings.
    let removing = RwSignal::new(false);
    // An adventure as a campaign (MIMIR-T-0739).
    let making = RwSignal::new(false);
    let remove_error = RwSignal::new(String::new());
    let remove_book = move |_| {
        let code = chosen.get_untracked();
        spawn_local(async move {
            match api::delete(&format!("/catalog/sources/{}", enc(&code))).await {
                Ok(()) => {
                    removing.set(false);
                    sources_v.update(|v| *v += 1);
                    let back = if fixed {
                        "/adventures"
                    } else {
                        "/reference?mode=reader"
                    };
                    go.with_value(|g| g(back.to_string()));
                }
                Err(e) => {
                    removing.set(false);
                    remove_error.set(api::message(e));
                }
            }
        });
    };
    let data = LocalResource::new(move || {
        let b = chosen.get();
        async move {
            if b.is_empty() {
                return Ok(None);
            }
            api::get::<BookData>(format!("/books/{}", enc(&b)))
                .await
                .map(Some)
        }
    });
    let contents = Memo::new(move |_| match data.get() {
        Some(Ok(Some(d))) => book::contents(&d.data),
        _ => Vec::new(),
    });
    let html = Signal::derive(move || match data.get() {
        Some(Ok(Some(d))) => book::section(&d.data, section.get()),
        _ => String::new(),
    });
    // A heading in the address: scroll to it once the section is in.
    Effect::new(move |_| {
        html.track();
        let target = at.get();
        request_animation_frame(move || {
            let doc = web_sys::window().and_then(|w| w.document());
            match target.and_then(|id| doc.as_ref()?.get_element_by_id(&id)) {
                Some(el) => el.scroll_into_view(),
                None => {
                    if let Some(main) =
                        doc.and_then(|d| d.query_selector(".mimir-reader__page").ok().flatten())
                    {
                        main.scroll_into_view();
                    }
                }
            }
        });
    });

    view! {
        <super::adventure::AdventureCampaignDialog open=making source=Signal::derive(move || chosen.get()) />
        <ConfirmDialog
            open=removing
            title=Signal::derive(move || {
                let name = chosen_info().map(|b| b.name).unwrap_or_else(|| chosen.get());
                format!("Remove {name}?")
            })
            message="Its catalog entries and its book text go; characters and campaigns keep what they hold. Import it again to bring it back."
            confirm_label="Remove"
            on_confirm=Callback::new(remove_book)
            on_cancel=Callback::new(move |_| removing.set(false))
        />
        {move || match sources.get() {
            None => view! { <Loading /> }.into_any(),
            Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
            Some(Ok(_)) if fixed && chosen_info().is_none() => view! {
                <Empty message="This adventure is not in the catalog." />
            }.into_any(),
            Some(Ok(_)) if books().is_empty() => view! {
                <Empty message="No books to read. Import the 5etools data or a homebrew file (Settings)." />
            }.into_any(),
            Some(Ok(_)) => {
                let list = books();
                view! {
                    <div class="mimir-split mimir-reader">
                        <nav class="mimir-split__side mimir-reader__contents" aria-label="Contents">
                            {(!fixed).then(|| view! {
                                <label class="mimir-file">
                                    <span>"Book"</span>
                                    <select
                                        class="mimir-wizard__select"
                                        aria-label="Book"
                                        on:change=move |ev| {
                                            let v = event_target_value(&ev);
                                            go.with_value(|g| g(href(&v, 0, None)));
                                        }
                                    >
                                        {list.into_iter().map(|b| view! {
                                            <option value=b.source.clone() selected=b.source == chosen.get_untracked()>{b.name.clone()}</option>
                                        }).collect_view()}
                                    </select>
                                </label>
                            })}
                            {move || {
                                let line = chosen_info().map(|b| adventure_line(&b)).unwrap_or_default();
                                (!line.is_empty()).then(|| view! { <p class="mimir-reader__levels">{line}</p> })
                            }}
                            {move || chosen_info().is_some_and(|b| b.kind == "adventure").then(|| view! {
                                <Button variant="light" on_click=Callback::new(move |_| making.set(true))>"Make a campaign…"</Button>
                            })}
                            <Button variant="subtle" bad=true on_click=Callback::new(move |_| removing.set(true))>
                                {move || if chosen_info().is_some_and(|b| b.kind == "adventure") { "Remove this adventure…" } else { "Remove this book…" }}
                            </Button>
                            {move || {
                                let e = remove_error.get();
                                (!e.is_empty()).then(|| view! { <Alert color=aurora_leptos::tokens::token::BAD>{e}</Alert> })
                            }}
                            <ol class="mimir-reader__toc">
                                {move || {
                                    let b = chosen.get();
                                    let here = section.get();
                                    contents.get().into_iter().map(|c| {
                                        let current = c.index == here;
                                        let b2 = b.clone();
                                        view! {
                                            <li>
                                                <a
                                                    class="cl-navlink"
                                                    class:cl-navlink--active=current
                                                    aria-current=current.then_some("page")
                                                    href=addr(&b, c.index, None)
                                                >
                                                    <span class="cl-navlink__label">{c.name.clone()}</span>
                                                </a>
                                                {current.then(|| view! {
                                                    <ul class="mimir-reader__headings">
                                                        {c.headings.iter().map(|h| view! {
                                                            <li class=format!("mimir-reader__h{}", h.depth)>
                                                                <a href=addr(&b2, c.index, Some(&h.id))>{h.name.clone()}</a>
                                                            </li>
                                                        }).collect_view()}
                                                    </ul>
                                                })}
                                            </li>
                                        }
                                    }).collect_view()
                                }}
                            </ol>
                        </nav>
                        <div
                            class="mimir-split__main mimir-reader__page"
                            on:click=move |ev: web_sys::MouseEvent| {
                                // An `{@area}` link: the chapter that holds the place.
                                use wasm_bindgen::JsCast;
                                let Some(el) = ev.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) else { return };
                                let Some(link) = el.closest(".area-ref").ok().flatten() else { return };
                                let Some(id) = link.get_attribute("data-area") else { return };
                                ev.prevent_default();
                                let target = match data.get_untracked() {
                                    Some(Ok(Some(d))) => book::section_of(&d.data, &id),
                                    _ => None,
                                };
                                if let Some(sec) = target {
                                    let b = chosen.get_untracked();
                                    go.with_value(|g| g(addr(&b, sec, Some(&id))));
                                }
                            }
                        >
                            {move || match data.get() {
                                None => view! { <Loading /> }.into_any(),
                                Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                                _ => {
                                    let n = contents.get().len();
                                    let i = section.get();
                                    let b = chosen.get();
                                    view! {
                                        <Rendered html=html class="mimir-book" />
                                        <Group>
                                            {(i > 0).then(|| view! { <a class="cl-btn cl-btn--subtle" href=addr(&b, i - 1, None)>"← Previous"</a> })}
                                            {(i + 1 < n).then(|| view! { <a class="cl-btn cl-btn--light" href=addr(&b, i + 1, None)>"Next →"</a> })}
                                        </Group>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>
                }.into_any()
            }
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(kind: &str, start: Option<i64>, end: Option<i64>, story: Option<&str>) -> BookInfo {
        BookInfo {
            source: "X".into(),
            name: "X".into(),
            kind: kind.into(),
            level_start: start,
            level_end: end,
            storyline: story.map(str::to_string),
            cover: None,
            homebrew: false,
        }
    }

    #[test]
    fn adventure_lines() {
        assert_eq!(
            adventure_line(&info("adventure", Some(1), Some(5), Some("Starter Set"))),
            "Levels 1–5 · Starter Set"
        );
        assert_eq!(
            adventure_line(&info("adventure", Some(2), Some(2), None)),
            "Level 2"
        );
        assert_eq!(
            adventure_line(&info("adventure", None, None, Some("Tales"))),
            "Tales"
        );
        assert_eq!(adventure_line(&info("book", Some(1), Some(5), None)), "");
    }

    #[test]
    fn addresses() {
        assert_eq!(href("PHB", 2, None), "?mode=reader&book=PHB&section=2");
        assert_eq!(
            adventure_href("LMoP", 3, Some("a b")),
            "/adventures/LMoP?section=3&at=a%20b"
        );
    }
}
