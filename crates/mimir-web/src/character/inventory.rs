//! The Equipment tab (MIMIR-T-0717): coins, what is equipped, and the
//! rest of the inventory, with the changes of the desktop inventory
//! manager: add from the catalog (search by name, with a quantity and
//! notes), quantity, equip, attune (at most 3; the server checks), remove.
//! A name opens the item's details (MIMIR-T-0733).

use aurora_leptos::components::*;
use aurora_leptos::data::SectionLabel;
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_wire::{CharacterSheet, Choice, Currency, InventoryItem};
use serde_json::json;

use super::stats;
use super::text::SheetRef;
use crate::api;

/// The coin fields: (label, value).
pub fn coin_fields(c: &Currency) -> [(&'static str, i32); 5] {
    [
        ("PP", c.pp),
        ("GP", c.gp),
        ("EP", c.ep),
        ("SP", c.sp),
        ("CP", c.cp),
    ]
}

/// Coins from the fields (PP, GP, EP, SP, CP); `None` if one is not a
/// whole number of at least 0.
pub fn parse_coins(texts: [&str; 5]) -> Option<Currency> {
    let n: Vec<i32> = texts
        .iter()
        .map(|t| t.trim().parse::<i32>().ok().filter(|n| *n >= 0))
        .collect::<Option<Vec<_>>>()?;
    Some(Currency {
        pp: n[0],
        gp: n[1],
        ep: n[2],
        sp: n[3],
        cp: n[4],
    })
}

#[component]
pub fn InventoryPanel(
    sheet: CharacterSheet,
    on_changed: Callback<()>,
    /// A shared sheet seen by another player: no changes.
    #[prop(optional)]
    read_only: bool,
    /// Where a name's details open.
    #[prop(optional)]
    showing: Option<RwSignal<Option<SheetRef>>>,
) -> impl IntoView {
    let id = sheet.id.clone();
    let error = RwSignal::new(String::new());
    let fail = move |e| error.set(api::message(e));

    // Coins.
    let coins: Vec<(&'static str, RwSignal<String>)> = coin_fields(&sheet.currency)
        .into_iter()
        .map(|(l, v)| (l, RwSignal::new(v.to_string())))
        .collect();
    let save_coins = {
        let coins = coins.clone();
        let id = id.clone();
        move |_| {
            let texts: Vec<String> = coins.iter().map(|(_, v)| v.get_untracked()).collect();
            let Some(c) = parse_coins([&texts[0], &texts[1], &texts[2], &texts[3], &texts[4]])
            else {
                error.set("Coins are whole numbers, 0 or more.".into());
                return;
            };
            let path = format!("/characters/{id}");
            spawn_local(async move {
                match api::send::<_, CharacterSheet>("PATCH", &path, &json!({ "currency": c }))
                    .await
                {
                    Ok(_) => {
                        error.set(String::new());
                        on_changed.run(());
                    }
                    Err(e) => fail(e),
                }
            });
        }
    };

    // Add from the catalog.
    let search = RwSignal::new(String::new());
    let results = RwSignal::new(Vec::<Choice>::new());
    let quantity = RwSignal::new("1".to_string());
    let notes = RwSignal::new(String::new());
    // The campaign's homebrew items join the search (source HB, MIMIR-T-0722).
    let campaign = StoredValue::new(
        sheet
            .campaign_id
            .clone()
            .map(|c| format!("&campaign={}", encode(&c)))
            .unwrap_or_default(),
    );
    let find = move || {
        let q = search.get_untracked();
        spawn_local(async move {
            let path = format!(
                "/catalog/items?search={}{}",
                encode(&q),
                campaign.get_value()
            );
            match api::get::<Vec<Choice>>(path).await {
                Ok(list) => results.set(list),
                Err(e) => fail(e),
            }
        });
    };
    let add = {
        let id = id.clone();
        move |c: Choice| {
            let qty = quantity
                .get_untracked()
                .trim()
                .parse::<i32>()
                .unwrap_or(1)
                .max(1);
            let path = format!("/characters/{id}/inventory");
            let note = notes.get_untracked().trim().to_string();
            spawn_local(async move {
                match api::send::<_, InventoryItem>(
                    "POST",
                    &path,
                    &json!({
                        "item_name": c.name,
                        "item_source": c.source,
                        "quantity": qty,
                        "notes": (!note.is_empty()).then_some(note),
                    }),
                )
                .await
                {
                    Ok(_) => {
                        results.set(Vec::new());
                        search.set(String::new());
                        quantity.set("1".into());
                        notes.set(String::new());
                        on_changed.run(());
                    }
                    Err(e) => fail(e),
                }
            });
        }
    };

    let change = move |item_id: String, body: serde_json::Value| {
        spawn_local(async move {
            match api::send::<_, InventoryItem>("PATCH", &format!("/inventory/{item_id}"), &body)
                .await
            {
                Ok(_) => on_changed.run(()),
                Err(e) => fail(e),
            }
        });
    };
    let remove = move |item_id: String| {
        spawn_local(async move {
            match api::delete(&format!("/inventory/{item_id}")).await {
                Ok(()) => on_changed.run(()),
                Err(e) => fail(e),
            }
        });
    };

    let attuned = sheet.inventory.iter().filter(|i| i.attuned).count();
    let (equipped_items, carried) = split_equipped(sheet.inventory.clone());

    let row = move |i: InventoryItem| {
        let equipped = RwSignal::new(i.equipped);
        let attune = RwSignal::new(i.attuned);
        let (a, b, c, d, e) = (
            i.id.clone(),
            i.id.clone(),
            i.id.clone(),
            i.id.clone(),
            i.id.clone(),
        );
        let q = i.quantity;
        let name = i.item_name.clone();
        let label = name.clone();
        let r = SheetRef::new("items", &i.item_source, &i.item_name);
        // The name opens the details when the sheet has a place for them.
        let shown = name.clone();
        let title = match showing {
            Some(sh) => view! {
                <button type="button" class="mimir-link-button" on:click=move |_| sh.set(Some(r.clone()))>{shown}</button>
            }
            .into_any(),
            None => view! { <Text bold=true>{shown}</Text> }.into_any(),
        };
        let (fewer, more, gone) = (
            format!("One fewer {name}"),
            format!("One more {name}"),
            format!("Remove {name}"),
        );
        if read_only {
            let flags = [(i.equipped, "equipped"), (i.attuned, "attuned")]
                .iter()
                .filter(|(on, _)| *on)
                .map(|(_, l)| *l)
                .collect::<Vec<_>>()
                .join(", ");
            return view! {
                <li class="mimir-inv__item" aria-label=label>
                    <div class="mimir-inv__name">{title}</div>
                    <span class="mimir-sheet__num">{format!("×{q} {flags}")}</span>
                </li>
            }
            .into_any();
        }
        view! {
            <li class="mimir-inv__item" aria-label=label>
                <div class="mimir-inv__name">
                    {title}
                    <Text size="xs" dimmed=true>{i.item_source.clone()}{i.notes.clone().map(|n| format!(" · {n}"))}</Text>
                </div>
                <div class="mimir-inv__qty">
                    <ActionIcon title=fewer on_click=Callback::new(move |_| {
                        if q > 1 { change(a.clone(), json!({"quantity": q - 1})) }
                    })>"−"</ActionIcon>
                    <span aria-label="Quantity">{q}</span>
                    <ActionIcon title=more on_click=Callback::new(move |_| change(b.clone(), json!({"quantity": q + 1})))>"+"</ActionIcon>
                </div>
                <Switch label="Equipped" checked=equipped on_change=Callback::new(move |on: bool| change(c.clone(), json!({"equipped": on}))) />
                <Switch label="Attuned" checked=attune on_change=Callback::new(move |on: bool| change(d.clone(), json!({"attuned": on}))) />
                <ActionIcon title=gone on_click=Callback::new(move |_| remove(e.clone()))>"×"</ActionIcon>
            </li>
        }
        .into_any()
    };

    view! {
        <SectionLabel label="Coins" />
        {if read_only {
            view! {
                <Text>{coin_fields(&sheet.currency).iter().map(|(l, v)| format!("{v} {l}")).collect::<Vec<_>>().join(" · ")}</Text>
            }.into_any()
        } else {
            view! {
                <div class="mimir-inv__coins">
                    {coins.iter().map(|(label, v)| view! {
                        <TextInput label=*label value=*v input_type="number" />
                    }).collect_view()}
                    <Button variant="light" on_click=Callback::new(save_coins.clone())>"Save coins"</Button>
                </div>
            }.into_any()
        }}
        <Text size="sm" dimmed=true>{format!("Worth {:.2} gp in all.", stats::gold_value(&sheet))}</Text>

        <SectionLabel label="Equipped" count=equipped_items.len() />
        <Text size="sm" dimmed=true>{format!("Attuned: {attuned} of 3")}</Text>
        {if equipped_items.is_empty() {
            view! { <Text size="sm" dimmed=true>"Nothing equipped."</Text> }.into_any()
        } else {
            view! {
                <ul class="mimir-inv__list" aria-label="Equipped">
                    {equipped_items.into_iter().map(row).collect_view()}
                </ul>
            }.into_any()
        }}
        <SectionLabel label="Inventory" count=carried.len() />
        {if carried.is_empty() {
            view! { <Text size="sm" dimmed=true>"Nothing else carried."</Text> }.into_any()
        } else {
            view! {
                <ul class="mimir-inv__list" aria-label="Inventory">
                    {carried.into_iter().map(row).collect_view()}
                </ul>
            }.into_any()
        }}

        {(!read_only).then(|| view! {
        <SectionLabel label="Add an item" />
        <form class="mimir-sheet__add" on:submit=move |ev| {
            ev.prevent_default();
            find();
        }>
            <TextInput aria_label="Item name" placeholder="Search the catalog (2 letters or more)" value=search name="item" />
            <TextInput aria_label="Quantity" value=quantity input_type="number" name="qty" />
            <TextInput aria_label="Item notes" placeholder="Notes (optional)" value=notes name="notes" />
            <Button variant="light" button_type="submit">"Search"</Button>
        </form>
        })}
        {move || {
            let list = results.get();
            (!list.is_empty()).then(|| {
                let add = add.clone();
                view! {
                    <ul class="mimir-inv__results" aria-label="Catalog items">
                        {list.into_iter().map(|c| {
                            let add = add.clone();
                            let pick = c.clone();
                            view! {
                                <li>
                                    <span>{c.name.clone()}<Text size="xs" dimmed=true>{format!(" {} {}", c.source, c.detail.clone().unwrap_or_default())}</Text></span>
                                    <Button variant="subtle" on_click=Callback::new(move |_| add(pick.clone()))>"Add"</Button>
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                }
            })
        }}
        {move || {
            let e = error.get();
            (!e.is_empty()).then(|| view! { <Alert color=token::BAD>{e}</Alert> })
        }}
    }
}

/// The equipped items and the rest, each by name.
pub fn split_equipped(mut items: Vec<InventoryItem>) -> (Vec<InventoryItem>, Vec<InventoryItem>) {
    items.sort_by(|a, b| a.item_name.cmp(&b.item_name));
    items.into_iter().partition(|i| i.equipped)
}

/// Percent-encode a query value.
pub fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coins_parse_whole_numbers_of_zero_or_more() {
        assert_eq!(
            parse_coins(["1", "20", "0", " 3 ", "40"]),
            Some(Currency {
                pp: 1,
                gp: 20,
                ep: 0,
                sp: 3,
                cp: 40
            })
        );
        assert_eq!(parse_coins(["1", "-2", "0", "0", "0"]), None);
        assert_eq!(parse_coins(["1", "x", "0", "0", "0"]), None);
        let c = Currency {
            cp: 1,
            sp: 2,
            ep: 3,
            gp: 4,
            pp: 5,
        };
        assert_eq!(
            coin_fields(&c).map(|(l, _)| l),
            ["PP", "GP", "EP", "SP", "CP"]
        );
    }

    #[test]
    fn equipped_apart_from_the_rest() {
        let item = |n: &str, e: bool| InventoryItem {
            id: n.into(),
            item_name: n.into(),
            item_source: "PHB".into(),
            quantity: 1,
            equipped: e,
            attuned: false,
            notes: None,
        };
        let (on, rest) = split_equipped(vec![
            item("Shield", true),
            item("Rope", false),
            item("Axe", true),
        ]);
        assert_eq!(
            on.iter().map(|i| i.item_name.as_str()).collect::<Vec<_>>(),
            vec!["Axe", "Shield"]
        );
        assert_eq!(rest.len(), 1);
    }

    #[test]
    fn query_encoding() {
        assert_eq!(encode("rope of climbing"), "rope%20of%20climbing");
        assert_eq!(encode("+1"), "%2B1");
    }
}
