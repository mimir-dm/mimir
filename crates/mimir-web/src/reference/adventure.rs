//! An adventure as a campaign (MIMIR-T-0739): the plan of the adventure
//! open in the reader, the DM's choice for each part, then the campaign.

use aurora_leptos::components::*;
use aurora_leptos::frame::Modal;
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_wire::{AdventureCampaign, AdventureCampaignRequest, AdventurePlan, PartChoice};

use crate::api;

/// The roles a part can take, as the select shows them.
pub const ROLES: [(&str, &str); 3] = [
    ("module", "Module"),
    ("document", "Campaign document"),
    ("skip", "Leave out"),
];

/// "3 documents · 1 map · 6 creatures" for a part as a module.
pub fn part_line(documents: usize, maps: usize, creatures: usize) -> String {
    let n = |k: usize, one: &str, many: &str| format!("{k} {}", if k == 1 { one } else { many });
    [
        n(documents, "document", "documents"),
        n(maps, "map", "maps"),
        n(creatures, "creature", "creatures"),
    ]
    .join(" · ")
}

/// "Made 4 modules, 30 documents, 5 maps and 40 monsters."
pub fn made_line(m: &AdventureCampaign) -> String {
    let n = |k: usize, one: &str, many: &str| format!("{k} {}", if k == 1 { one } else { many });
    let mut s = format!(
        "Made {}, {}, {} and {}.",
        n(m.modules, "module", "modules"),
        n(m.documents, "document", "documents"),
        n(m.maps, "map", "maps"),
        n(m.monsters, "monster", "monsters")
    );
    if !m.missing.is_empty() {
        s += &format!(" Not found: {}.", m.missing.join(", "));
    }
    s
}

#[component]
pub fn AdventureCampaignDialog(
    open: RwSignal<bool>,
    #[prop(into)] source: Signal<String>,
) -> impl IntoView {
    let plan = LocalResource::new(move || {
        let s = source.get();
        let shown = open.get();
        async move {
            if !shown || s.is_empty() {
                return Ok(None);
            }
            api::get::<AdventurePlan>(format!("/books/{}/plan", js_sys::encode_uri_component(&s)))
                .await
                .map(Some)
        }
    });
    let name = RwSignal::new(String::new());
    // index -> (role, one document)
    let choices = RwSignal::new(Vec::<(String, bool)>::new());
    Effect::new(move |_| {
        if let Some(Ok(Some(p))) = plan.get() {
            name.set(p.name.clone());
            choices.set(p.parts.iter().map(|x| (x.role.clone(), false)).collect());
        }
    });
    let busy = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    let made = RwSignal::new(None::<AdventureCampaign>);
    Effect::new(move |_| {
        if open.get() {
            made.set(None);
        }
    });
    let create = move |_| {
        let req = AdventureCampaignRequest {
            name: name.get_untracked(),
            parts: choices
                .get_untracked()
                .into_iter()
                .enumerate()
                .map(|(index, (role, one_document))| PartChoice {
                    index,
                    role,
                    one_document,
                })
                .collect(),
        };
        let s = source.get_untracked();
        busy.set(true);
        error.set(String::new());
        spawn_local(async move {
            let path = format!("/books/{}/campaign", js_sys::encode_uri_component(&s));
            match api::send::<_, AdventureCampaign>("POST", &path, &req).await {
                Ok(m) => {
                    busy.set(false);
                    made.set(Some(m));
                }
                Err(e) => {
                    busy.set(false);
                    error.set(api::message(e));
                }
            }
        });
    };
    view! {
        <Modal open=open title="Make a campaign" size="lg" locked=busy>
            {move || made.get().map(|m| {
                let line = made_line(&m);
                let href = format!("/campaigns/{}", m.campaign_id);
                view! {
                <Stack gap="sm">
                    <Alert color=token::OK><span role="status">{line}</span></Alert>
                    <Group>
                        <a class="cl-btn cl-btn--light" href=href>"Open the campaign"</a>
                        <Button variant="subtle" on_click=Callback::new(move |_| open.set(false))>"Close"</Button>
                    </Group>
                </Stack>
            }})}
            {move || if made.with(Option::is_some) { ().into_any() } else { match plan.get() {
                None | Some(Ok(None)) => view! { <Loading /> }.into_any(),
                Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                Some(Ok(Some(p))) => {
                    let roles = ROLES.iter().map(|(k, n)| (k.to_string(), n.to_string())).collect::<Vec<_>>();
                    let roles = StoredValue::new(roles);
                    view! {
                        <Stack gap="sm">
                            <TextInput label="Campaign name" value=name />
                            <Text size="sm" dimmed=true>"A numbered chapter or part becomes a module, with a document for each section, its square-grid maps and its creatures. The rest becomes campaign documents."</Text>
                            <Table label="Parts" min_width="560px">
                                <thead><tr><th>"Part"</th><th>"Becomes"</th><th>"As a module"</th></tr></thead>
                                <tbody>
                                    {p.parts.into_iter().map(|part| {
                                        let i = part.index;
                                        let role = RwSignal::new(part.role.clone());
                                        Effect::new(move |_| {
                                            let r = role.get();
                                            choices.update(|c| if let Some(x) = c.get_mut(i) { x.0 = r; });
                                        });
                                        let one = move || choices.with(|c| c.get(i).is_some_and(|x| x.1));
                                        let is_module = move || role.get() == "module";
                                        let line = part_line(part.documents.len(), part.maps.len(), part.creatures);
                                        let label = format!("{} becomes", part.name);
                                        view! {
                                            <tr>
                                                <td>{part.name.clone()}</td>
                                                <td><Select aria_label=label option_pairs=roles.get_value() value=role /></td>
                                                <td>
                                                    <Text size="sm" dimmed=true>{line}</Text>
                                                    <label class="mimir-check" class:mimir-hidden=move || !is_module()>
                                                        <input
                                                            type="checkbox"
                                                            aria-label=format!("{} in one document", part.name)
                                                            prop:checked=one
                                                            on:change=move |_| choices.update(|c| if let Some(x) = c.get_mut(i) { x.1 = !x.1; })
                                                        />
                                                        " One document"
                                                    </label>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </Table>
                            {move || {
                                let e = error.get();
                                (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
                            }}
                            <Group>
                                <Button
                                    disabled=Signal::derive(move || name.get().trim().is_empty() || busy.get())
                                    on_click=Callback::new(create)
                                >"Make the campaign"</Button>
                                <Button variant="subtle" on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                            </Group>
                        </Stack>
                    }.into_any()
                }
            }}}
        </Modal>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines() {
        assert_eq!(part_line(3, 1, 6), "3 documents · 1 map · 6 creatures");
        assert_eq!(part_line(1, 0, 1), "1 document · 0 maps · 1 creature");
        let m = AdventureCampaign {
            campaign_id: "c".into(),
            modules: 2,
            documents: 5,
            maps: 1,
            monsters: 5,
            missing: vec!["map image adventure/X/a.png".into()],
        };
        assert_eq!(
            made_line(&m),
            "Made 2 modules, 5 documents, 1 map and 5 monsters. Not found: map image adventure/X/a.png."
        );
    }
}
