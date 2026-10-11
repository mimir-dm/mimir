//! The source books of a campaign or a character (MIMIR-T-0724): the
//! catalog searches in it keep to them; none chosen: all books.

use aurora_leptos::components::*;
use aurora_leptos::frame::Modal;
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_wire::{SourceInfo, Sources};

use crate::api;

/// The core rule books, both editions (MIMIR-T-0734, the desktop's "Core
/// Only").
pub const CORE_BOOKS: &[&str] = &["PHB", "MM", "DMG", "XPHB", "XMM", "XDMG"];

/// The core books among those the catalog has.
pub fn core_only(all: &[String]) -> Vec<String> {
    all.iter()
        .filter(|c| CORE_BOOKS.contains(&c.as_str()))
        .cloned()
        .collect()
}

/// `path`: `/campaigns/<id>/sources` or `/characters/<id>/sources`.
#[component]
pub fn SourcesDialog(
    open: RwSignal<bool>,
    #[prop(into)] title: String,
    #[prop(into)] path: Signal<String>,
    /// What keeps to the chosen books.
    #[prop(optional, into)]
    hint: Option<String>,
) -> impl IntoView {
    let hint = StoredValue::new(hint.unwrap_or_else(|| {
        "The catalog searches keep to the books chosen here. None chosen: every book.".into()
    }));
    let all = LocalResource::new(|| api::get::<Vec<SourceInfo>>("/catalog/sources"));
    let chosen = RwSignal::new(Vec::<String>::new());
    let loaded = RwSignal::new(false);
    Effect::new(move |_| {
        if !open.get() {
            loaded.set(false);
            return;
        }
        let p = path.get_untracked();
        spawn_local(async move {
            if let Ok(s) = api::get::<Sources>(&p).await {
                chosen.set(s.codes);
                loaded.set(true);
            }
        });
    });
    let error = RwSignal::new(String::new());
    let save = move |_| {
        let body = Sources {
            codes: chosen.get_untracked(),
        };
        let p = path.get_untracked();
        spawn_local(async move {
            match api::send::<_, Sources>("PUT", &p, &body).await {
                Ok(_) => open.set(false),
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    view! {
        <Modal open=open title=title size="lg">
            <Stack gap="sm">
                <Text size="sm" dimmed=true>{hint.get_value()}</Text>
                <Group>
                    <Button variant="subtle" on_click=Callback::new(move |_| {
                        chosen.set(all.get().and_then(Result::ok).unwrap_or_default().into_iter().map(|s| s.code).collect())
                    })>"All"</Button>
                    <Button variant="subtle" on_click=Callback::new(move |_| chosen.set(Vec::new()))>"None"</Button>
                    <Button variant="subtle" on_click=Callback::new(move |_| {
                        let codes: Vec<String> = all.get().and_then(Result::ok).unwrap_or_default().into_iter().map(|s| s.code).collect();
                        chosen.set(core_only(&codes))
                    })>"Core only"</Button>
                </Group>
                {move || match all.get() {
                    None => view! { <Loading /> }.into_any(),
                    Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                    Some(Ok(list)) => view! {
                        <div class="mimir-wizard__skills mimir-sources" role="group" aria-label="Source books">
                            {list.into_iter().map(|s| {
                                let code = s.code.clone();
                                let c2 = code.clone();
                                view! {
                                    <label class="mimir-wizard__skill">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || chosen.get().contains(&code)
                                            on:change=move |_| chosen.update(|l| {
                                                if let Some(i) = l.iter().position(|x| x == &c2) { l.remove(i); } else { l.push(c2.clone()); }
                                            })
                                        />
                                        <span>{format!("{} ({})", s.name, s.code)}</span>
                                    </label>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any(),
                }}
                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                }}
                <Group>
                    <Button disabled=Signal::derive(move || !loaded.get()) on_click=Callback::new(save)>"Save"</Button>
                    <Button variant="subtle" on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                </Group>
            </Stack>
        </Modal>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_core_books_the_catalog_has() {
        let all: Vec<String> = ["PHB", "XGE", "MM", "XPHB", "TCE"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(core_only(&all), vec!["PHB", "MM", "XPHB"]);
    }
}
