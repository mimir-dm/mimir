//! A campaign as a 5etools homebrew adventure (MIMIR-T-0740): the DM names
//! the adventure's source, then the browser downloads the file.

use aurora_leptos::components::*;
use aurora_leptos::frame::Modal;
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;

/// The query of `/campaigns/{id}/adventure-export`.
#[allow(clippy::too_many_arguments)]
pub fn export_query(
    source: &str,
    name: &str,
    authors: &str,
    version: &str,
    storyline: &str,
    from: &str,
    to: &str,
    url: &str,
) -> String {
    let enc = |s: &str| -> String { js_sys::encode_uri_component(s.trim()).into() };
    let mut q = format!(
        "source={}&name={}&authors={}&version={}&storyline={}",
        enc(source),
        enc(name),
        enc(authors),
        enc(version),
        enc(storyline)
    );
    if let Ok(n) = from.trim().parse::<i64>() {
        q += &format!("&level_start={n}");
    }
    if let Ok(n) = to.trim().parse::<i64>() {
        q += &format!("&level_end={n}");
    }
    if !url.trim().is_empty() {
        q += &format!("&url={}", enc(url));
    }
    q
}

#[component]
pub fn AdventureExportDialog(
    open: RwSignal<bool>,
    #[prop(into)] campaign_id: String,
    #[prop(into)] campaign_name: String,
) -> impl IntoView {
    let id = StoredValue::new(campaign_id);
    let source = RwSignal::new(String::new());
    let name = RwSignal::new(campaign_name);
    let authors = RwSignal::new(String::new());
    let version = RwSignal::new("1.0.0".to_string());
    let storyline = RwSignal::new("Homebrew".to_string());
    let from = RwSignal::new("1".to_string());
    let to = RwSignal::new("1".to_string());
    let url = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    let download = move |_| {
        let q = export_query(
            &source.get_untracked(),
            &name.get_untracked(),
            &authors.get_untracked(),
            &version.get_untracked(),
            &storyline.get_untracked(),
            &from.get_untracked(),
            &to.get_untracked(),
            &url.get_untracked(),
        );
        let file = format!("{}.json", source.get_untracked().trim());
        let path = format!("/campaigns/{}/adventure-export?{q}", id.get_value());
        busy.set(true);
        error.set(String::new());
        spawn_local(async move {
            match api::download(&path, &file).await {
                Ok(()) => open.set(false),
                Err(e) => error.set(api::message(e)),
            }
            busy.set(false);
        });
    };
    view! {
        <Modal open=open title="Export as a 5etools adventure" locked=busy>
            <Stack gap="sm">
                <Text size="sm" dimmed=true>"A homebrew file that 5etools and Mimir read: the campaign's documents, its modules as chapters, its homebrew, and a copy of every monster, item and spell the text names, under the adventure's own source."</Text>
                <TextInput label="Source code" value=source />
                <Text size="xs" dimmed=true>"6 or more letters, digits or - & + !, not an official source: for example LowFen."</Text>
                <TextInput label="Adventure name" value=name />
                <TextInput label="Authors" value=authors />
                <Group>
                    <TextInput label="Version" value=version />
                    <TextInput label="Storyline" value=storyline />
                </Group>
                <Group>
                    <TextInput label="From level" value=from />
                    <TextInput label="To level" value=to />
                </Group>
                <TextInput label="Web address" value=url />
                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                }}
                <Group>
                    <Button
                        disabled=Signal::derive(move || source.get().trim().len() < 6 || name.get().trim().is_empty() || busy.get())
                        on_click=Callback::new(download)
                    >"Download"</Button>
                    <Button variant="subtle" on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                </Group>
            </Stack>
        </Modal>
    }
}
