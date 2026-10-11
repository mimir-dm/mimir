//! `/reference` (MIMIR-T-0723): the catalog and the book reader, which
//! the desktop opened in a window of its own; and the detail drawer the
//! other pages share (the module page's inspector).

pub mod adventure;
pub mod catalog;
pub mod detail;
pub mod reader;

use aurora_leptos::frame::{PageHeader, TabItem, Tabs};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use catalog::CatalogView;
use reader::ReaderView;

/// The mode of a `?mode=` value: the catalog unless "reader".
pub fn mode_of(value: Option<&str>) -> &'static str {
    match value {
        Some("reader") => "reader",
        _ => "catalog",
    }
}

#[component]
pub fn ReferencePage() -> impl IntoView {
    let query = use_query_map();
    let current = move || mode_of(query.get().get("mode").as_deref()).to_string();
    let mode = RwSignal::new(current());
    Effect::new(move |_| mode.set(current()));
    let tabs = vec![
        TabItem::new("catalog", "Catalog").href("?mode=catalog"),
        TabItem::new("reader", "Books").href("?mode=reader"),
    ];
    view! {
        <PageHeader title="Reference" />
        <Tabs tabs=tabs value=mode label="Reference sections" />
        {move || if mode.get() == "reader" {
            view! { <ReaderView /> }.into_any()
        } else {
            view! { <CatalogView /> }.into_any()
        }}
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn modes() {
        assert_eq!(super::mode_of(None), "catalog");
        assert_eq!(super::mode_of(Some("reader")), "reader");
        assert_eq!(super::mode_of(Some("x")), "catalog");
    }
}
