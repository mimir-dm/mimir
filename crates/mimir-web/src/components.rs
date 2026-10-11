//! Pieces that more than one page uses.

use aurora_leptos::components::*;
use aurora_leptos::tokens::ApiError;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use mimir_wire::{ClassLevel, DocumentSummary};

use crate::{api, markdown};

/// A list read from the API.
pub type ListResource<T> = LocalResource<Result<Vec<T>, ApiError>>;

/// Show a resource: a loader while it loads, the error if it failed, else
/// `ready(value)`.
pub fn loaded<T, V>(
    res: LocalResource<Result<T, ApiError>>,
    ready: impl Fn(T) -> V + 'static,
) -> impl Fn() -> AnyView
where
    T: Clone + 'static,
    V: IntoView + 'static,
{
    move || match res.get() {
        None => view! { <Loading /> }.into_any(),
        Some(Ok(value)) => ready(value).into_any(),
        Some(Err(error)) => view! { <ErrorState error=error /> }.into_any(),
    }
}

/// "Wizard 5", or "Fighter 3 / Rogue 2"; "—" with no class.
pub fn class_label(classes: &[ClassLevel]) -> String {
    if classes.is_empty() {
        return "—".to_string();
    }
    classes
        .iter()
        .map(|c| format!("{} {}", c.class_name, c.level))
        .collect::<Vec<_>>()
        .join(" / ")
}

/// An optional text, or "—".
pub fn or_dash(text: &Option<String>) -> String {
    match text.as_deref().map(str::trim) {
        Some(t) if !t.is_empty() => t.to_string(),
        _ => "—".to_string(),
    }
}

/// Markdown, rendered (raw HTML escaped, see [`markdown`]).
#[component]
pub fn Markdown(#[prop(into)] source: String) -> impl IntoView {
    view! { <div class="mimir-prose" inner_html=markdown::to_html(&source)></div> }
}

/// A list of documents and the selected one, rendered. The selection is
/// in the address (`?doc=<id>`), so a link opens a document; with none,
/// the first document shows.
#[component]
pub fn DocumentBrowser(docs: ListResource<DocumentSummary>) -> impl IntoView {
    let query = use_query_map();
    let selected = Memo::new(move |_| {
        let wanted = query.get().get("doc");
        let list = match docs.get() {
            Some(Ok(list)) => list,
            _ => return None,
        };
        wanted
            .filter(|id| list.iter().any(|d| &d.id == id))
            .or_else(|| list.first().map(|d| d.id.clone()))
    });
    let document = LocalResource::new(move || {
        let id = selected.get();
        async move {
            match id {
                Some(id) => api::document(id).await.map(Some),
                None => Ok(None),
            }
        }
    });

    let list = loaded(docs, move |list: Vec<DocumentSummary>| {
        if list.is_empty() {
            return view! { <Empty message="No documents yet." /> }.into_any();
        }
        view! {
            <nav class="mimir-doclist" aria-label="Documents">
                {list
                    .into_iter()
                    .map(|d| {
                        let id = d.id.clone();
                        let active = move || selected.get().as_deref() == Some(id.as_str());
                        view! {
                            <a
                                class="cl-navlink"
                                class:cl-navlink--active=active.clone()
                                aria-current=move || active().then_some("page")
                                href=format!("?doc={}", d.id)
                            >
                                <span class="cl-navlink__label">{d.title}</span>
                            </a>
                        }
                    })
                    .collect_view()}
            </nav>
        }
        .into_any()
    });
    let viewer = loaded(document, |doc| match doc {
        Some(doc) => view! {
            <article class="mimir-doc">
                <h2 class="mimir-doc__title">{doc.title}</h2>
                <Markdown source=doc.content />
            </article>
        }
        .into_any(),
        None => ().into_any(),
    });

    view! {
        <div class="mimir-split">
            <div class="mimir-split__side">{list}</div>
            <div class="mimir-split__main">{viewer}</div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class(name: &str, level: i32) -> ClassLevel {
        ClassLevel {
            class_name: name.into(),
            subclass_name: None,
            level,
        }
    }

    #[test]
    fn class_labels() {
        assert_eq!(class_label(&[]), "—");
        assert_eq!(class_label(&[class("Wizard", 5)]), "Wizard 5");
        assert_eq!(
            class_label(&[class("Fighter", 3), class("Rogue", 2)]),
            "Fighter 3 / Rogue 2"
        );
    }

    #[test]
    fn empty_text_is_a_dash() {
        assert_eq!(or_dash(&None), "—");
        assert_eq!(or_dash(&Some("  ".into())), "—");
        assert_eq!(or_dash(&Some("Neverwinter".into())), "Neverwinter");
    }
}
