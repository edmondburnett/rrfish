use crate::doc::Section;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

/// Split a markdown document into sections, one per heading.
pub fn parse_markdown(source: &str) -> Vec<Section> {
    let mut builder = SectionBuilder::new();
    for (event, range) in Parser::new(source).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => builder.start_heading(level, line_at(source, range.start)),
            Event::End(TagEnd::Heading(_)) => builder.end_heading(),
            Event::Text(text) | Event::Code(text) => builder.push_text(&text),
            Event::SoftBreak | Event::HardBreak | Event::End(TagEnd::Paragraph) => builder.push_text("\n"),
            _ => {}
        }
    }
    builder.finish()
}

/// 1-based line number of the given byte offset in `source`.
fn line_at(source: &str, byte_offset: usize) -> u32 {
    let start = &source[..byte_offset];
    let newline_count = start.matches('\n').count();
    newline_count as u32 + 1
}

#[derive(Default)]
struct SectionBuilder {
    sections: Vec<Section>,
    section_line: u32,
    headings: Vec<(HeadingLevel, String)>,
    current_text: String,
    in_heading: Option<(HeadingLevel, String)>,
}

impl SectionBuilder {
    fn new() -> Self {
        Self {
            section_line: 1,
            ..Default::default()
        }
    }

    fn start_heading(&mut self, level: HeadingLevel, line: u32) {
        self.close_section();
        self.section_line = line;
        self.in_heading = Some((level, String::new()));
    }

    fn end_heading(&mut self) {
        let Some((level, title)) = self.in_heading.take() else {
            return;
        };
        self.headings.retain(|(existing_level, _)| *existing_level < level);
        self.headings.push((level, title.trim().to_string()));
    }

    /// Text goes to the heading title if inside one, else to the section body.
    fn push_text(&mut self, text: &str) {
        if let Some((_level, title)) = &mut self.in_heading {
            title.push_str(text);
        } else {
            self.current_text.push_str(text);
        }
    }

    fn close_section(&mut self) {
        let current_text = std::mem::take(&mut self.current_text);
        if current_text.trim().is_empty() {
            return;
        }

        let mut heading_path = Vec::new();
        for (_, title) in &self.headings {
            heading_path.push(title.clone());
        }

        self.sections.push(Section {
            heading_path,
            line: self.section_line,
            text: current_text.trim().to_string(),
        });
    }

    fn finish(mut self) -> Vec<Section> {
        self.close_section();
        self.sections
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(md: &str) -> Vec<Vec<String>> {
        parse_markdown(md).into_iter().map(|s| s.heading_path).collect()
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
            .into_iter()
            .map(|s| s.line)
            .collect();
        assert_eq!(lines, [1, 3, 7]);
    }
}
