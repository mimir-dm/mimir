//! The campaign the DM works on (MIMIR-T-0734, the desktop's campaign
//! selector): remembered in the browser, chosen in the header, offered on
//! Home as "Continue".

use aurora_leptos::components::*;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use mimir_wire::CampaignSummary;

use crate::api;

const KEY: &str = "mimir.last_campaign";

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// The campaign the DM last opened (its id; empty: none).
#[derive(Clone, Copy)]
pub struct CurrentCampaign(pub RwSignal<String>);

impl CurrentCampaign {
    /// Provide it, from the browser's memory (the DM shell).
    pub fn provide() -> Self {
        let id = storage()
            .and_then(|s| s.get_item(KEY).ok().flatten())
            .unwrap_or_default();
        let c = CurrentCampaign(RwSignal::new(id));
        provide_context(c);
        c
    }

    pub fn get() -> Option<Self> {
        use_context::<CurrentCampaign>()
    }

    /// The DM opened this campaign.
    pub fn set(&self, id: &str) {
        if self.0.get_untracked() != id {
            self.0.set(id.to_string());
        }
        if let Some(s) = storage() {
            let _ = s.set_item(KEY, id);
        }
    }
}

/// The selector's options: active campaigns by name, then the archived
/// ones, marked.
pub fn switcher_options(list: &[CampaignSummary]) -> Vec<(String, String)> {
    let mut active: Vec<&CampaignSummary> =
        list.iter().filter(|c| c.archived_at.is_none()).collect();
    let mut archived: Vec<&CampaignSummary> =
        list.iter().filter(|c| c.archived_at.is_some()).collect();
    active.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    archived.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    active
        .into_iter()
        .map(|c| (c.id.clone(), c.name.clone()))
        .chain(
            archived
                .into_iter()
                .map(|c| (c.id.clone(), format!("{} (archived)", c.name))),
        )
        .collect()
}

/// The header's campaign selector: a choice opens the campaign.
#[component]
pub fn CampaignSwitcher() -> impl IntoView {
    let Some(current) = CurrentCampaign::get() else {
        return ().into_any();
    };
    let campaigns = LocalResource::new(move || {
        // Read again when the current one changes (a new campaign).
        current.0.track();
        api::campaigns(true)
    });
    let value = RwSignal::new(current.0.get_untracked());
    Effect::new(move |_| value.set(current.0.get()));
    let navigate = use_navigate();
    let open = Callback::new(move |id: String| {
        if !id.is_empty() {
            navigate(&format!("/campaigns/{id}"), Default::default());
        }
    });
    view! {
        <div class="mimir-switcher">
            {move || {
                let opts = campaigns
                    .get()
                    .and_then(Result::ok)
                    .map(|l| switcher_options(&l))
                    .unwrap_or_default();
                view! {
                    <Select aria_label="Campaign" placeholder="Campaign…" option_pairs=opts value=value on_change=open />
                }
            }}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(id: &str, name: &str, archived: bool) -> CampaignSummary {
        serde_json::from_value(serde_json::json!({
            "id": id, "name": name, "description": null,
            "archived_at": if archived { serde_json::json!("2026-01-01") } else { serde_json::Value::Null },
            "created_at": "", "updated_at": ""
        }))
        .unwrap()
    }

    #[test]
    fn active_first_then_archived_marked() {
        let o = switcher_options(&[
            c("1", "Zed", false),
            c("2", "Old", true),
            c("3", "alpha", false),
        ]);
        assert_eq!(
            o.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>(),
            vec!["alpha", "Zed", "Old (archived)"]
        );
    }
}
