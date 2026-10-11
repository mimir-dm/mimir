//! `/settings` (MIMIR-T-0724): Campaigns (import an archive; export,
//! archive, restore, delete), Books (the catalog's sources; import 5etools
//! data and images, remove a source), About. The section is in the
//! address (`?section=`).

use aurora_leptos::components::*;
use aurora_leptos::data::CodeBlock;
use aurora_leptos::frame::{ConfirmDialog, PageHeader, TabItem, Tabs};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use mimir_wire::{ArchiveImported, ArchivePreview, CampaignSummary, DevFixture, Job, SourceInfo};

use crate::api;
use crate::components::DropZone;

pub const SECTIONS: [(&str, &str); 4] = [
    ("campaigns", "Campaigns"),
    ("books", "Books"),
    ("integrations", "Integrations"),
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
            "integrations" => view! { <IntegrationsSection /> }.into_any(),
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
    let deleting = RwSignal::new(None::<CampaignSummary>);
    let confirm_open = RwSignal::new(false);
    Effect::new(move |_| confirm_open.set(deleting.get().is_some()));

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
            <ImportArchive on_imported=Callback::new(move |_| reload()) />
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

/// Import a campaign archive (MIMIR-T-0731): drop or choose the file, see
/// what it holds (`/archives/preview`), name it, import it. Settings and
/// Home use it.
#[component]
pub fn ImportArchive(on_imported: Callback<ArchiveImported>) -> impl IntoView {
    let file = StoredValue::new_local(None::<web_sys::File>);
    let file_name = RwSignal::new(String::new());
    let new_name = RwSignal::new(String::new());
    let progress = RwSignal::new(None::<f64>);
    let preview = RwSignal::new(None::<ArchivePreview>);
    let all_refs = RwSignal::new(false);
    let result = RwSignal::new(None::<ArchiveImported>);
    let error = RwSignal::new(String::new());
    let form_of = move |with_name: bool| {
        let f = file.get_value()?;
        let form = web_sys::FormData::new().ok()?;
        let _ = form.append_with_blob_and_filename("file", &f, &f.name());
        let n = new_name.get_untracked();
        if with_name && !n.trim().is_empty() {
            let _ = form.append_with_str("name", n.trim());
        }
        Some(form)
    };
    // A chosen file: read its manifest first.
    let chose = Callback::new(move |f: web_sys::File| {
        file_name.set(f.name());
        file.set_value(Some(f));
        preview.set(None);
        result.set(None);
        error.set(String::new());
        all_refs.set(false);
        let Some(form) = form_of(false) else { return };
        progress.set(Some(0.0));
        spawn_local(async move {
            let r =
                api::upload_with_progress::<ArchivePreview>("/archives/preview", &form, move |p| {
                    progress.set(Some(p))
                })
                .await;
            progress.set(None);
            match r {
                Ok(p) => {
                    new_name.set(p.campaign_name.clone());
                    preview.set(Some(p));
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    });
    let import = move |_| {
        let Some(form) = form_of(true) else { return };
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
                    preview.set(None);
                    result.set(Some(done.clone()));
                    on_imported.run(done);
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    view! {
        <Stack gap="sm">
            <Text size="sm" dimmed=true>"A campaign archive (.mimir-campaign.tar.gz) made by Export."</Text>
            <DropZone label="Archive" accept=".gz,.tar.gz,application/gzip" on_file=chose chosen=file_name />
            <Progress value=progress />
            {move || preview.get().map(|p| {
                let c = &p.counts;
                let counts = format!(
                    "{} modules, {} documents, {} characters, {} maps, {} images and files, {} homebrew",
                    c.modules, c.documents, c.characters, c.maps, c.assets,
                    c.homebrew_items + c.homebrew_monsters + c.homebrew_spells
                );
                let refs = p.catalog_references.clone();
                let n = refs.len();
                view! {
                    <div class="mimir-archive-preview" aria-label="In the archive">
                        <Text bold=true>{p.campaign_name.clone()}</Text>
                        <Text size="sm">{counts}</Text>
                        <Text size="xs" dimmed=true>{format!("Made by Mimir {} on {}", p.mimir_version, p.created_at.get(..10).unwrap_or(&p.created_at))}</Text>
                        {(n > 0).then(|| view! {
                            <SectionLabel label="Catalog entries it names" count=n />
                            <ul class="mimir-archive-preview__refs">
                                {move || {
                                    let shown = if all_refs.get() { n } else { n.min(8) };
                                    refs.iter().take(shown).map(|r| view! { <li>{r.clone()}</li> }).collect_view()
                                }}
                            </ul>
                            {move || (n > 8 && !all_refs.get()).then(|| view! {
                                <Button variant="subtle" on_click=Callback::new(move |_| all_refs.set(true))>{format!("Show all {n}")}</Button>
                            })}
                        })}
                    </div>
                    <TextInput label="Name of the new campaign" value=new_name />
                    <Group>
                        <Button disabled=Signal::derive(move || progress.get().is_some()) on_click=Callback::new(import)>"Import"</Button>
                    </Group>
                }
            })}
            {move || result.get().map(|r| view! {
                <Alert color=token::OK>
                    {format!("Imported {}: {} modules, {} documents, {} characters, {} maps. ", r.campaign_name, r.counts.modules, r.counts.documents, r.counts.characters, r.counts.maps)}
                    <a href=format!("/campaigns/{}", r.campaign_id)>"Open it"</a>
                </Alert>
            })}
            {move || {
                let e = error.get();
                (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
            }}
        </Stack>
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
            <DropZone
                label="File"
                accept=".gz,.tar.gz,application/gzip"
                chosen=file_name
                on_file=Callback::new(move |f: web_sys::File| {
                    file_name.set(f.name());
                    file.set_value(Some(f));
                })
            />
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
    // The books to remove: one (×) or the ticked ones (MIMIR-T-0734).
    let removing = RwSignal::new(Vec::<SourceInfo>::new());
    let ticked = RwSignal::new(Vec::<String>::new());
    let confirm_open = RwSignal::new(false);
    Effect::new(move |_| confirm_open.set(!removing.with(Vec::is_empty)));
    let error = RwSignal::new(String::new());
    let remove = move |_| {
        let list = removing.get_untracked();
        spawn_local(async move {
            for s in list {
                if let Err(e) = api::delete(&format!(
                    "/catalog/sources/{}",
                    js_sys::encode_uri_component(&s.code)
                ))
                .await
                {
                    error.set(api::message(e));
                    break;
                }
            }
            removing.set(Vec::new());
            ticked.set(Vec::new());
            reload.run(());
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
                Some(Ok(list)) => {
                    let all = StoredValue::new(list.clone());
                    view! {
                    <Group>
                        <Button
                            variant="light"
                            bad=true
                            disabled=Signal::derive(move || ticked.with(Vec::is_empty))
                            on_click=Callback::new(move |_| {
                                let t = ticked.get_untracked();
                                removing.set(all.with_value(|l| l.iter().filter(|s| t.contains(&s.code)).cloned().collect()));
                            })
                        >
                            {move || format!("Remove selected ({})", ticked.with(Vec::len))}
                        </Button>
                    </Group>
                    <Table label="Sources" min_width="480px">
                        <thead><tr><th><span class="mimir-visually-hidden">"Select"</span></th><th>"Code"</th><th>"Book"</th><th>"Readable"</th><th></th></tr></thead>
                        <tbody>
                            {list.into_iter().map(|s| {
                                let del = s.clone();
                                let code = s.code.clone();
                                let (c1, c2) = (s.code.clone(), s.code.clone());
                                view! {
                                    <tr>
                                        <td>
                                            <input
                                                type="checkbox"
                                                aria-label=format!("Select {}", s.code)
                                                prop:checked=move || ticked.with(|t| t.contains(&c1))
                                                on:change=move |_| ticked.update(|t| {
                                                    if let Some(i) = t.iter().position(|x| *x == c2) { t.remove(i); } else { t.push(c2.clone()); }
                                                })
                                            />
                                        </td>
                                        <td>{s.code.clone()}</td>
                                        <td>{s.name.clone()}</td>
                                        <td>{if s.book { "Yes" } else { "—" }}</td>
                                        <td class="mimir-row-actions">
                                            <ActionIcon title=format!("Remove {code}") on_click=Callback::new(move |_| removing.set(vec![del.clone()]))>"×"</ActionIcon>
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </Table>
                    }.into_any()
                }
            }}
        </section>
        <ConfirmDialog
            open=confirm_open
            title=Signal::derive(move || removing.with(|l| remove_title(l)))
            message="Its catalog entries and its book text go; characters and campaigns keep what they hold. Import it again to bring it back."
            confirm_label="Remove"
            on_confirm=Callback::new(remove)
            on_cancel=Callback::new(move |_| removing.set(Vec::new()))
        />
    }
}

/// "Remove Monster Manual?", or "Remove 3 books?".
pub fn remove_title(list: &[SourceInfo]) -> String {
    match list {
        [one] => format!("Remove {}?", one.name),
        many => format!("Remove {} books?", many.len()),
    }
}

/// The command that adds Mimir's MCP endpoint to Claude Code.
pub fn claude_command(origin: &str, open: bool) -> String {
    let base = format!("claude mcp add --transport http mimir {origin}/mcp");
    if open {
        base
    } else {
        format!("{base} \\\n  --header \"Authorization: Bearer <your DM token>\"")
    }
}

/// Claude and other MCP clients (MIMIR-T-0725): the endpoint and how to
/// add it. The token is not shown: it is the server's `MIMIR_API_TOKEN`.
#[component]
fn IntegrationsSection() -> impl IntoView {
    let config = LocalResource::new(api::config);
    let origin = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default();
    let endpoint = format!("{origin}/mcp");
    view! {
        <section class="mimir-settings__section" aria-label="Claude">
            <SectionLabel label="Claude (MCP)" />
            <Text size="sm">"Claude reaches your campaigns through the server's MCP endpoint. Only the DM token works there; a player link is refused."</Text>
            <dl class="mimir-sheet__profs">
                <dt>"Endpoint"</dt><dd><code>{endpoint}</code></dd>
            </dl>
            {move || config.get().and_then(Result::ok).map(|c| {
                let open = c.auth == mimir_wire::AuthMode::Open;
                let cmd = claude_command(&origin, open);
                view! {
                    <Text size="sm">"In a terminal, add Mimir to Claude Code:"</Text>
                    <CodeBlock code=cmd label="Claude Code" wrap=true />
                    <Text size="sm">"Or install the plugin (slash commands and skills too); set MIMIR_URL to this address and MIMIR_API_TOKEN to the DM token first:"</Text>
                    <CodeBlock code="/plugin marketplace add mimir-dm/mimir\n/plugin install mimir-dm@mimir".to_string() label="Plugin" />
                    {(!open).then(|| view! {
                        <Text size="sm" dimmed=true>"The DM token is the server's MIMIR_API_TOKEN (the one you sign in with)."</Text>
                    })}
                }
            })}
        </section>
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
                    {c.dev_tools.then(|| view! { <DevTools /> })}
                }.into_any(),
            }}
        </section>
    }
}

/// Dev tools (MIMIR-T-0734): on a development server with the fixture,
/// seed, reseed or clear the fixture campaign (the desktop's dev tools).
#[component]
fn DevTools() -> impl IntoView {
    let state = RwSignal::new(None::<bool>);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    spawn_local(async move {
        match api::get::<DevFixture>("/dev/fixture").await {
            Ok(f) => state.set(Some(f.seeded)),
            Err(e) => error.set(api::message(e)),
        }
    });
    let act = move |action: &'static str| {
        busy.set(true);
        error.set(String::new());
        spawn_local(async move {
            let r =
                api::send::<_, DevFixture>("POST", &format!("/dev/fixture/{action}"), &()).await;
            busy.set(false);
            match r {
                Ok(f) => state.set(Some(f.seeded)),
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    view! {
        <section class="mimir-settings__section" aria-label="Dev tools">
            <SectionLabel label="Dev tools" />
            <Text size="sm" dimmed=true>"A development server: the fixture campaign, for tests and screenshots. Never real data."</Text>
            <p class="cl-text cl-text--sm" role="status">
                {move || match state.get() {
                    None => "…".to_string(),
                    Some(true) => format!("{} is in.", TEST_CAMPAIGN),
                    Some(false) => format!("{} is not in.", TEST_CAMPAIGN),
                }}
            </p>
            <Group>
                <Button variant="light" disabled=Signal::derive(move || busy.get() || state.get() != Some(false)) on_click=Callback::new(move |_| act("seed"))>"Seed"</Button>
                <Button variant="light" disabled=busy on_click=Callback::new(move |_| act("reseed"))>"Reseed"</Button>
                <Button variant="light" bad=true disabled=Signal::derive(move || busy.get() || state.get() != Some(true)) on_click=Callback::new(move |_| act("clear"))>"Clear"</Button>
            </Group>
            {move || {
                let e = error.get();
                (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
            }}
        </section>
    }
}

/// The fixture campaign's name (mimir-core's `TEST_CAMPAIGN_NAME`).
const TEST_CAMPAIGN: &str = "The Lost Mine of Phandelver";

#[cfg(test)]
mod tests {
    #[test]
    fn claude_commands() {
        assert_eq!(
            super::claude_command("http://x", true),
            "claude mcp add --transport http mimir http://x/mcp"
        );
        assert!(super::claude_command("http://x", false)
            .contains("Authorization: Bearer <your DM token>"));
    }

    #[test]
    fn removal_titles() {
        let s = |n: &str| mimir_wire::SourceInfo {
            code: n.into(),
            name: n.into(),
            book: false,
        };
        assert_eq!(
            super::remove_title(&[s("Monster Manual")]),
            "Remove Monster Manual?"
        );
        assert_eq!(super::remove_title(&[s("A"), s("B")]), "Remove 2 books?");
    }

    #[test]
    fn sections() {
        assert_eq!(super::section_of(None), "campaigns");
        assert_eq!(super::section_of(Some("books")), "books");
        assert_eq!(super::section_of(Some("x")), "campaigns");
    }
}
