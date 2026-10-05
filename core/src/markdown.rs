use crate::doc::Section;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

/// Split a markdown document into sections, one per heading.
pub fn parse_markdown(source: &str) -> Vec<Section> {}

#[derive(Default)]
struct SectionBuilder {
    sections: Vec<Section>,
    headings: Vec<(HeadingLevel, String)>,
    current_text: String,
    in_heading: Option<(HeadingLevel, String)>,
}

impl SectionBuilder {
    fn start_heading(&mut self, level: HeadingLevel) {}
    fn end_heading(&mut self) {}
    fn push_text(&mut self, text: &str) {}
    fn close_section(&mut self) {}
    fn finish(self) -> Vec<Section> {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(md: &str) -> Vec<Vec<String>> {
        parse_markdown(md)
            .into_iter()
            .map(|s| s.heading_path)
            .collect()
    }

    #[test]
    fn heading_then_paragraph() {
        let sections = parse_markdown("# Title\n\nHello *world*");
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
        assert_eq!(
            paths("# A\n\n## B\n\nb\n\n## C\n\nc"),
            [vec!["A", "B"], vec!["A", "C"]]
        );
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
}
