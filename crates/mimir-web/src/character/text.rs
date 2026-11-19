//! The text a sheet names (MIMIR-T-0733): a catalog entry of the sheet (an
//! item, the background, a spell) in a dialog, and the class and subclass
//! features with their text. Read through the character's routes, so a
//! player sees them for their own sheet.

use aurora_leptos::components::*;
use aurora_leptos::data::SectionLabel;
use aurora_leptos::frame::Modal;
use leptos::prelude::*;
use mimir_render::{detail, entries};
use mimir_wire::FeatureText;
use serde_json::Value;

use crate::api;
use crate::reference::detail::Rendered;

/// A catalog entry the sheet names.
#[derive(Debug, Clone, PartialEq)]
pub struct SheetRef {
    pub category: &'static str,
    pub source: String,
    pub name: String,
}

impl SheetRef {
    pub fn new(category: &'static str, source: &str, name: &str) -> Self {
        SheetRef {
            category,
            source: source.into(),
            name: name.into(),
        }
    }

    /// `/characters/<id>/entry/<category>/<source>/<name>`
    pub fn path(&self, character: &str) -> String {
        format!(
            "/characters/{character}/entry/{}/{}/{}",
            self.category,
            enc(&self.source),
            enc(&self.name)
        )
    }
}

/// Percent-encode a path segment (as `encodeURIComponent`).
pub fn enc(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'~'
            | b'!'
            | b'*'
            | b'\''
            | b'('
            | b')' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// A dialog with the entry `showing` names.
#[component]
pub fn SheetEntry(
    #[prop(into)] character: Signal<String>,
    showing: RwSignal<Option<SheetRef>>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    Effect::new(move |_| open.set(showing.with(Option::is_some)));
    Effect::new(move |prev: Option<bool>| {
        let now = open.get();
        if prev == Some(true) && !now {
            showing.set(None);
        }
        now
    });
    let entry = LocalResource::new(move || {
        let r = showing.get();
        let c = character.get();
        async move {
            match r {
                Some(r) => api::get::<Value>(&r.path(&c)).await.map(|v| Some((r, v))),
                None => Ok(None),
            }
        }
    });
    let title = Signal::derive(move || showing.get().map(|r| r.name).unwrap_or_default());
    view! {
        <Modal open=open title=title size="lg">
            {move || match entry.get() {
                None => view! { <Loading /> }.into_any(),
                Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                Some(Ok(None)) => ().into_any(),
                Some(Ok(Some((r, v)))) => {
                    let html = detail::detail(r.category, &v);
                    view! { <Rendered html=html class="mimir-detail" /> }.into_any()
                }
            }}
        </Modal>
    }
}

/// The features by class, then subclass, in level order: ("Wizard", …),
/// ("Wizard: School of Evocation", …).
pub fn group_features(list: Vec<FeatureText>) -> Vec<(String, Vec<FeatureText>)> {
    let mut groups: Vec<(String, Vec<FeatureText>)> = Vec::new();
    for f in list {
        let heading = match &f.subclass_name {
            Some(sub) => format!("{}: {sub}", f.class_name),
            None => f.class_name.clone(),
        };
        match groups.iter_mut().find(|(h, _)| *h == heading) {
            Some((_, l)) => l.push(f),
            None => groups.push((heading, vec![f])),
        }
    }
    for (_, l) in &mut groups {
        l.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
    }
    groups
}

/// The class and subclass features of a character; each opens to its text.
#[component]
pub fn ClassFeatures(#[prop(into)] character: Signal<String>) -> impl IntoView {
    let features = LocalResource::new(move || {
        api::get::<Vec<FeatureText>>(format!("/characters/{}/features", character.get()))
    });
    move || {
        match features.get() {
        None => view! { <Loading /> }.into_any(),
        Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
        Some(Ok(list)) if list.is_empty() => view! {
            <SectionLabel label="Class features" />
            <Text size="sm" dimmed=true>"The catalog has no features for these classes."</Text>
        }
        .into_any(),
        Some(Ok(list)) => group_features(list)
            .into_iter()
            .map(|(heading, list)| {
                let n = list.len();
                view! {
                    <SectionLabel label=format!("{heading} features") count=n />
                    <div class="mimir-features">
                        {list.into_iter().map(|f| {
                            let html = entries::entries(&f.entries);
                            view! {
                                <details class="mimir-feature">
                                    <summary>
                                        <span class="mimir-feature__level">{format!("Level {}", f.level)}</span>
                                        <span>{f.name}</span>
                                    </summary>
                                    <Rendered html=html />
                                </details>
                            }
                        }).collect_view()}
                    </div>
                }
            })
            .collect_view()
            .into_any(),
    }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(name: &str, class: &str, sub: Option<&str>, level: i32) -> FeatureText {
        FeatureText {
            name: name.into(),
            class_name: class.into(),
            subclass_name: sub.map(String::from),
            level,
            entries: Value::Array(vec![]),
        }
    }

    #[test]
    fn features_group_by_class_then_subclass() {
        let g = group_features(vec![
            f("Arcane Recovery", "Wizard", None, 1),
            f("Sculpt Spells", "Wizard", Some("Evocation"), 2),
            f("Spellcasting", "Wizard", None, 1),
            f("Potent Cantrip", "Wizard", Some("Evocation"), 6),
        ]);
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].0, "Wizard");
        assert_eq!(
            g[0].1.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(),
            vec!["Arcane Recovery", "Spellcasting"]
        );
        assert_eq!(g[1].0, "Wizard: Evocation");
    }

    #[test]
    fn entry_paths() {
        assert_eq!(
            SheetRef::new("items", "PHB", "Bag of Holding").path("c1"),
            "/characters/c1/entry/items/PHB/Bag%20of%20Holding"
        );
        assert_eq!(
            enc("Potion of Healing (Greater)/x"),
            "Potion%20of%20Healing%20(Greater)%2Fx"
        );
    }
}
