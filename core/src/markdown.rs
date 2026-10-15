use crate::doc::{Link, LinkKind, Section};
use crate::section::SectionBuilder;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, LinkType, Options, Parser, Tag, TagEnd};
use serde::Deserialize;
use std::path::Path;

/// A parsed Markdown file.
#[derive(Debug)]
pub struct ParsedMarkdown {
    pub front_matter: Option<String>,
    pub first_h1: Option<String>,
    pub sections: Vec<Section>,
    pub links: Vec<Link>,
}

#[derive(Debug, Deserialize)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub date: Option<String>,
    pub updated: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Split a markdown document into sections, one per heading.
pub fn parse_markdown(source: &str) -> ParsedMarkdown {
    let mut builder = SectionBuilder::new();
    let mut front_matter: Option<String> = None;
    let mut in_front_matter = false;
    let parser = Parser::new_ext(
        source,
        Options::ENABLE_YAML_STYLE_METADATA_BLOCKS | Options::ENABLE_WIKILINKS,
    );
    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                builder.start_heading(level as u8, line_at(source, range.start))
            }
            Event::End(TagEnd::Heading(_)) => builder.end_heading(),
            Event::Start(Tag::MetadataBlock(_)) => {
                in_front_matter = true;
                front_matter = Some(String::new());
            }
            Event::End(TagEnd::MetadataBlock(_)) => in_front_matter = false,
            Event::Text(text) if in_front_matter => front_matter.get_or_insert_default().push_str(&text),
            Event::Start(Tag::Link {
                link_type, dest_url, ..
            }) => builder.push_link(to_link(link_type, dest_url)),
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => builder.push_code_lang(fence_lang(&info)),
            Event::Text(text) | Event::Code(text) => builder.push_text(&text),
            Event::SoftBreak | Event::HardBreak | Event::Start(Tag::Item) => builder.end_line(),
            Event::End(TagEnd::Paragraph | TagEnd::CodeBlock | TagEnd::List(_)) => builder.end_block(),
            _ => {}
        }
    }
    let built = builder.finish();
    ParsedMarkdown {
        front_matter,
        first_h1: built.first_h1,
        sections: built.sections,
        links: built.links,
    }
}

/// Deserialize raw front-matter YAML.
pub fn parse_front_matter(yaml: &str) -> Result<FrontMatter, serde_saphyr::Error> {
    serde_saphyr::from_str(yaml)
}

/// 1-based line number of the given byte offset in `source`.
fn line_at(source: &str, byte_offset: usize) -> u32 {
    let before = &source[..byte_offset];
    let newline_count = before.matches('\n').count();
    newline_count as u32 + 1
}

/// Convert a pulldown-cmark link into our format-neutral `Link`.
fn to_link(link_type: LinkType, dest_url: CowStr) -> Link {
    let kind = match link_type {
        LinkType::WikiLink { .. } => LinkKind::Wiki,
        _ => LinkKind::Markdown,
    };
    Link {
        target: dest_url.to_string(),
        kind,
    }
}

/// The language from a fence info string: "rust,ignore" or "rust title=x" → "rust".
fn fence_lang(info: &str) -> &str {
    info.split(|c: char| c.is_whitespace() || c == ',').next().unwrap_or("")
}

/// Pick a note's title
pub fn resolve_title(front_matter_title: Option<&str>, first_h1: Option<&str>, path: &Path) -> String {
    if let Some(front_matter_title) = front_matter_title {
        return front_matter_title.to_string();
    }
    if let Some(first_h1) = first_h1 {
        return first_h1.to_string();
    }
    file_stem(path)
}

fn file_stem(path: &Path) -> String {
    path.file_stem().unwrap_or_default().to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(md: &str) -> Vec<Vec<String>> {
        parse_markdown(md)
            .sections
            .into_iter()
            .map(|s| s.heading_path)
            .collect()
    }

    #[test]
    fn heading_then_paragraph() {
        let sections = parse_markdown("# Title\n\nHello *world*").sections;
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].heading_path, ["Title"]);
        assert_eq!(sections[0].text, "Hello world");
    }

    #[test]
    fn nested_headings_build_a_path() {
        assert_eq!(paths("# A\n\na\n\n## B\n\nb"), [vec!["A"], vec!["A", "B"]]);
    }

    #[test]
    fn sibling_heading_replaces_previous() {
        assert_eq!(paths("# A\n\n## B\n\nb\n\n## C\n\nc"), [vec!["A", "B"], vec!["A", "C"]]);
    }

    #[test]
    fn shallower_heading_pops_deeper_ones() {
        assert_eq!(
            paths("# A\n\n## B\n\n### C\n\nc\n\n# D\n\nd"),
            [vec!["A", "B", "C"], vec!["D"]]
        );
    }

    #[test]
    fn text_before_first_heading_has_empty_path() {
        assert_eq!(paths("intro\n\n# A\n\na"), [vec![], vec!["A"]]);
    }

    #[test]
    fn line_at_counts_newlines_before_offset() {
        let src = "a\nbc\n\nd";
        assert_eq!(line_at(src, 0), 1);
        assert_eq!(line_at(src, 2), 2);
        assert_eq!(line_at(src, 7), 4);
    }

    #[test]
    fn sections_record_their_heading_line() {
        let lines: Vec<u32> = parse_markdown("intro\n\n# A\n\na\n\n## B\n\nb")
            .sections
            .into_iter()
            .map(|s| s.line)
            .collect();
        assert_eq!(lines, [1, 3, 7]);
    }

    #[test]
    fn front_matter_is_captured_not_indexed() {
        let parsed = parse_markdown("---\ntitle: AI notes\ntags: [ml]\n---\nbody\n");
        assert_eq!(parsed.front_matter.as_deref(), Some("title: AI notes\ntags: [ml]\n"));
        assert_eq!(parsed.sections.len(), 1);
        assert!(parsed.sections[0].heading_path.is_empty());
        assert_eq!(parsed.sections[0].text, "body");
    }

    #[test]
    fn file_without_front_matter_has_none() {
        assert_eq!(parse_markdown("# A\n\nbody").front_matter, None);
    }

    #[test]
    fn front_matter_reads_typical_fields() {
        let yaml = "title: Linux Audio\ndate: Thu Sep 05 2024 11:07:30 AM PDT\ntags:\n    - computers\n    - linux\n";
        let fm = parse_front_matter(yaml).unwrap();
        assert_eq!(fm.title.as_deref(), Some("Linux Audio"));
        assert_eq!(fm.date.as_deref(), Some("Thu Sep 05 2024 11:07:30 AM PDT"));
        assert_eq!(fm.updated, None);
        assert_eq!(fm.tags, ["computers", "linux"]);
    }

    #[test]
    fn missing_fields_default_and_unknown_fields_are_ignored() {
        let fm = parse_front_matter("title: Only\nauthor: me\n").unwrap();
        assert_eq!(fm.title.as_deref(), Some("Only"));
        assert_eq!(fm.date, None);
        assert!(fm.tags.is_empty());
    }

    #[test]
    fn invalid_front_matter_is_an_error() {
        assert!(parse_front_matter("title: x\ntags: ml\n").is_err());
        assert!(parse_front_matter("title: Rust: lifetimes\n").is_err());
    }

    #[test]
    fn first_h1_is_recorded() {
        let parsed = parse_markdown("intro\n\n## Not this\n\n# Title\n\n# Later");
        assert_eq!(parsed.first_h1.as_deref(), Some("Title"));
    }

    #[test]
    fn no_h1_means_none() {
        assert_eq!(parse_markdown("## Only h2\n\nbody").first_h1, None);
    }

    #[test]
    fn title_prefers_front_matter_then_h1_then_filename() {
        let path = Path::new("notes/linux-audio.md");
        assert_eq!(resolve_title(Some("FM"), Some("H1"), path), "FM");
        assert_eq!(resolve_title(None, Some("H1"), path), "H1");
        assert_eq!(resolve_title(None, None, path), "linux-audio");
    }

    fn targets(md: &str) -> Vec<(String, LinkKind)> {
        parse_markdown(md)
            .links
            .into_iter()
            .map(|l| (l.target, l.kind))
            .collect()
    }

    #[test]
    fn inline_and_autolinks_are_markdown_links() {
        assert_eq!(
            targets("see [docs](https://a.example) and <https://b.example>"),
            [
                ("https://a.example".to_string(), LinkKind::Markdown),
                ("https://b.example".to_string(), LinkKind::Markdown),
            ]
        );
    }

    #[test]
    fn wikilinks_keep_target_not_label() {
        assert_eq!(
            targets("[[Linux Audio]] and [[pipewire|the PW note]]"),
            [
                ("Linux Audio".to_string(), LinkKind::Wiki),
                ("pipewire".to_string(), LinkKind::Wiki),
            ]
        );
    }

    #[test]
    fn link_text_stays_in_section_body() {
        let parsed = parse_markdown("read [the docs](https://a.example) and [[pipewire|PW]]");
        assert_eq!(parsed.sections[0].text, "read the docs and PW");
    }

    fn langs(md: &str) -> Vec<Vec<String>> {
        parse_markdown(md).sections.into_iter().map(|s| s.code_langs).collect()
    }

    #[test]
    fn fenced_code_records_language_and_keeps_code_text() {
        let parsed = parse_markdown("# A\n\n```rust\nfn main() {}\n```\n");
        assert_eq!(parsed.sections[0].code_langs, ["rust"]);
        assert!(parsed.sections[0].text.contains("fn main() {}"));
    }

    #[test]
    fn fence_info_extras_are_dropped_and_lowercased() {
        assert_eq!(langs("```Rust,ignore\nx\n```"), [vec!["rust"]]);
        assert_eq!(langs("```python title=x.py\nx\n```"), [vec!["python"]]);
    }

    #[test]
    fn unlabeled_and_indented_code_have_no_language() {
        assert_eq!(langs("```\nx\n```\n\n    indented\n"), [Vec::<String>::new()]);
    }

    #[test]
    fn languages_are_deduped_and_stay_in_their_section() {
        let md = "# A\n\n```rust\na\n```\n\n```rust\nb\n```\n\n# B\n\n```lua\nc\n```\n";
        assert_eq!(langs(md), [vec!["rust"], vec!["lua"]]);
    }

    fn body(md: &str) -> String {
        parse_markdown(md).sections[0].text.clone()
    }

    #[test]
    fn list_items_are_separate_lines() {
        assert_eq!(body("- game one\n- game two\n"), "game one\ngame two");
        assert_eq!(body("1. one\n2. two\n"), "one\ntwo");
    }

    #[test]
    fn blocks_are_separated_by_one_blank_line() {
        assert_eq!(body("first\nsame para\n\nsecond\n"), "first\nsame para\n\nsecond");
        assert_eq!(body("- a\n- b\n\nafter list\n"), "a\nb\n\nafter list");
        assert_eq!(body("```sh\nls\n```\nafter code\n"), "ls\n\nafter code");
    }
}
