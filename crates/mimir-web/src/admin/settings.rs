//! `/settings` (MIMIR-T-0724): Campaigns (import an archive; export,
//! archive, restore, delete), Books (the catalog's sources; import 5etools
//! data and images, remove a source), About. The section is in the
//! address (`?section=`).

use aurora_leptos::components::*;
use aurora_leptos::frame::{ConfirmDialog, PageHeader, TabItem, Tabs};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use mimir_wire::{ArchiveImported, CampaignSummary, Job, SourceInfo};
use wasm_bindgen::JsCast;

use crate::api;

pub const SECTIONS: [(&str, &str); 3] = [
    ("campaigns", "Campaigns"),
    ("books", "Books"),
    ("about", "About"),
];

/// The section of a `?section=` value; campaigns when absent or unknown.
pub fn section_of(value: Option<&str>) -> &'static str {
    SECTIONS
        .iter()
        .map(|s| s.0)
        .find(|s| Some(*s) == value)
        .unwrap_or("campaigns")
}

/// Wait `ms` milliseconds.
pub async fn sleep(ms: i32) {
    let p = js_sys::Promise::new(&mut |resolve, _| {
        if let Some(w) = web_sys::window() {
            let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(p).await;
}

/// The file picked in an `<input type=file>` change event.
fn picked(ev: &leptos::ev::Event) -> Option<web_sys::File> {
    ev.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .and_then(|i| i.files())
        .and_then(|f| f.get(0))
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let query = use_query_map();
    let current = move || section_of(query.get().get("section").as_deref()).to_string();
    let section = RwSignal::new(current());
    Effect::new(move |_| section.set(current()));
    let tabs = SECTIONS
        .iter()
        .map(|(k, l)| TabItem::new(*k, *l).href(format!("?section={k}")))
        .collect::<Vec<_>>();
    view! {
        <PageHeader title="Settings" />
        <Tabs tabs=tabs value=section label="Settings sections" />
        {move || match section.get().as_str() {
            "books" => view! { <BooksSection /> }.into_any(),
            "about" => view! { <AboutSection /> }.into_any(),
            _ => view! { <CampaignsSection /> }.into_any(),
        }}
    }
}

/// An upload with its progress bar.
#[component]
fn Progress(value: RwSignal<Option<f64>>) -> impl IntoView {
    move || {
        value.get().map(|v| {
            let pct = (v * 100.0).round();
            view! {
                <div class="mimir-progress" role="progressbar" aria-valuenow=pct.to_string() aria-valuemin="0" aria-valuemax="100" aria-label="Upload">
                    <div class="mimir-progress__bar" style=format!("width: {pct}%")></div>
                </div>
                <Text size="xs" dimmed=true>{format!("Uploading: {pct}%")}</Text>
            }
        })
    }
}

#[component]
fn CampaignsSection() -> impl IntoView {
    let version = RwSignal::new(0u32);
    let campaigns = LocalResource::new(move || {
        version.track();
        api::campaigns(true)
    });
    let reload = move || version.update(|v| *v += 1);
    let error = RwSignal::new(String::new());
    let file = StoredValue::new_local(None::<web_sys::File>);
    let file_name = RwSignal::new(String::new());
    let new_name = RwSignal::new(String::new());
    let progress = RwSignal::new(None::<f64>);
    let result = RwSignal::new(None::<ArchiveImported>);
    let deleting = RwSignal::new(None::<CampaignSummary>);
    let confirm_open = RwSignal::new(false);
    Effect::new(move |_| confirm_open.set(deleting.get().is_some()));

    let import = move |_| {
        let Some(f) = file.get_value() else { return };
        let Ok(form) = web_sys::FormData::new() else {
            return;
        };
        let _ = form.append_with_blob_and_filename("file", &f, &f.name());
        let n = new_name.get_untracked();
        if !n.trim().is_empty() {
            let _ = form.append_with_str("name", n.trim());
        }
        progress.set(Some(0.0));
        error.set(String::new());
        spawn_local(async move {
            let r =
                api::upload_with_progress::<ArchiveImported>("/archives/import", &form, move |p| {
                    progress.set(Some(p))
                })
                .await;
            progress.set(None);
            match r {
                Ok(done) => {
                    result.set(Some(done));
                    reload();
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    let set_archived = move |id: String, on: bool| {
        spawn_local(async move {
            let path = format!("/campaigns/{id}/archive");
            let r = if on {
                api::send::<_, CampaignSummary>("PUT", &path, &())
                    .await
                    .map(|_| ())
            } else {
                api::delete_json::<CampaignSummary>(&path).await.map(|_| ())
            };
            match r {
                Ok(()) => reload(),
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    let export = move |c: CampaignSummary| {
        spawn_local(async move {
            if let Err(e) = api::download(
                &format!("/campaigns/{}/export", c.id),
                &format!("{}.mimir-campaign.tar.gz", c.name),
            )
            .await
            {
                error.set(api::message(e));
            }
        });
    };
    let remove = move |_| {
        let Some(c) = deleting.get_untracked() else {
            return;
        };
        spawn_local(async move {
            match api::delete(&format!("/campaigns/{}", c.id)).await {
                Ok(()) => {
                    deleting.set(None);
                    reload();
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };

    view! {
        <section class="mimir-settings__section" aria-label="Import a campaign">
            <SectionLabel label="Import a campaign" />
            <Text size="sm" dimmed=true>"A campaign archive (.mimir-campaign.tar.gz) made by Export, here or in the desktop app."</Text>
            <label class="mimir-file">
                <span>"Archive"</span>
                <input type="file" accept=".gz,.tar.gz,application/gzip" on:change=move |ev| {
                    if let Some(f) = picked(&ev) {
                        file_name.set(f.name());
                        file.set_value(Some(f));
                        result.set(None);
                    }
                } />
            </label>
            <TextInput label="Name (optional; else the archive's)" value=new_name />
            <Group>
                <Button
                    disabled=Signal::derive(move || file_name.get().is_empty() || progress.get().is_some())
                    on_click=Callback::new(import)
                >"Import"</Button>
            </Group>
            <Progress value=progress />
            {move || result.get().map(|r| view! {
                <Alert color=token::OK>
                    {format!("Imported {}: {} modules, {} documents, {} characters, {} maps. ", r.campaign_name, r.counts.modules, r.counts.documents, r.counts.characters, r.counts.maps)}
                    <a href=format!("/campaigns/{}", r.campaign_id)>"Open it"</a>
                </Alert>
            })}
        </section>
        {move || {
            let e = error.get();
            (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
        }}
        <section class="mimir-settings__section" aria-label="Campaigns">
            <SectionLabel label="Campaigns" />
            {move || match campaigns.get() {
                None => view! { <Loading /> }.into_any(),
                Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                Some(Ok(list)) => view! {
                    <Table label="Campaigns" min_width="560px">
                        <thead><tr><th>"Campaign"</th><th>"State"</th><th></th></tr></thead>
                        <tbody>
                            {list.into_iter().map(|c| {
                                let archived = c.archived_at.is_some();
                                let (id, ex, del) = (c.id.clone(), c.clone(), c.clone());
                                view! {
                                    <tr>
                                        <td><a href=format!("/campaigns/{}", c.id)>{c.name.clone()}</a></td>
                                        <td>{if archived { "Archived" } else { "Active" }}</td>
                                        <td class="mimir-row-actions">
                                            <Button variant="subtle" on_click=Callback::new(move |_| export(ex.clone()))>"Export"</Button>
                                            <Button variant="subtle" on_click=Callback::new(move |_| set_archived(id.clone(), !archived))>{if archived { "Restore" } else { "Archive" }}</Button>
                                            {archived.then(|| view! {
                                                <Button variant="subtle" on_click=Callback::new(move |_| deleting.set(Some(del.clone())))>"Delete…"</Button>
                                            })}
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </Table>
                }.into_any(),
            }}
        </section>
        // Made when a campaign is chosen: the name to type is its own.
        {move || deleting.get().map(|c| view! {
            <ConfirmDialog
                open=confirm_open
                title=format!("Delete {}?", c.name)
                message="The campaign, its documents, modules, characters and maps go for good. Export it first to keep a copy."
                confirm_text=c.name.clone()
                confirm_label="Delete"
                on_confirm=Callback::new(remove)
                on_cancel=Callback::new(move |_| deleting.set(None))
            />
        })}
    }
}

/// An upload that starts a job, then the job's message until it ends.
#[component]
fn JobImport(
    #[prop(into)] label: String,
    #[prop(into)] hint: String,
    path: &'static str,
    on_done: Callback<()>,
) -> impl IntoView {
    let file = StoredValue::new_local(None::<web_sys::File>);
    let file_name = RwSignal::new(String::new());
    let progress = RwSignal::new(None::<f64>);
    let job = RwSignal::new(None::<Job>);
    let error = RwSignal::new(String::new());
    let running = Signal::derive(move || {
        progress.get().is_some() || job.get().is_some_and(|j| j.state == "running")
    });
    let start = move |_| {
        let Some(f) = file.get_value() else { return };
        let Ok(form) = web_sys::FormData::new() else {
            return;
        };
        let _ = form.append_with_blob_and_filename("file", &f, &f.name());
        progress.set(Some(0.0));
        error.set(String::new());
        job.set(None);
        spawn_local(async move {
            let r =
                api::upload_with_progress::<Job>(path, &form, move |p| progress.set(Some(p))).await;
            progress.set(None);
            let mut j = match r {
                Ok(j) => j,
                Err(e) => {
                    error.set(api::message(e));
                    return;
                }
            };
            job.set(Some(j.clone()));
            while j.state == "running" {
                sleep(1000).await;
                match api::get::<Job>(&format!("/jobs/{}", j.id)).await {
                    Ok(next) => {
                        j = next;
                        job.set(Some(j.clone()));
                    }
                    Err(e) => {
                        error.set(api::message(e));
                        return;
                    }
                }
            }
            if j.state == "done" {
                on_done.run(());
            }
        });
    };
    let title = label.clone();
    view! {
        <section class="mimir-settings__section" aria-label=title>
            <SectionLabel label=label />
            <Text size="sm" dimmed=true>{hint}</Text>
            <label class="mimir-file">
                <span>"File"</span>
                <input type="file" accept=".gz,.tar.gz,application/gzip" on:change=move |ev| {
                    if let Some(f) = picked(&ev) {
                        file_name.set(f.name());
                        file.set_value(Some(f));
                    }
                } />
            </label>
            <Group>
                <Button disabled=Signal::derive(move || file_name.get().is_empty() || running.get()) on_click=Callback::new(start)>"Import"</Button>
            </Group>
            <Progress value=progress />
            {move || job.get().map(|j| {
                let color = match j.state.as_str() {
                    "done" => token::OK,
                    "failed" => token::BAD,
                    _ => token::ICE,
                };
                view! { <Alert color=color><span role="status">{j.message}</span></Alert> }
            })}
            {move || {
                let e = error.get();
                (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
            }}
        </section>
    }
}

#[component]
fn BooksSection() -> impl IntoView {
    let version = RwSignal::new(0u32);
    let sources = LocalResource::new(move || {
        version.track();
        api::get::<Vec<SourceInfo>>("/catalog/sources")
    });
    let reload = Callback::new(move |_| version.update(|v| *v += 1));
    let removing = RwSignal::new(None::<SourceInfo>);
    let confirm_open = RwSignal::new(false);
    Effect::new(move |_| confirm_open.set(removing.get().is_some()));
    let error = RwSignal::new(String::new());
    let remove = move |_| {
        let Some(s) = removing.get_untracked() else {
            return;
        };
        spawn_local(async move {
            match api::delete(&format!(
                "/catalog/sources/{}",
                js_sys::encode_uri_component(&s.code)
            ))
            .await
            {
                Ok(()) => {
                    removing.set(None);
                    reload.run(());
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    view! {
        <JobImport
            label="Import 5etools data"
            hint="The 5etools data archive (.tar.gz): the core and supplement books, their entries and their text. A full import takes a few minutes."
            path="/catalog/import"
            on_done=reload
        />
        <JobImport
            label="Import 5etools images"
            hint="The 5etools image archive (5etools-img-….tar.gz): pictures for the books, monsters and items."
            path="/catalog/import-images"
            on_done=reload
        />
        {move || {
            let e = error.get();
            (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
        }}
        <section class="mimir-settings__section" aria-label="Sources">
            <SectionLabel label="Sources" />
            {move || match sources.get() {
                None => view! { <Loading /> }.into_any(),
                Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                Some(Ok(list)) => view! {
                    <Table label="Sources" min_width="480px">
                        <thead><tr><th>"Code"</th><th>"Book"</th><th>"Readable"</th><th></th></tr></thead>
                        <tbody>
                            {list.into_iter().map(|s| {
                                let del = s.clone();
                                let code = s.code.clone();
                                view! {
                                    <tr>
                                        <td>{s.code.clone()}</td>
                                        <td>{s.name.clone()}</td>
                                        <td>{if s.book { "Yes" } else { "—" }}</td>
                                        <td class="mimir-row-actions">
                                            <ActionIcon title=format!("Remove {code}") on_click=Callback::new(move |_| removing.set(Some(del.clone())))>"×"</ActionIcon>
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </Table>
                }.into_any(),
            }}
        </section>
        <ConfirmDialog
            open=confirm_open
            title=Signal::derive(move || format!("Remove {}?", removing.get().map(|s| s.name).unwrap_or_default()))
            message="Its catalog entries and its book text go; characters and campaigns keep what they hold. Import it again to bring it back."
            confirm_label="Remove"
            on_confirm=Callback::new(remove)
            on_cancel=Callback::new(move |_| removing.set(None))
        />
    }
}

#[component]
fn AboutSection() -> impl IntoView {
    let config = LocalResource::new(api::config);
    view! {
        <section class="mimir-settings__section" aria-label="About">
            {move || match config.get() {
                None => view! { <Loading /> }.into_any(),
                Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                Some(Ok(c)) => view! {
                    <dl class="mimir-sheet__profs">
                        <dt>"Mimir"</dt><dd>{format!("Version {}", c.version)}</dd>
                        <dt>"Sign-in"</dt><dd>{format!("{:?}", c.auth)}</dd>
                    </dl>
                }.into_any(),
            }}
        </section>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sections() {
        assert_eq!(super::section_of(None), "campaigns");
        assert_eq!(super::section_of(Some("books")), "books");
        assert_eq!(super::section_of(Some("x")), "campaigns");
    }
}
