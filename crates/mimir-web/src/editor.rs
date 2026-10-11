//! The rich editor (MIMIR-T-0719): TipTap in the browser, driven from
//! Leptos. `editor/src/index.js` is bundled by esbuild (a Trunk hook) and
//! sets `window.MimirEditor`; this module binds its `Handle` with
//! wasm-bindgen. Markdown in, markdown out. Without the bundle (a host
//! test, a failed load) the editor falls back to a plain text area.

use aurora_leptos::components::Textarea;
use leptos::html::Div;
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /// `window.MimirEditor.Handle`.
    #[wasm_bindgen(js_namespace = MimirEditor)]
    type Handle;

    #[wasm_bindgen(constructor, js_namespace = MimirEditor, catch)]
    fn new(
        el: &web_sys::Element,
        markdown: &str,
        placeholder: &str,
        label: &str,
        on_change: &js_sys::Function,
        on_state: &js_sys::Function,
    ) -> Result<Handle, JsValue>;

    #[wasm_bindgen(method, js_name = getMarkdown)]
    fn get_markdown(this: &Handle) -> String;

    #[wasm_bindgen(method, js_name = setMarkdown)]
    fn set_markdown(this: &Handle, markdown: &str);

    #[wasm_bindgen(method)]
    fn run(this: &Handle, name: &str, args: &str) -> bool;

    #[wasm_bindgen(method)]
    fn can(this: &Handle, name: &str, args: &str) -> bool;

    #[wasm_bindgen(method, js_name = isActive)]
    fn is_active(this: &Handle, name: &str, args: &str) -> bool;

    #[wasm_bindgen(method)]
    fn destroy(this: &Handle);
}

/// Whether the bundle is loaded.
fn available() -> bool {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("MimirEditor")).ok())
        .is_some_and(|v| !v.is_undefined())
}

/// A toolbar button: the command, its arguments (JSON), the label, the
/// title (also its accessible name).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tool {
    pub command: &'static str,
    pub args: &'static str,
    pub label: &'static str,
    pub title: &'static str,
}

const fn tool(
    command: &'static str,
    args: &'static str,
    label: &'static str,
    title: &'static str,
) -> Tool {
    Tool {
        command,
        args,
        label,
        title,
    }
}

/// The toolbar, in groups: the same features as the desktop editor.
pub const TOOLBAR: [&[Tool]; 6] = [
    &[
        tool("heading", r#"{"level":1}"#, "H1", "Heading 1"),
        tool("heading", r#"{"level":2}"#, "H2", "Heading 2"),
        tool("heading", r#"{"level":3}"#, "H3", "Heading 3"),
    ],
    &[
        tool("bold", "", "B", "Bold"),
        tool("italic", "", "I", "Italic"),
        tool("strike", "", "S", "Strikethrough"),
    ],
    &[
        tool("bulletList", "", "• List", "Bullet list"),
        tool("orderedList", "", "1. List", "Numbered list"),
        tool("blockquote", "", "Quote", "Quote"),
        tool("rule", "", "Rule", "Horizontal rule"),
    ],
    // The adventure boxes (MIMIR-T-0737).
    &[
        tool("readAloud", "", "Read aloud", "Insert a read-aloud box"),
        tool("sidebar", "", "Sidebar", "Insert a sidebar"),
    ],
    &[
        tool("insertTable", "", "+ Table", "Insert a table"),
        tool("deleteTable", "", "− Table", "Delete the table"),
        tool("addColumn", "", "+ Col", "Add a column"),
        tool("deleteColumn", "", "− Col", "Delete the column"),
        tool("addRow", "", "+ Row", "Add a row"),
        tool("deleteRow", "", "− Row", "Delete the row"),
    ],
    &[
        tool("undo", "", "Undo", "Undo"),
        tool("redo", "", "Redo", "Redo"),
    ],
];

/// Whether a tool shows an on/off state (a mark or a block), or is an
/// action that is enabled only when it can run.
pub fn is_toggle(t: &Tool) -> bool {
    matches!(
        t.command,
        "heading"
            | "bold"
            | "italic"
            | "strike"
            | "bulletList"
            | "orderedList"
            | "blockquote"
            | "readAloud"
            | "sidebar"
    )
}

/// A markdown editor. `value` holds the markdown: the editor writes each
/// change into it, and a change from outside (another document loaded)
/// replaces the editor's content.
#[component]
pub fn RichEditor(
    value: RwSignal<String>,
    /// The accessible name of the editing area.
    #[prop(into)]
    label: String,
    #[prop(optional, into)] placeholder: String,
) -> impl IntoView {
    if !available() {
        return view! { <Textarea label=label value=value /> }.into_any();
    }
    let label = StoredValue::new(label);
    let mount = NodeRef::<Div>::new();
    let handle = StoredValue::new_local(None::<Handle>);
    // The callbacks live as long as the editor.
    let callbacks = StoredValue::new_local(None::<(Closure<dyn Fn(String)>, Closure<dyn Fn()>)>);
    // Bumped on each transaction: the toolbar reads its state again.
    let state = RwSignal::new(0u32);

    Effect::new(move |_| {
        let Some(el) = mount.get() else { return };
        if handle.with_value(Option::is_some) {
            return;
        }
        let on_change = Closure::<dyn Fn(String)>::new(move |md: String| {
            if value.get_untracked() != md {
                value.set(md);
            }
        });
        let on_state = Closure::<dyn Fn()>::new(move || state.update(|s| *s = s.wrapping_add(1)));
        let h = Handle::new(
            &el,
            &value.get_untracked(),
            &placeholder,
            &label.get_value(),
            on_change.as_ref().unchecked_ref(),
            on_state.as_ref().unchecked_ref(),
        );
        match h {
            Ok(h) => {
                handle.set_value(Some(h));
                callbacks.set_value(Some((on_change, on_state)));
            }
            Err(e) => leptos::logging::warn!("editor: {e:?}"),
        }
    });
    // Content from outside: load it into the editor.
    Effect::new(move |_| {
        let md = value.get();
        handle.with_value(|h| {
            if let Some(h) = h {
                if h.get_markdown() != md {
                    h.set_markdown(&md);
                }
            }
        });
    });
    on_cleanup(move || {
        handle.try_update_value(|h| {
            if let Some(h) = h.take() {
                h.destroy();
            }
        });
        callbacks.try_update_value(|c| c.take());
    });

    let toolbar = TOOLBAR
        .iter()
        .map(|group| {
            let buttons = group
                .iter()
                .map(|t| {
                    let t = *t;
                    let active = move || {
                        state.track();
                        is_toggle(&t)
                            && handle.with_value(|h| {
                                h.as_ref().is_some_and(|h| h.is_active(t.command, t.args))
                            })
                    };
                    let disabled = move || {
                        state.track();
                        !is_toggle(&t)
                            && !handle.with_value(|h| {
                                h.as_ref().is_some_and(|h| h.can(t.command, t.args))
                            })
                    };
                    view! {
                        <button
                            type="button"
                            class="mimir-editor__btn"
                            class:is-active=active
                            aria-pressed=move || is_toggle(&t).then(|| active().to_string())
                            disabled=disabled
                            title=t.title
                            aria-label=t.title
                            // Keep the selection in the editor.
                            on:mousedown=|ev| ev.prevent_default()
                            on:click=move |_| {
                                handle.with_value(|h| {
                                    if let Some(h) = h {
                                        h.run(t.command, t.args);
                                    }
                                });
                            }
                        >
                            {t.label}
                        </button>
                    }
                })
                .collect_view();
            view! { <div class="mimir-editor__group">{buttons}</div> }
        })
        .collect_view();

    view! {
        <div class="mimir-editor">
            <div class="mimir-editor__toolbar" role="toolbar" aria-label="Formatting">{toolbar}</div>
            <div class="mimir-editor__body" node_ref=mount></div>
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_command_is_known_to_the_bundle() {
        // The names of COMMANDS in editor/src/index.js.
        let js = include_str!("../editor/src/index.js");
        for t in TOOLBAR.iter().flat_map(|g| g.iter()) {
            assert!(
                js.contains(&format!("  {}: (c", t.command)),
                "{} is not a command of the bundle",
                t.command
            );
            if is_toggle(t) {
                assert!(js.contains(&format!("  {}: (", t.command)), "{}", t.command);
            }
            if !t.args.is_empty() {
                assert!(serde_json::from_str::<serde_json::Value>(t.args).is_ok());
            }
        }
    }

    #[test]
    fn titles_are_unique() {
        let mut titles: Vec<&str> = TOOLBAR
            .iter()
            .flat_map(|g| g.iter())
            .map(|t| t.title)
            .collect();
        let n = titles.len();
        titles.sort();
        titles.dedup();
        assert_eq!(titles.len(), n);
    }
}
