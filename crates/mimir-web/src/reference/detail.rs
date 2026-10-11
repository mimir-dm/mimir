//! The detail drawer of a catalog entry (MIMIR-T-0723), shared by the
//! catalog, the reader and the module page (its inspector). A stack:
//! a cross-reference in a detail opens over it; Back returns.

use std::cell::RefCell;
use std::collections::HashMap;

use aurora_leptos::components::*;
use aurora_leptos::frame::Drawer;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_render::{detail, summary};
use serde_json::Value;
use wasm_bindgen::JsCast;

use crate::api;

/// An entry to show.
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub category: String,
    pub source: String,
    pub name: String,
}

impl Target {
    pub fn new(category: &str, source: &str, name: &str) -> Self {
        Target {
            category: category.into(),
            source: source.into(),
            name: name.into(),
        }
    }

    fn path(&self) -> String {
        format!(
            "/catalog/entry/{}/{}/{}",
            self.category,
            enc(&self.source),
            enc(&self.name)
        )
    }
}

fn enc(s: &str) -> String {
    js_sys::encode_uri_component(s).into()
}

/// The open details (context of the DM shell).
#[derive(Clone, Copy)]
pub struct Reference {
    stack: RwSignal<Vec<Target>>,
}

impl Reference {
    pub fn provide() -> Self {
        let r = Reference {
            stack: RwSignal::new(Vec::new()),
        };
        provide_context(r);
        r
    }

    pub fn get() -> Option<Self> {
        use_context::<Reference>()
    }

    /// Show one entry (the stack starts again).
    pub fn open(&self, t: Target) {
        self.stack.set(vec![t]);
    }

    /// Show an entry over the current one.
    pub fn push(&self, t: Target) {
        self.stack.update(|s| s.push(t));
    }
}

/// The cross-reference target of a link in rendered HTML.
pub fn target_of(el: &web_sys::Element) -> Option<Target> {
    let link = el.closest("a.cross-ref-link").ok().flatten()?;
    let kind = link.get_attribute("data-ref-type")?;
    let category = summary::category_of(&kind)?;
    Some(Target::new(
        category,
        &link.get_attribute("data-ref-source").unwrap_or_default(),
        &link.get_attribute("data-ref-name").unwrap_or_default(),
    ))
}

fn event_element(ev: &web_sys::Event) -> Option<web_sys::Element> {
    ev.target()?.dyn_into::<web_sys::Element>().ok()
}

/// A click in rendered content: a cross-reference opens its detail.
pub fn on_content_click(ev: web_sys::MouseEvent, r: Option<Reference>) {
    let Some(t) = event_element(&ev).and_then(|e| target_of(&e)) else {
        return;
    };
    ev.prevent_default();
    if let Some(r) = r {
        r.push(t);
    }
}

thread_local! {
    static TOOLTIPS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// A pointer over a cross-reference: its one-line summary as the link's
/// title (fetched once).
pub fn on_content_hover(ev: web_sys::MouseEvent) {
    let Some(el) = event_element(&ev) else { return };
    let Some(link) = el.closest("a.cross-ref-link").ok().flatten() else {
        return;
    };
    if link.has_attribute("title") {
        return;
    }
    let Some(t) = target_of(&link) else { return };
    let key = t.path();
    if let Some(tip) = TOOLTIPS.with(|c| c.borrow().get(&key).cloned()) {
        let _ = link.set_attribute("title", &tip);
        return;
    }
    let _ = link.set_attribute("title", &t.name);
    spawn_local(async move {
        if let Ok(v) = api::get::<Value>(&key).await {
            let tip = detail::tooltip(&t.category, &v);
            let _ = link.set_attribute("title", &tip);
            TOOLTIPS.with(|c| c.borrow_mut().insert(key, tip));
        }
    });
}

/// Load the image placeholders (`.image-wrapper[data-image-path]`) under
/// a node, with the token.
pub fn load_images(root: &web_sys::Element) {
    let Ok(list) = root.query_selector_all(".image-wrapper[data-image-path]") else {
        return;
    };
    for i in 0..list.length() {
        let Some(el) = list
            .get(i)
            .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
        else {
            continue;
        };
        if el.get_attribute("data-loaded").is_some() {
            continue;
        }
        let _ = el.set_attribute("data-loaded", "1");
        let path = el.get_attribute("data-image-path").unwrap_or_default();
        spawn_local(async move {
            let url = path.split('/').map(enc).collect::<Vec<_>>().join("/");
            match api::image_url(format!("/book-images/{url}")).await {
                Ok(Some(src)) => {
                    let alt = path.rsplit('/').next().unwrap_or("");
                    el.set_inner_html(&format!(
                        r#"<img src="{src}" alt="{}" loading="lazy">"#,
                        mimir_render::esc(alt)
                    ));
                }
                // Not in this catalog's images: no gap left.
                _ => el.remove(),
            }
        });
    }
}

/// HTML from mimir-render, with cross-references, tooltips and images.
#[component]
pub fn Rendered(
    #[prop(into)] html: Signal<String>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let node = NodeRef::<Div>::new();
    let r = Reference::get();
    Effect::new(move |_| {
        html.track();
        if let Some(el) = node.get() {
            // After the new HTML is in.
            request_animation_frame(move || load_images(&el));
        }
    });
    view! {
        <div
            class=format!("mimir-prose mimir-rendered {class}")
            node_ref=node
            inner_html=move || html.get()
            on:click=move |ev| on_content_click(ev, r)
            on:mouseover=on_content_hover
        ></div>
    }
}

/// The drawer: the top of the stack.
#[component]
pub fn DetailDrawer() -> impl IntoView {
    let Some(r) = Reference::get() else {
        return ().into_any();
    };
    let open = RwSignal::new(false);
    Effect::new(move |_| open.set(!r.stack.with(Vec::is_empty)));
    Effect::new(move |prev: Option<bool>| {
        let now = open.get();
        if prev == Some(true) && !now {
            r.stack.set(Vec::new());
        }
        now
    });
    let top = Memo::new(move |_| r.stack.with(|s| s.last().cloned()));
    let entry = LocalResource::new(move || {
        let t = top.get();
        async move {
            match t {
                Some(t) => api::get::<Value>(&t.path()).await.map(|v| Some((t, v))),
                None => Ok(None),
            }
        }
    });
    let title = Signal::derive(move || top.get().map(|t| t.name).unwrap_or_default());
    view! {
        <Drawer open=open title=title size="lg">
            {move || (r.stack.with(Vec::len) > 1).then(|| view! {
                <Button variant="subtle" on_click=Callback::new(move |_| r.stack.update(|s| { s.pop(); }))>"← Back"</Button>
            })}
            {move || match entry.get() {
                None => view! { <Loading /> }.into_any(),
                Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                Some(Ok(None)) => ().into_any(),
                Some(Ok(Some((t, v)))) => {
                    let html = detail::detail(&t.category, &v);
                    view! { <Rendered html=html class="mimir-detail" /> }.into_any()
                }
            }}
        </Drawer>
    }
    .into_any()
}
