//! Maps of a campaign or a module (MIMIR-T-0720): the list, an upload (a
//! UVTT file, or an image with its grid size), delete. The map opens on
//! the DM map page, where tokens, fog, lights and markers are set up.

use aurora_leptos::components::*;
use aurora_leptos::data::SectionLabel;
use aurora_leptos::frame::{ConfirmDialog, Modal};
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_wire::{MapSummary, LIGHTING_MODES};
use wasm_bindgen::JsCast;

use crate::api;

/// Whether a file is an image (it needs a grid size), not a UVTT file.
pub fn is_image(name: &str) -> bool {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp")
}

/// A map name from a file name: no extension, words in title case
/// ("old-owl_well.png" → "Old Owl Well"), as the desktop app.
pub fn map_name(file: &str) -> String {
    let stem = file.rsplit_once('.').map(|(s, _)| s).unwrap_or(file);
    stem.split(['-', '_', ' '])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().chain(c).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `campaign_id`, and `module_id` for a module's maps (none: the
/// campaign-level maps).
#[component]
pub fn MapsPanel(
    #[prop(into)] campaign_id: Signal<String>,
    #[prop(optional, into)] module_id: Option<Signal<String>>,
) -> impl IntoView {
    let version = RwSignal::new(0u32);
    let maps = LocalResource::new(move || {
        version.track();
        let path = match module_id {
            Some(m) => format!("/modules/{}/maps", m.get()),
            None => format!("/campaigns/{}/maps", campaign_id.get()),
        };
        async move {
            let list = api::get::<Vec<MapSummary>>(path).await?;
            // The campaign's list holds every map; keep its own level.
            Ok::<_, aurora_leptos::tokens::ApiError>(if module_id.is_none() {
                list.into_iter().filter(|m| m.module_id.is_none()).collect()
            } else {
                list
            })
        }
    });
    let uploading = RwSignal::new(false);
    let removing = RwSignal::new(None::<MapSummary>);
    let confirm_open = RwSignal::new(false);
    Effect::new(move |_| confirm_open.set(removing.get().is_some()));
    let error = RwSignal::new(String::new());
    let remove = move |_| {
        let Some(m) = removing.get_untracked() else {
            return;
        };
        spawn_local(async move {
            match api::delete(&format!("/maps/{}", m.id)).await {
                Ok(()) => {
                    removing.set(None);
                    version.update(|v| *v += 1);
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };

    view! {
        <div class="mimir-maps">
            <Group>
                <Button variant="light" on_click=Callback::new(move |_| uploading.set(true))>"Upload a map"</Button>
            </Group>
            {move || match maps.get() {
                None => view! { <Loading /> }.into_any(),
                Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                Some(Ok(list)) => {
                    let n = list.len();
                    view! {
                        <SectionLabel label="Maps" count=n />
                        <Table label="Maps" min_width="480px">
                            <thead><tr><th>"Map"</th><th>"Lighting"</th><th>"Fog of war"</th><th></th></tr></thead>
                            <tbody>
                                {if list.is_empty() {
                                    view! { <TableEmpty message="No maps yet." colspan=4 /> }.into_any()
                                } else {
                                    list.into_iter().map(|m| {
                                        let del = m.clone();
                                        let name = m.name.clone();
                                        view! {
                                            <tr>
                                                <td><a href=format!("/maps/{}", m.id)>{m.name}</a></td>
                                                <td>{m.lighting_mode}</td>
                                                <td>{if m.fog_enabled { "On" } else { "Off" }}</td>
                                                <td class="mimir-row-actions">
                                                    <ActionIcon title=format!("Delete {name}") on_click=Callback::new(move |_| removing.set(Some(del.clone())))>"×"</ActionIcon>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view().into_any()
                                }}
                            </tbody>
                        </Table>
                    }.into_any()
                }
            }}
            {move || {
                let e = error.get();
                (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
            }}
        </div>
        <UploadDialog
            open=uploading
            campaign_id=campaign_id
            module_id=module_id
            on_done=Callback::new(move |_| version.update(|v| *v += 1))
        />
        <ConfirmDialog
            open=confirm_open
            title=Signal::derive(move || format!("Delete {}?", removing.get().map(|m| m.name).unwrap_or_default()))
            message="Its tokens, lights, traps and points of interest go with it."
            confirm_label="Delete"
            on_confirm=Callback::new(remove)
            on_cancel=Callback::new(move |_| removing.set(None))
        />
    }
}

#[component]
fn UploadDialog(
    open: RwSignal<bool>,
    campaign_id: Signal<String>,
    module_id: Option<Signal<String>>,
    on_done: Callback<()>,
) -> impl IntoView {
    let file = StoredValue::new_local(None::<web_sys::File>);
    let file_name = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    let grid = RwSignal::new(70.0);
    let lighting = RwSignal::new("bright".to_string());
    let error = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let pick = move |ev: leptos::ev::Event| {
        let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        else {
            return;
        };
        let Some(f) = input.files().and_then(|l| l.get(0)) else {
            return;
        };
        let n = f.name();
        if name.get_untracked().trim().is_empty() {
            name.set(map_name(&n));
        }
        file_name.set(n);
        file.set_value(Some(f));
        error.set(String::new());
    };
    let send = move |_| {
        let Some(f) = file.get_value() else {
            error.set("Pick a file.".into());
            return;
        };
        let Ok(form) = web_sys::FormData::new() else {
            return;
        };
        let _ = form.append_with_str("name", name.get_untracked().trim());
        let _ = form.append_with_str("lighting_mode", &lighting.get_untracked());
        if is_image(&f.name()) {
            let _ = form.append_with_str("grid_px", &grid.get_untracked().to_string());
        }
        if let Some(m) = module_id {
            let _ = form.append_with_str("module_id", &m.get_untracked());
        }
        let _ = form.append_with_blob_and_filename("file", &f, &f.name());
        busy.set(true);
        spawn_local(async move {
            let path = format!("/campaigns/{}/maps", campaign_id.get_untracked());
            let r = api::upload::<MapSummary>(&path, &form).await;
            busy.set(false);
            match r {
                Ok(_) => {
                    open.set(false);
                    file.set_value(None);
                    file_name.set(String::new());
                    name.set(String::new());
                    on_done.run(());
                }
                Err(e) => error.set(api::message(e)),
            }
        });
    };
    let modes = StoredValue::new(
        LIGHTING_MODES
            .iter()
            .map(|(k, n)| (k.to_string(), n.to_string()))
            .collect::<Vec<_>>(),
    );
    view! {
        <Modal open=open title="Upload a map" locked=busy>
            <Stack gap="sm">
                <label class="mimir-file">
                    <span>"A UVTT file (.dd2vtt, .uvtt) or an image (PNG, JPEG, WebP)"</span>
                    <input type="file" accept=".dd2vtt,.uvtt,image/png,image/jpeg,image/webp" on:change=pick />
                </label>
                <TextInput label="Name" value=name />
                {move || is_image(&file_name.get()).then(|| view! {
                    <NumberInput label="Grid: pixels a square" value=grid min=10.0 max=1000.0 />
                })}
                <Select label="Lighting" option_pairs=modes.get_value() value=lighting />
                {move || busy.get().then(|| view! { <Text size="sm" dimmed=true>"Uploading…"</Text> })}
                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                }}
                <Group>
                    <Button
                        disabled=Signal::derive(move || file_name.get().is_empty() || busy.get())
                        on_click=Callback::new(send)
                    >"Upload"</Button>
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
    fn map_names() {
        assert_eq!(map_name("old-owl_well.png"), "Old Owl Well");
        assert_eq!(map_name("cragmaw castle.dd2vtt"), "Cragmaw Castle");
    }

    #[test]
    fn images_need_a_grid() {
        assert!(is_image("cave.PNG"));
        assert!(is_image("cave.webp"));
        assert!(!is_image("cave.dd2vtt"));
    }
}
