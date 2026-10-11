//! Markdown to HTML for the document viewer.
//!
//! Raw HTML in the markdown is escaped, not passed through: the HTML goes
//! into the page with `inner_html`. Links with a `javascript:` (or `data:`,
//! `vbscript:`) address lose the address.

use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag};

/// Render markdown to HTML that is safe to put in the page.
pub fn to_html(source: &str) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;
    let events = Parser::new_ext(source, options).map(|event| match event {
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
    });
    let mut out = String::new();
    html::push_html(&mut out, events);
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
}
