//! Markdown to HTML for the document viewer.
//!
//! Raw HTML in the markdown is escaped, not passed through: the HTML goes
//! into the page with `inner_html`. Links with a `javascript:` (or `data:`,
//! `vbscript:`) address lose the address.
//!
//! The adventure forms (MIMIR-T-0737, ADR MIMIR-A-0011): `> [!read-aloud]`
//! and `> [!sidebar] Title` are boxes, a paragraph that is only a catalog
//! tag (`{@creature Klarg|LMoP}`) is a stat block, inline `{@…}` tags render
//! as in the reader, `{#id}` gives a heading its id, and a `5etools:` image
//! loads from the catalog images. With [`Catalog::Off`] (a player's view)
//! the tags are plain text and nothing points at the catalog.

use std::ops::Range;

use mimir_render::adventure::{FLUFF_TAG, IMAGE_SCHEME};
use mimir_render::{esc, link, tags};
use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag, TagEnd};

/// Whether the HTML may point at the catalog (the DM's views).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Catalog {
    /// Tags link to the catalog; stat block lines and 5etools images load.
    On,
    /// Tags are their plain text; a stat block line is its name.
    Off,
}

/// The tags a stat block line may hold (as mimir-render's converter).
const STATBLOCK_TAGS: &[&str] = &[
    "creature",
    FLUFF_TAG,
    "item",
    "spell",
    "background",
    "deck",
    "object",
    "vehicle",
    "trap",
    "hazard",
    "race",
    "feat",
    "optfeature",
    "reward",
    "table",
];

/// A stat block line: (tag, name, source).
pub fn statblock_line(text: &str) -> Option<(&str, &str, Option<&str>)> {
    let inner = text.trim().strip_prefix("{@")?.strip_suffix('}')?;
    if inner.contains(['{', '}']) {
        return None;
    }
    let (tag, args) = inner.split_once(' ')?;
    if !STATBLOCK_TAGS.contains(&tag) {
        return None;
    }
    let mut parts = args.split('|');
    let name = parts.next()?.trim();
    let source = parts.next().map(str::trim).filter(|s| !s.is_empty());
    (parts.next().is_none() && !name.is_empty()).then_some((tag, name, source))
}

/// Render markdown to HTML that is safe to put in the page, as the DM sees
/// it.
pub fn to_html(source: &str) -> String {
    to_html_with(source, Catalog::On)
}

/// Render markdown to HTML that is safe to put in the page.
pub fn to_html_with(source: &str, catalog: Catalog) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_HEADING_ATTRIBUTES;
    let raw: Vec<(Event, Range<usize>)> = Parser::new_ext(source, options)
        .into_offset_iter()
        .map(|(event, range)| (safe_event(event), range))
        .collect();
    let events = merge_text(raw);
    let out_events = adventure_forms(events, source, catalog);
    let mut out = String::new();
    html::push_html(&mut out, out_events.into_iter());
    out
}

/// Raw HTML as text; no script addresses.
fn safe_event(event: Event<'_>) -> Event<'_> {
    match event {
        // push_html escapes Text events; Html events it does not.
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: safe_url(dest_url),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Image {
            link_type,
            dest_url: safe_url(dest_url),
            title,
            id,
        }),
        other => other,
    }
}

/// Adjacent text events as one (the parser splits text at `[`, `]` and
/// the like), so a tag or a callout marker is in one piece.
fn merge_text(events: Vec<(Event<'_>, Range<usize>)>) -> Vec<(Event<'_>, Range<usize>)> {
    let mut out: Vec<(Event, Range<usize>)> = Vec::with_capacity(events.len());
    for (e, r) in events {
        if let (Event::Text(t), Some((Event::Text(prev), pr))) = (&e, out.last_mut()) {
            *prev = CowStr::from(format!("{prev}{t}"));
            pr.end = r.end;
            continue;
        }
        out.push((e, r));
    }
    out
}

/// The opening of a callout box, from its marker line.
fn callout_open(line: &str) -> Option<String> {
    for (marker, class) in [
        ("[!read-aloud]", "inset-readaloud"),
        ("[!sidebar]", "inset"),
    ] {
        if let Some(rest) = line.trim_start().strip_prefix(marker) {
            let title = rest.trim();
            return Some(if title.is_empty() {
                format!(r#"<div class="{class}">"#)
            } else {
                format!(r#"<div class="{class}"><h5>{}</h5>"#, tags::tags(title))
            });
        }
    }
    None
}

fn statblock_html(tag: &str, name: &str, source: Option<&str>, catalog: Catalog) -> String {
    if catalog == Catalog::Off {
        return format!("<p>{}</p>", esc(name));
    }
    let (kind, lore) = if tag == FLUFF_TAG {
        ("creature", true)
    } else {
        (tag, false)
    };
    let src = source.unwrap_or("");
    format!(
        r#"<div class="mimir-statblock-line{}" data-ref-type="{}" data-ref-name="{}" data-ref-source="{}">{}</div>"#,
        if lore {
            " mimir-statblock-line--lore"
        } else {
            ""
        },
        esc(kind),
        esc(name),
        esc(src),
        link(kind, name, src, &esc(name))
    )
}

fn adventure_forms<'a>(
    events: Vec<(Event<'a>, Range<usize>)>,
    source: &str,
    catalog: Catalog,
) -> Vec<Event<'a>> {
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    // For each open block quote: whether it is a callout box.
    let mut quotes: Vec<bool> = Vec::new();
    let mut in_code = false;
    let mut i = 0;
    while i < events.len() {
        let (ev, range) = &events[i];
        match ev {
            Event::Start(Tag::CodeBlock(_)) => in_code = true,
            Event::End(TagEnd::CodeBlock) => in_code = false,
            _ => {}
        }
        // A block quote whose first line is a callout marker.
        if let Event::Start(Tag::BlockQuote(_)) = ev {
            if let (
                Some((Event::Start(Tag::Paragraph), _)),
                Some((Event::Text(t), _)),
                Some((next, _)),
            ) = (events.get(i + 1), events.get(i + 2), events.get(i + 3))
            {
                if let Some(open) = callout_open(t) {
                    out.push(Event::Html(open.into()));
                    quotes.push(true);
                    match next {
                        // The marker is the whole paragraph.
                        Event::End(TagEnd::Paragraph) => i += 4,
                        // More lines follow in the same paragraph.
                        Event::SoftBreak | Event::HardBreak => {
                            out.push(Event::Start(Tag::Paragraph));
                            i += 4;
                        }
                        _ => i += 3,
                    }
                    continue;
                }
            }
            quotes.push(false);
            out.push(ev.clone());
            i += 1;
            continue;
        }
        if let Event::End(TagEnd::BlockQuote(_)) = ev {
            if quotes.pop() == Some(true) {
                out.push(Event::Html("</div>".into()));
            } else {
                out.push(ev.clone());
            }
            i += 1;
            continue;
        }
        // A paragraph that is only a stat block tag (not `\{@…}`).
        if let (
            Event::Start(Tag::Paragraph),
            Some((Event::Text(t), _)),
            Some((Event::End(TagEnd::Paragraph), _)),
        ) = (ev, events.get(i + 1), events.get(i + 2))
        {
            let escaped = source
                .get(range.clone())
                .is_some_and(|raw| raw.trim_start().starts_with("\\{"));
            if let (Some((tag, name, src)), false) = (statblock_line(t), escaped) {
                out.push(Event::Html(statblock_html(tag, name, src, catalog).into()));
                i += 3;
                continue;
            }
        }
        // A catalog image.
        if let Event::Start(Tag::Image { dest_url, .. }) = ev {
            if let Some(path) = dest_url.strip_prefix(IMAGE_SCHEME) {
                let path = path.to_string();
                let mut alt = String::new();
                i += 1;
                while i < events.len() {
                    match &events[i].0 {
                        Event::End(TagEnd::Image) => break,
                        Event::Text(t) => alt.push_str(t),
                        _ => {}
                    }
                    i += 1;
                }
                i += 1;
                let html = match catalog {
                    Catalog::On => format!(
                        r#"<span class="image-wrapper" data-image-path="{}">{}</span>"#,
                        esc(&path),
                        esc(&alt)
                    ),
                    Catalog::Off => esc(&alt),
                };
                out.push(Event::InlineHtml(html.into()));
                continue;
            }
        }
        match ev {
            Event::Text(t) if !in_code && t.contains("{@") => {
                let html = match catalog {
                    Catalog::On => tags::tags(t),
                    Catalog::Off => esc(&tags::plain(t)),
                };
                out.push(Event::InlineHtml(html.into()));
            }
            _ => out.push(ev.clone()),
        }
        i += 1;
    }
    out
}

fn safe_url(url: CowStr<'_>) -> CowStr<'_> {
    let scheme = url
        .trim_start()
        .split(':')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if url.contains(':') && matches!(scheme.as_str(), "javascript" | "data" | "vbscript") {
        CowStr::from("#")
    } else {
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_common_markdown() {
        let html = to_html("# Title\n\n- one\n- two\n\n| a | b |\n|---|---|\n| 1 | 2 |\n");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<li>one</li>"));
        assert!(html.contains("<table>"));
    }

    #[test]
    fn raw_html_is_escaped() {
        let html = to_html("Hi <script>alert(1)</script>\n\n<img src=x onerror=alert(1)>\n");
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn script_links_lose_their_address() {
        let html =
            to_html("[x](javascript:alert(1)) [y](JavaScript:alert(1)) [ok](https://example.com)");
        assert!(!html.to_lowercase().contains("javascript:"));
        assert!(html.contains(r#"href="https://example.com""#));
    }

    #[test]
    fn callout_boxes() {
        let html = to_html(
            "> [!read-aloud]\n>\n> The cave is dark.\n\n> [!sidebar] Goblin Tactics\n> They flee at half HP.\n\n> A plain quote.\n",
        );
        assert!(
            html.contains("<div class=\"inset-readaloud\">\n<p>The cave is dark.</p>\n</div>"),
            "{html}"
        );
        assert!(
            html.contains("<div class=\"inset\"><h5>Goblin Tactics</h5>\n<p>They flee at half HP.</p>\n</div>"),
            "{html}"
        );
        assert!(
            html.contains("<blockquote>\n<p>A plain quote.</p>\n</blockquote>"),
            "{html}"
        );
        assert!(!html.contains("[!"), "{html}");
    }

    #[test]
    fn stat_block_lines_and_tags() {
        let html = to_html(
            "{@creature Klarg|LMoP}\n\n{@creatureFluff Fume Drake|DoSI}\n\n\\{@creature Mad Mary|CoS}\n\nMeet {@creature goblin|MM|two goblins}.\n",
        );
        assert!(html.contains(r#"<div class="mimir-statblock-line" data-ref-type="creature" data-ref-name="Klarg" data-ref-source="LMoP">"#), "{html}");
        assert!(html.contains("mimir-statblock-line--lore"), "{html}");
        // The escaped line is text, with its tag as a link.
        assert!(html.contains(r#"data-ref-name="Mad Mary""#), "{html}");
        assert_eq!(
            html.matches("mimir-statblock-line\"").count()
                + html.matches("mimir-statblock-line mimir").count(),
            2,
            "{html}"
        );
        assert!(
            html.contains("class=\"cross-ref-link creature-ref\""),
            "{html}"
        );
        assert!(html.contains(">two goblins</a>"), "{html}");
        // In code, a tag is text.
        let code = to_html("`{@creature x}`\n");
        assert!(code.contains("<code>{@creature x}</code>"), "{code}");
    }

    #[test]
    fn heading_ids_and_catalog_images() {
        let html = to_html(
            "## 1. Cave Mouth {#0be}\n\n![Cragmaw Hideout](<5etools:adventure/LMoP/map.webp>)\n",
        );
        assert!(
            html.contains(r#"<h2 id="0be">1. Cave Mouth</h2>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span class="image-wrapper" data-image-path="adventure/LMoP/map.webp">Cragmaw Hideout</span>"#),
            "{html}"
        );
    }

    #[test]
    fn a_players_view_points_at_nothing() {
        let html = to_html_with(
            "{@creature Klarg|LMoP}\n\nMeet {@creature goblin|MM|two goblins}.\n\n![Map](<5etools:adventure/X/map.png>)\n",
            Catalog::Off,
        );
        assert!(!html.contains("cross-ref-link"), "{html}");
        assert!(!html.contains("data-ref"), "{html}");
        assert!(!html.contains("data-image-path"), "{html}");
        assert!(html.contains("<p>Klarg</p>"), "{html}");
        assert!(html.contains("Meet two goblins."), "{html}");
    }

    #[test]
    fn statblock_lines_parse() {
        assert_eq!(
            statblock_line("{@creature Klarg|LMoP}"),
            Some(("creature", "Klarg", Some("LMoP")))
        );
        assert_eq!(
            statblock_line("{@item Trinket}"),
            Some(("item", "Trinket", None))
        );
        assert_eq!(statblock_line("{@dice 1d6}"), None);
        assert_eq!(statblock_line("{@creature goblin|MM|goblins}"), None);
        assert_eq!(statblock_line("{@b {@creature x}}"), None);
    }
}
