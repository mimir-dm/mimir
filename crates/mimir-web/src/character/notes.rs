//! A character's notes (MIMIR-T-0718): the player writes notes about
//! their encounters (markdown); a new note is about the module whose map
//! is on the display. The DM reads them.

use aurora_leptos::components::*;
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_wire::{NewNote, PlayerNote};
use serde_json::json;

use crate::api;
use crate::components::Markdown;
use crate::editor::RichEditor;

/// The date part of a timestamp.
pub fn day(at: &str) -> &str {
    at.get(..10).unwrap_or(at)
}

/// One note: title, module and date, the markdown.
#[component]
pub fn NoteCard(
    note: PlayerNote,
    /// Show whose note it is (the DM's module view).
    #[prop(optional)]
    by: bool,
    #[prop(optional)] actions: Option<ViewFn>,
) -> impl IntoView {
    let about = note.module_name.clone().unwrap_or_else(|| "—".into());
    let meta = if by {
        format!(
            "{} · {about} · {}",
            note.character_name,
            day(&note.updated_at)
        )
    } else {
        format!("{about} · {}", day(&note.updated_at))
    };
    let (label, title) = (note.title.clone(), note.title.clone());
    view! {
        <li class="mimir-notes__note" aria-label=label>
            <div class="mimir-notes__head">
                <Text bold=true>{title}</Text>
                <Text size="xs" dimmed=true>{meta}</Text>
                {actions.map(|a| a.run())}
            </div>
            <Markdown source=note.content />
        </li>
    }
}

#[component]
pub fn NotesPanel(character_id: String, editable: bool) -> impl IntoView {
    let version = RwSignal::new(0u32);
    let id = StoredValue::new(character_id);
    let notes = LocalResource::new(move || {
        version.track();
        api::get::<Vec<PlayerNote>>(format!("/characters/{}/notes", id.get_value()))
    });

    let editing = RwSignal::new(None::<String>); // note id, or "" for a new one
    let title = RwSignal::new(String::new());
    let content = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());

    let start = move |n: Option<PlayerNote>| {
        let (i, t, c) = match n {
            Some(n) => (n.id, n.title, n.content),
            None => Default::default(),
        };
        title.set(t);
        content.set(c);
        editing.set(Some(i));
        error.set(String::new());
    };
    let save = move |_| {
        let Some(note_id) = editing.get_untracked() else {
            return;
        };
        let (t, c) = (title.get_untracked(), content.get_untracked());
        spawn_local(async move {
            let r = if note_id.is_empty() {
                let body = NewNote {
                    title: t,
                    content: c,
                    module_id: None,
                };
                let path = format!("/characters/{}/notes", id.get_value());
                api::send::<_, PlayerNote>("POST", &path, &body).await
            } else {
                let body = json!({ "title": t, "content": c });
                api::send::<_, PlayerNote>("PATCH", &format!("/notes/{note_id}"), &body).await
            };
            match r {
                Ok(_) => {
                    editing.set(None);
                    version.update(|v| *v += 1);
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    let remove = move |note_id: String| {
        spawn_local(async move {
            match api::delete(&format!("/notes/{note_id}")).await {
                Ok(()) => version.update(|v| *v += 1),
                Err(e) => error.set(api::message(e)),
            }
        });
    };

    view! {
        <div class="mimir-notes">
            {editable.then(|| view! {
                <Group>
                    <Button on_click=Callback::new(move |_| start(None))>"New note"</Button>
                </Group>
            })}
            {move || editing.get().map(|_| view! {
                <div class="mimir-notes__editor">
                    <TextInput label="Title" value=title />
                    <RichEditor value=content label="Note" placeholder="What happened? Start a line with 1. for a numbered list." />
                    <Group>
                        <Button on_click=Callback::new(save)>"Save note"</Button>
                        <Button variant="subtle" on_click=Callback::new(move |_| editing.set(None))>"Cancel"</Button>
                    </Group>
                </div>
            })}
            {move || {
                let e = error.get();
                (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
            }}
            {move || match notes.get() {
                None => view! { <Loading /> }.into_any(),
                Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                Some(Ok(list)) if list.is_empty() => view! {
                    <Text size="sm" dimmed=true>{if editable { "No notes yet. Write what happened." } else { "No notes yet." }}</Text>
                }.into_any(),
                Some(Ok(list)) => view! {
                    <ul class="mimir-notes__list" aria-label="Notes">
                        {list.into_iter().map(|n| {
                            let actions = editable.then(|| {
                                let (edit_note, del_id) = (n.clone(), n.id.clone());
                                ViewFn::from(move || {
                                    let (edit_note, del_id) = (edit_note.clone(), del_id.clone());
                                    view! {
                                        <span class="mimir-sheet__row-actions">
                                            <Button variant="subtle" on_click=Callback::new(move |_| start(Some(edit_note.clone())))>"Edit"</Button>
                                            <ActionIcon title="Delete the note" on_click=Callback::new(move |_| remove(del_id.clone()))>"×"</ActionIcon>
                                        </span>
                                    }
                                })
                            });
                            match actions {
                                Some(a) => view! { <NoteCard note=n actions=a /> }.into_any(),
                                None => view! { <NoteCard note=n /> }.into_any(),
                            }
                        }).collect_view()}
                    </ul>
                }.into_any(),
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days() {
        assert_eq!(day("2026-10-10T12:00:00Z"), "2026-10-10");
        assert_eq!(day("short"), "short");
    }
}
