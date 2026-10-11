//! The documents of a campaign or a module, to read and to write
//! (MIMIR-T-0720). The DM reads a document; Edit opens the rich editor,
//! which saves one second after the last change (as the desktop app).
//! The starting documents keep their order; the DM's own documents move
//! up and down. The selection is in the address (`?doc=<id>`). Images
//! (handouts, portraits) list under the documents; a new document can be
//! an image (MIMIR-T-0731).

use std::time::Duration;

use aurora_leptos::components::*;
use aurora_leptos::frame::{ConfirmDialog, Modal};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_query_map};
use mimir_wire::{
    image_type, Asset, Direction, Document, DocumentPatch, DocumentSummary, Move, NewDocument,
};

use crate::api;
use crate::components::{DropZone, Markdown};
use crate::editor::RichEditor;

/// The DM's own documents (the others came with the campaign or module).
pub fn is_own(doc_type: &str) -> bool {
    matches!(doc_type, "user_document" | "note")
}

/// The title of an uploaded file: its name without the extension.
pub fn title_of_file(name: &str) -> String {
    let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name);
    stem.replace(['-', '_'], " ").trim().to_string()
}

/// Where the editor is with saving.
#[derive(Debug, Clone, PartialEq)]
pub enum Save {
    Saved,
    Pending,
    Saving,
    Failed(String),
}

impl Save {
    pub fn label(&self) -> String {
        match self {
            Save::Saved => "Saved".into(),
            Save::Pending | Save::Saving => "Saving…".into(),
            Save::Failed(e) => format!("Not saved: {e}"),
        }
    }
}

/// The wait after the last change before a save.
const AUTOSAVE: Duration = Duration::from_millis(1000);

/// `base`: `/campaigns/<id>` or `/modules/<id>`; the documents are under it.
#[component]
pub fn DocumentWorkspace(#[prop(into)] base: Signal<String>) -> impl IntoView {
    let version = RwSignal::new(0u32);
    let docs = LocalResource::new(move || {
        version.track();
        api::get::<Vec<DocumentSummary>>(format!("{}/documents", base.get()))
    });
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
    let navigate = use_navigate();
    let open_doc = StoredValue::new(move |id: String| {
        navigate(&format!("?doc={id}"), Default::default());
    });
    let creating = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    let images_v = RwSignal::new(0u32);
    let images = LocalResource::new(move || {
        images_v.track();
        api::get::<Vec<Asset>>(format!("{}/images", base.get()))
    });
    let viewing = RwSignal::new(None::<Asset>);

    let move_doc = move |id: String, direction: Direction| {
        spawn_local(async move {
            match api::send::<_, Vec<DocumentSummary>>(
                "POST",
                &format!("/documents/{id}/move"),
                &Move { direction },
            )
            .await
            {
                Ok(_) => version.update(|v| *v += 1),
                Err(e) => error.set(api::message(e)),
            }
        });
    };

    let nav_item = move |d: DocumentSummary, order: Option<(bool, bool)>| {
        let id = d.id.clone();
        let active = {
            let id = id.clone();
            move || selected.get().as_deref() == Some(id.as_str())
        };
        let (up_id, down_id) = (id.clone(), id.clone());
        let title = d.title.clone();
        view! {
            <div class="mimir-doclist__row">
                <a
                    class="cl-navlink"
                    class:cl-navlink--active=active.clone()
                    aria-current=move || active().then_some("page")
                    href=format!("?doc={}", d.id)
                >
                    <span class="cl-navlink__label">{d.title}</span>
                </a>
                {order.map(|(first, last)| view! {
                    <span class="mimir-doclist__order">
                        <OrderButtons
                            name=title
                            first=first
                            last=last
                            on_move=Callback::new(move |d: Direction| move_doc(if d == Direction::Up { up_id.clone() } else { down_id.clone() }, d))
                        />
                    </span>
                })}
            </div>
        }
    };

    let list = move || match docs.get() {
        None => view! { <Loading /> }.into_any(),
        Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
        Some(Ok(list)) => {
            let (own, starting): (Vec<_>, Vec<_>) =
                list.into_iter().partition(|d| is_own(&d.doc_type));
            let n = own.len();
            view! {
                <nav class="mimir-doclist" aria-label="Documents">
                    {starting.into_iter().map(|d| nav_item(d, None)).collect_view()}
                    {(n > 0).then(|| view! { <SectionLabel label="Your documents" count=n /> })}
                    {own.into_iter().enumerate().map(|(i, d)| nav_item(d, Some((i == 0, i + 1 == n)))).collect_view()}
                </nav>
            }
            .into_any()
        }
    };

    view! {
        <div class="mimir-split">
            <div class="mimir-split__side">
                <Group>
                    <Button variant="light" on_click=Callback::new(move |_| creating.set(true))>"New document"</Button>
                </Group>
                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                }}
                {list}
                {move || images.get().and_then(Result::ok).filter(|l| !l.is_empty()).map(|list| {
                    let n = list.len();
                    view! {
                        <nav class="mimir-doclist" aria-label="Images">
                            <SectionLabel label="Images" count=n />
                            {list.into_iter().map(|a| {
                                let label = a.title.clone().unwrap_or_else(|| a.filename.clone());
                                view! {
                                    <button type="button" class="cl-navlink mimir-doclist__image" on:click=move |_| viewing.set(Some(a.clone()))>
                                        <span class="cl-navlink__label">{label}</span>
                                    </button>
                                }
                            }).collect_view()}
                        </nav>
                    }
                })}
            </div>
            <div class="mimir-split__main">
                {move || selected.get().map(|id| view! {
                    <DocumentPane
                        id=id
                        on_changed=Callback::new(move |_| version.update(|v| *v += 1))
                    />
                })}
            </div>
        </div>
        <NewDocumentDialog
            open=creating
            base=base
            on_created=Callback::new(move |id: String| {
                version.update(|v| *v += 1);
                open_doc.with_value(|f| f(id));
            })
            on_image=Callback::new(move |a: Asset| {
                images_v.update(|v| *v += 1);
                viewing.set(Some(a));
            })
        />
        <ImageViewer viewing=viewing on_deleted=Callback::new(move |_| images_v.update(|v| *v += 1)) />
    }
}

/// Up and down buttons for an item of an ordered list.
#[component]
pub fn OrderButtons(
    #[prop(into)] name: String,
    first: bool,
    last: bool,
    on_move: Callback<Direction>,
) -> impl IntoView {
    let (up, down) = (format!("Move {name} up"), format!("Move {name} down"));
    view! {
        <span class="mimir-order">
            <button type="button" class="mimir-order__btn" title=up.clone() aria-label=up disabled=first
                on:click=move |_| on_move.run(Direction::Up)>"↑"</button>
            <button type="button" class="mimir-order__btn" title=down.clone() aria-label=down disabled=last
                on:click=move |_| on_move.run(Direction::Down)>"↓"</button>
        </span>
    }
}

/// One document: read, or edit with autosave.
#[component]
fn DocumentPane(id: String, on_changed: Callback<()>) -> impl IntoView {
    let id = StoredValue::new(id);
    let doc =
        LocalResource::new(move || api::get::<Document>(format!("/documents/{}", id.get_value())));
    let editing = RwSignal::new(false);
    let content = RwSignal::new(String::new());
    let title = RwSignal::new(String::new());
    let saved = StoredValue::new((String::new(), String::new())); // (title, content) on the server
    let state = RwSignal::new(Save::Saved);
    let timer = StoredValue::new_local(None::<TimeoutHandle>);
    let deleting = RwSignal::new(false);
    let navigate = use_navigate();

    // The loaded document is the saved state.
    Effect::new(move |_| {
        if let Some(Ok(d)) = doc.get() {
            saved.set_value((d.title.clone(), d.content.clone()));
            title.set(d.title);
            content.set(d.content);
            state.set(Save::Saved);
        }
    });

    let save_now = move || {
        timer.update_value(|t| {
            if let Some(t) = t.take() {
                t.clear();
            }
        });
        let (t, c) = (title.get_untracked(), content.get_untracked());
        let (st, sc) = saved.get_value();
        if t == st && c == sc {
            state.set(Save::Saved);
            return;
        }
        let patch = DocumentPatch {
            title: (t != st).then(|| t.clone()),
            content: (c != sc).then(|| c.clone()),
        };
        state.set(Save::Saving);
        spawn_local(async move {
            match api::send::<_, Document>(
                "PATCH",
                &format!("/documents/{}", id.get_value()),
                &patch,
            )
            .await
            {
                Ok(d) => {
                    // The pane may be gone (another document, another
                    // page): the save is done; nothing to show.
                    let (Some(t), Some(c)) =
                        (title.try_get_untracked(), content.try_get_untracked())
                    else {
                        return;
                    };
                    let _ = saved.try_update_value(|v| *v = (d.title.clone(), d.content.clone()));
                    // A change while saving waits for the next save.
                    if t == d.title && c == d.content {
                        state.set(Save::Saved);
                    }
                    if patch.title.is_some() {
                        let _ = on_changed.try_run(());
                    }
                }
                Err(e) => {
                    state.try_set(Save::Failed(api::message(e)));
                }
            }
        });
    };
    // A change: save after the pause.
    Effect::new(move |_| {
        let (t, c) = (title.get(), content.get());
        if !editing.get_untracked() {
            return;
        }
        let (st, sc) = saved.get_value();
        if t == st && c == sc {
            return;
        }
        state.set(Save::Pending);
        timer.update_value(|old| {
            if let Some(h) = old.take() {
                h.clear();
            }
        });
        if let Ok(h) = set_timeout_with_handle(save_now, AUTOSAVE) {
            timer.set_value(Some(h));
        }
    });
    on_cleanup(move || {
        // Leaving with a change: save it.
        if editing.get_untracked() && state.get_untracked() == Save::Pending {
            save_now();
        }
    });

    let remove = move |_| {
        let navigate = navigate.clone();
        spawn_local(async move {
            match api::delete(&format!("/documents/{}", id.get_value())).await {
                Ok(()) => {
                    deleting.set(false);
                    on_changed.run(());
                    navigate("?", Default::default());
                }
                Err(e) => state.set(Save::Failed(api::message(e))),
            }
        });
    };

    view! {
        {move || match doc.get() {
            None => view! { <Loading /> }.into_any(),
            Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
            Some(Ok(_)) => view! {
                <article class="mimir-doc">
                    <div class="mimir-doc__head">
                        {move || if editing.get() {
                            view! { <TextInput label="Title" value=title /> }.into_any()
                        } else {
                            view! { <h2 class="mimir-doc__title">{title.get()}</h2> }.into_any()
                        }}
                        <Group>
                            {move || editing.get().then(|| {
                                let s = state.get();
                                let failed = matches!(s, Save::Failed(_));
                                view! { <span class="mimir-doc__save" class:mimir-doc__save--bad=failed role="status">{s.label()}</span> }
                            })}
                            {move || if editing.get() {
                                view! {
                                    <Button on_click=Callback::new(move |_| { save_now(); editing.set(false); })>"Done"</Button>
                                }.into_any()
                            } else {
                                view! {
                                    <Button variant="light" on_click=Callback::new(move |_| editing.set(true))>"Edit"</Button>
                                }.into_any()
                            }}
                            <Button variant="subtle" on_click=Callback::new(move |_| {
                                let (doc, t) = (id.get_value(), title.get_untracked());
                                spawn_local(async move {
                                    if let Err(e) = api::download(&format!("/documents/{doc}/pdf"), &format!("{t}.pdf")).await {
                                        state.set(Save::Failed(api::message(e)));
                                    }
                                });
                            })>"PDF"</Button>
                            <ActionIcon title="Delete the document" on_click=Callback::new(move |_| deleting.set(true))>"×"</ActionIcon>
                        </Group>
                    </div>
                    {move || if editing.get() {
                        view! { <RichEditor value=content label="Document" /> }.into_any()
                    } else {
                        view! { <Markdown source=content.get() /> }.into_any()
                    }}
                </article>
            }.into_any(),
        }}
        <ConfirmDialog
            open=deleting
            title=Signal::derive(move || format!("Delete {}?", title.get()))
            message="The document goes for good."
            confirm_label="Delete"
            on_confirm=Callback::new(remove)
        />
    }
}

/// A new document: a title, or a markdown file.
#[component]
fn NewDocumentDialog(
    open: RwSignal<bool>,
    #[prop(into)] base: Signal<String>,
    on_created: Callback<String>,
    on_image: Callback<Asset>,
) -> impl IntoView {
    let title = RwSignal::new(String::new());
    let content = RwSignal::new(None::<String>);
    let file_name = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    // An image file: it becomes an image of the campaign or module.
    let image = StoredValue::new_local(None::<web_sys::File>);
    let preview = RwSignal::new(None::<String>);

    let pick = Callback::new(move |file: web_sys::File| {
        error.set(String::new());
        if image_type(&file.name()).is_some() {
            if file.size() > 50.0 * 1024.0 * 1024.0 {
                error.set("An image can be 50 MB at most.".into());
                return;
            }
            let name = file.name();
            if title.get_untracked().trim().is_empty() {
                title.set(title_of_file(&name));
            }
            file_name.set(name);
            content.set(None);
            preview.set(web_sys::Url::create_object_url_with_blob(&file).ok());
            image.set_value(Some(file));
            return;
        }
        image.set_value(None);
        preview.set(None);
        if file.size() > 5.0 * 1024.0 * 1024.0 {
            error.set("A markdown file can be 5 MB at most.".into());
            return;
        }
        let name = file.name();
        if title.get_untracked().trim().is_empty() {
            title.set(title_of_file(&name));
        }
        file_name.set(name);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(file.text()).await {
                Ok(text) => content.set(text.as_string()),
                Err(_) => error.set("The file could not be read.".into()),
            }
        });
    });
    let reset = move || {
        open.set(false);
        title.set(String::new());
        content.set(None);
        file_name.set(String::new());
        error.set(String::new());
        image.set_value(None);
        preview.set(None);
    };
    let create = move |_| {
        if let Some(f) = image.get_value() {
            let Ok(form) = web_sys::FormData::new() else {
                return;
            };
            let _ = form.append_with_blob_and_filename("file", &f, &f.name());
            let _ = form.append_with_str("title", title.get_untracked().trim());
            busy.set(true);
            spawn_local(async move {
                let r =
                    api::upload::<Asset>(&format!("{}/images", base.get_untracked()), &form).await;
                busy.set(false);
                match r {
                    Ok(a) => {
                        reset();
                        on_image.run(a);
                    }
                    Err(e) => error.set(api::message(e)),
                }
            });
            return;
        }
        let body = NewDocument {
            title: title.get_untracked(),
            content: content.get_untracked(),
            doc_type: None,
        };
        busy.set(true);
        spawn_local(async move {
            let r = api::send::<_, Document>(
                "POST",
                &format!("{}/documents", base.get_untracked()),
                &body,
            )
            .await;
            busy.set(false);
            match r {
                Ok(d) => {
                    reset();
                    on_created.run(d.id);
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };

    view! {
        <Modal open=open title="New document" locked=busy>
            <Stack gap="sm">
                <TextInput label="Title" value=title />
                <DropZone
                    label="Or start from a file: markdown, or an image (PNG, JPEG, WebP, GIF, SVG)"
                    accept=".md,.markdown,text/markdown,.png,.jpg,.jpeg,.webp,.gif,.svg,image/*"
                    on_file=pick
                    chosen=Signal::derive(move || {
                        let n = file_name.get();
                        if n.is_empty() { n } else { format!("From {n}") }
                    })
                />
                {move || preview.get().map(|url| view! {
                    <img class="mimir-image-preview" src=url alt=move || title.get() />
                })}
                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                }}
                <Group>
                    <Button
                        disabled=Signal::derive(move || title.get().trim().is_empty() || busy.get())
                        on_click=Callback::new(create)
                    >"Create"</Button>
                    <Button variant="subtle" on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                </Group>
            </Stack>
        </Modal>
    }
}

/// An image, big, with Delete.
#[component]
fn ImageViewer(viewing: RwSignal<Option<Asset>>, on_deleted: Callback<()>) -> impl IntoView {
    let open = RwSignal::new(false);
    let url = RwSignal::new(None::<String>);
    let error = RwSignal::new(String::new());
    Effect::new(move |_| match viewing.get() {
        Some(a) => {
            open.set(true);
            url.set(None);
            error.set(String::new());
            spawn_local(async move {
                match api::image_url(format!("/assets/{}/file", a.id)).await {
                    Ok(u) => {
                        url.try_set(u);
                    }
                    Err(e) => {
                        error.try_set(api::message(e));
                    }
                }
            });
        }
        None => open.set(false),
    });
    Effect::new(move |prev: Option<bool>| {
        let now = open.get();
        if prev == Some(true) && !now {
            viewing.set(None);
        }
        now
    });
    let title = Signal::derive(move || {
        viewing
            .get()
            .map(|a| a.title.unwrap_or(a.filename))
            .unwrap_or_default()
    });
    let remove = move |_| {
        let Some(a) = viewing.get_untracked() else {
            return;
        };
        let ok = web_sys::window()
            .and_then(|w| w.confirm_with_message("Delete this image?").ok())
            .unwrap_or(false);
        if !ok {
            return;
        }
        spawn_local(async move {
            match api::delete(&format!("/assets/{}", a.id)).await {
                Ok(()) => {
                    viewing.set(None);
                    on_deleted.run(());
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    view! {
        <Modal open=open title=title size="lg">
            <Stack gap="sm">
                {move || match url.get() {
                    Some(u) => view! { <img class="mimir-image-view" src=u alt=move || title.get() /> }.into_any(),
                    None => view! { <Loading /> }.into_any(),
                }}
                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                }}
                <Group>
                    <Button bad=true variant="light" on_click=Callback::new(remove)>"Delete image"</Button>
                    <Button variant="subtle" on_click=Callback::new(move |_| open.set(false))>"Close"</Button>
                </Group>
            </Stack>
        </Modal>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn own_documents() {
        assert!(is_own("user_document"));
        assert!(is_own("note"));
        assert!(!is_own("campaign_pitch"));
    }

    #[test]
    fn file_titles() {
        assert_eq!(title_of_file("session-zero_notes.md"), "session zero notes");
        assert_eq!(title_of_file("README"), "README");
    }

    #[test]
    fn save_labels() {
        assert_eq!(Save::Saved.label(), "Saved");
        assert_eq!(Save::Pending.label(), "Saving…");
        assert_eq!(Save::Failed("x".into()).label(), "Not saved: x");
    }
}
