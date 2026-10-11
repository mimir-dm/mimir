//! The book reader (MIMIR-T-0723): a book with readable content, its
//! contents, one section at a time. The book, the section and the
//! heading are in the address (`?mode=reader&book=PHB&section=3&at=id`).

use aurora_leptos::components::*;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};
use mimir_render::book;
use mimir_wire::{BookData, SourceInfo};

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

fn enc(s: &str) -> String {
    js_sys::encode_uri_component(s).into()
}

#[component]
pub fn ReaderView() -> impl IntoView {
    let query = use_query_map();
    let navigate = use_navigate();
    let go = StoredValue::new(move |to: String| navigate(&to, Default::default()));
    let sources = LocalResource::new(|| api::get::<Vec<SourceInfo>>("/catalog/sources"));
    let books = move || {
        sources
            .get()
            .and_then(Result::ok)
            .map(|l| l.into_iter().filter(|s| s.book).collect::<Vec<_>>())
            .unwrap_or_default()
    };
    let chosen = Memo::new(move |_| {
        query
            .get()
            .get("book")
            .or_else(|| books().first().map(|b| b.code.clone()))
            .unwrap_or_default()
    });
    let section = Memo::new(move |_| {
        query
            .get()
            .get("section")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0)
    });
    let at = Memo::new(move |_| query.get().get("at"));
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
        {move || match sources.get() {
            None => view! { <Loading /> }.into_any(),
            Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
            Some(Ok(_)) if books().is_empty() => view! {
                <Empty message="No books to read. Import a book with its content (Settings)." />
            }.into_any(),
            Some(Ok(_)) => {
                let list = books();
                view! {
                    <div class="mimir-split mimir-reader">
                        <nav class="mimir-split__side mimir-reader__contents" aria-label="Contents">
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
                                    {list.iter().map(|b| view! {
                                        <option value=b.code.clone() selected=b.code == chosen.get_untracked()>{b.name.clone()}</option>
                                    }).collect_view()}
                                </select>
                            </label>
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
                                                    href=href(&b, c.index, None)
                                                >
                                                    <span class="cl-navlink__label">{c.name.clone()}</span>
                                                </a>
                                                {current.then(|| view! {
                                                    <ul class="mimir-reader__headings">
                                                        {c.headings.iter().map(|h| view! {
                                                            <li class=format!("mimir-reader__h{}", h.depth)>
                                                                <a href=href(&b2, c.index, Some(&h.id))>{h.name.clone()}</a>
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
                        <div class="mimir-split__main mimir-reader__page">
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
                                            {(i > 0).then(|| view! { <a class="cl-btn cl-btn--subtle" href=href(&b, i - 1, None)>"← Previous"</a> })}
                                            {(i + 1 < n).then(|| view! { <a class="cl-btn cl-btn--light" href=href(&b, i + 1, None)>"Next →"</a> })}
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
