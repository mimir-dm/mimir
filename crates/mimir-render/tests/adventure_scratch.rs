//! A check of the adventure converter on real 5etools adventures
//! (MIMIR-T-0738). Official text is never in the repository: point
//! `MIMIR_ADVENTURE_DIR` at a folder of `adventure-*.json` files (a scratch
//! copy) and run
//!
//! ```text
//! MIMIR_ADVENTURE_DIR=/tmp/advscratch cargo test -p mimir-render --test adventure_scratch -- --ignored --nocapture
//! ```
//!
//! For each chapter, entries → markdown → entries must keep the text, the
//! headings and the number of boxes, tables, stat blocks, images, quotes
//! and lists.

use std::collections::BTreeMap;

use mimir_render::adventure::{area_targets, from_markdown, refs, to_markdown, ToMarkdown};
use serde_json::Value;

/// What a round trip must keep.
#[derive(Debug, Default, PartialEq)]
struct Shape {
    headings: Vec<String>,
    text: String,
    counts: BTreeMap<String, usize>,
}

fn shape(entries: &[Value]) -> Shape {
    let mut s = Shape::default();
    for e in entries {
        walk(e, &mut s);
    }
    s.text = s.text.split_whitespace().collect::<Vec<_>>().join(" ");
    s
}

fn count(s: &mut Shape, k: &str) {
    *s.counts.entry(k.to_string()).or_default() += 1;
}

fn walk(v: &Value, s: &mut Shape) {
    match v {
        Value::String(t) => {
            s.text.push_str(t);
            s.text.push(' ');
        }
        Value::Array(l) => l.iter().for_each(|x| walk(x, s)),
        Value::Object(o) => {
            let kind = o.get("type").and_then(Value::as_str).unwrap_or("");
            let name = o.get("name").and_then(Value::as_str);
            match kind {
                "section" | "entries" => {
                    if let Some(n) = name {
                        s.headings.push(n.to_string());
                    }
                }
                "insetReadaloud" | "inset" | "quote" | "table" | "image" | "list" => {
                    count(s, kind);
                    if let Some(n) = name {
                        s.text.push_str(n);
                        s.text.push(' ');
                    }
                }
                "statblock" => {
                    count(s, kind);
                    return;
                }
                "item" | "itemSub" | "itemSpell" => {
                    if let Some(n) = name {
                        s.text.push_str(n);
                        s.text.push(' ');
                    }
                }
                "cell" => {
                    // A cell's roll becomes its text.
                    if let Some(e) = o.get("entry") {
                        walk(e, s);
                    } else if let Some(r) = o.get("roll").and_then(Value::as_object) {
                        let n = |k: &str| r.get(k).and_then(Value::as_i64);
                        match (n("exact"), n("min"), n("max")) {
                            (Some(x), _, _) => s.text.push_str(&format!("{x} ")),
                            (_, Some(a), Some(b)) => s.text.push_str(&format!("{a}–{b} ")),
                            _ => {}
                        }
                    }
                    return;
                }
                _ => {}
            }
            for key in [
                "caption",
                "colLabels",
                "entries",
                "entry",
                "items",
                "rows",
                "footnotes",
                "wrappeds",
                "images",
                "title",
                "by",
            ] {
                if let Some(x) = o.get(key) {
                    walk(x, s);
                }
            }
        }
        Value::Number(n) => {
            s.text.push_str(&n.to_string());
            s.text.push(' ');
        }
        _ => {}
    }
}

#[test]
#[ignore = "needs MIMIR_ADVENTURE_DIR: a scratch copy of 5etools adventures"]
fn real_adventures_round_trip() {
    let dir = std::env::var("MIMIR_ADVENTURE_DIR").expect("MIMIR_ADVENTURE_DIR");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("adventure-") && n.ends_with(".json"))
        })
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no adventure-*.json in {dir}");
    let mut failures = 0;
    for f in files {
        let v: Value = serde_json::from_slice(&std::fs::read(&f).unwrap()).unwrap();
        let data = &v["adventureData"][0]["data"];
        let opts = ToMarkdown {
            keep_ids: area_targets(data),
        };
        let chapters = data.as_array().unwrap();
        let mut md_bytes = 0;
        let mut totals: BTreeMap<String, usize> = BTreeMap::new();
        for ch in chapters {
            let md = to_markdown(std::slice::from_ref(ch), 1, &opts);
            md_bytes += md.len();
            let back = from_markdown(&md, 1);
            let (a, b) = (shape(std::slice::from_ref(ch)), shape(&back));
            for (k, n) in &a.counts {
                *totals.entry(k.clone()).or_default() += n;
            }
            if a != b {
                failures += 1;
                let name = ch["name"].as_str().unwrap_or("?");
                println!("MISMATCH {} / {name}", f.display());
                if a.headings != b.headings {
                    let i = a
                        .headings
                        .iter()
                        .zip(&b.headings)
                        .position(|(x, y)| x != y)
                        .unwrap_or(a.headings.len().min(b.headings.len()));
                    println!(
                        "  headings {} vs {}; first difference at {i}: {:?} vs {:?}",
                        a.headings.len(),
                        b.headings.len(),
                        a.headings.get(i),
                        b.headings.get(i)
                    );
                }
                if a.counts != b.counts {
                    println!("  counts {:?} vs {:?}", a.counts, b.counts);
                }
                if a.text != b.text {
                    let i = a
                        .text
                        .char_indices()
                        .zip(b.text.chars())
                        .find(|((_, x), y)| x != y)
                        .map(|((i, _), _)| i)
                        .unwrap_or(a.text.len().min(b.text.len()));
                    let from = a.text.floor_char_boundary(i.saturating_sub(60));
                    let to_a = a.text.ceil_char_boundary((i + 60).min(a.text.len()));
                    let to_b = b.text.ceil_char_boundary((i + 60).min(b.text.len()));
                    println!(
                        "  text differs at {i}:\n    {:?}\n    {:?}",
                        &a.text[from..to_a],
                        &b.text[from.min(b.text.len())..to_b]
                    );
                }
            }
        }
        let named = refs(data);
        let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
        for r in &named {
            *kinds.entry(r.kind.as_str()).or_default() += 1;
        }
        println!(
            "{}: {} chapters, {} KB of markdown, {:?}; names {} catalog entries {:?}",
            f.file_name().unwrap().to_string_lossy(),
            chapters.len(),
            md_bytes / 1024,
            totals,
            named.len(),
            kinds
        );
    }
    assert_eq!(failures, 0, "chapters that do not round-trip");
}
