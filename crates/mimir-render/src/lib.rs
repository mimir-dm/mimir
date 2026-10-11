//! 5etools JSON to HTML (MIMIR-T-0723): the `{@tag}` inline tags
//! ([`tags`]), entries in a detail view ([`entries`]), a book section in
//! the reader ([`book`]), and the detail of each kind of catalog entry
//! ([`detail`]). A port of the desktop's TypeScript formatters, with two
//! changes: text and attributes are escaped, and tags may nest.
//!
//! A cross-reference is `<a href="#" class="cross-ref-link {type}-ref"
//! data-ref-type data-ref-name data-ref-source>`: the web app opens it.

pub mod book;
pub mod entries;
pub mod tags;

/// Escape text for HTML content or a quoted attribute.
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// A cross-reference link.
pub fn link(kind: &str, name: &str, source: &str, display_html: &str) -> String {
    format!(
        r##"<a href="#" class="cross-ref-link {kind}-ref" data-ref-type="{kind}" data-ref-name="{}" data-ref-source="{}">{display_html}</a>"##,
        esc(name),
        esc(source)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes() {
        assert_eq!(
            esc(r#"<a href="x">&'"#),
            "&lt;a href=&quot;x&quot;&gt;&amp;&#39;"
        );
    }

    #[test]
    fn links() {
        let l = link("spell", "Fire \"Bolt\"", "PHB", "Fire Bolt");
        assert!(l.contains(r#"data-ref-name="Fire &quot;Bolt&quot;""#));
        assert!(l.ends_with(">Fire Bolt</a>"));
    }
}
