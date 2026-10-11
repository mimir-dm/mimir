//! The Equipment tab (MIMIR-T-0717): coins and the inventory, with the
//! changes of the desktop inventory manager: add from the catalog (search
//! by name), quantity, equip, attune (at most 3; the server checks),
//! remove.

use aurora_leptos::components::*;
use aurora_leptos::data::SectionLabel;
use aurora_leptos::tokens::token;
use leptos::prelude::*;
use leptos::task::spawn_local;
use mimir_wire::{CharacterSheet, Choice, Currency, InventoryItem};
use serde_json::json;

use super::stats;
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
pub fn InventoryPanel(sheet: CharacterSheet, on_changed: Callback<()>) -> impl IntoView {
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
    let find = move || {
        let q = search.get_untracked();
        spawn_local(async move {
            match api::get::<Vec<Choice>>(format!("/catalog/items?search={}", encode(&q))).await {
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
            spawn_local(async move {
                match api::send::<_, InventoryItem>(
                    "POST",
                    &path,
                    &json!({"item_name": c.name, "item_source": c.source, "quantity": qty}),
                )
                .await
                {
                    Ok(_) => {
                        results.set(Vec::new());
                        search.set(String::new());
                        quantity.set("1".into());
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
    let mut items = sheet.inventory.clone();
    items.sort_by(|a, b| {
        b.equipped
            .cmp(&a.equipped)
            .then(a.item_name.cmp(&b.item_name))
    });

    view! {
        <SectionLabel label="Coins" />
        <div class="mimir-inv__coins">
            {coins.iter().map(|(label, v)| view! {
                <TextInput label=*label value=*v input_type="number" />
            }).collect_view()}
            <Button variant="light" on_click=Callback::new(save_coins)>"Save coins"</Button>
        </div>
        <Text size="sm" dimmed=true>{format!("Worth {:.2} gp in all.", stats::gold_value(&sheet))}</Text>

        <SectionLabel label="Inventory" count=items.len() />
        <Text size="sm" dimmed=true>{format!("Attuned: {attuned} of 3")}</Text>
        {if items.is_empty() {
            view! { <Text size="sm" dimmed=true>"Nothing carried."</Text> }.into_any()
        } else {
            view! {
                <ul class="mimir-inv__list" aria-label="Inventory">
                    {items.into_iter().map(|i| {
                        let equipped = RwSignal::new(i.equipped);
                        let attune = RwSignal::new(i.attuned);
                        let (a, b, c, d, e) = (i.id.clone(), i.id.clone(), i.id.clone(), i.id.clone(), i.id.clone());
                        let q = i.quantity;
                        let name = i.item_name.clone();
                        let label = name.clone();
                        let (fewer, more, gone) = (format!("One fewer {name}"), format!("One more {name}"), format!("Remove {name}"));
                        view! {
                            <li class="mimir-inv__item" aria-label=label>
                                <div class="mimir-inv__name">
                                    <Text bold=true>{name}</Text>
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
                    }).collect_view()}
                </ul>
            }.into_any()
        }}

        <SectionLabel label="Add an item" />
        <form class="mimir-sheet__add" on:submit=move |ev| {
            ev.prevent_default();
            find();
        }>
            <TextInput aria_label="Item name" placeholder="Search the catalog (2 letters or more)" value=search name="item" />
            <TextInput aria_label="Quantity" value=quantity input_type="number" name="qty" />
            <Button variant="light" button_type="submit">"Search"</Button>
        </form>
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
    fn query_encoding() {
        assert_eq!(encode("rope of climbing"), "rope%20of%20climbing");
        assert_eq!(encode("+1"), "%2B1");
    }
}
