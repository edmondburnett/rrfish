use crate::doc::Section;
use crate::section::SectionBuilder;

/// A parsed org file.
#[derive(Debug)]
pub struct ParsedOrg {
    /// From `#+TITLE:`
    pub title: Option<String>,
    /// From `#+FILETAGS:`
    pub tags: Vec<String>,
    pub first_h1: Option<String>,
    pub sections: Vec<Section>,
}

/// Split an org document into sections, one per headline.
pub fn parse_org(source: &str) -> ParsedOrg {
    let mut builder = SectionBuilder::new();
    let mut title = None;
    let mut tags = Vec::new();
    for (index, line) in source.lines().enumerate() {
        if let Some((key, value)) = keyword(line) {
            match key.as_str() {
                "TITLE" => title = Some(value.to_string()),
                "FILETAGS" => tags = filetags(value),
                _ => {}
            }
        }
        let line_number = index as u32 + 1;
        if let Some((level, heading)) = heading_level(line) {
            builder.start_heading(level, line_number);
            builder.push_text(heading);
            builder.end_heading();
        } else {
            builder.push_text(line);
            builder.push_text("\n");
        }
    }
    let built = builder.finish();
    ParsedOrg {
        title,
        tags,
        first_h1: built.first_h1,
        sections: built.sections,
    }
}

fn keyword(line: &str) -> Option<(String, &str)> {
    let rest = line.trim_start().strip_prefix("#+")?;
    let (key, value) = rest.split_once(':')?;
    if key.is_empty() || key.contains(char::is_whitespace) {
        return None;
    }
    Some((key.to_ascii_uppercase(), value.trim()))
}

fn filetags(value: &str) -> Vec<String> {
    let mut tags = Vec::new();
    for tag in value.split(':') {
        let tag = tag.trim();
        if !tag.trim().is_empty() {
            tags.push(tag.to_string());
        }
    }
    tags
}

/// `"** Title"` → `Some((2, "Title"))`; anything that isn't a headline → `None`.
fn heading_level(line: &str) -> Option<(u8, &str)> {
    let rest = line.trim_start_matches('*');
    let stars = line.len() - rest.len();
    if stars == 0 {
        return None;
    }
    let title = rest.strip_prefix(' ')?;
    Some((stars as u8, title.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(org: &str) -> Vec<Vec<String>> {
        parse_org(org).sections.into_iter().map(|s| s.heading_path).collect()
    }

    #[test]
    fn headline_levels() {
        assert_eq!(heading_level("* A"), Some((1, "A")));
        assert_eq!(heading_level("*** Deep  "), Some((3, "Deep")));
    }

    #[test]
    fn non_headlines() {
        assert_eq!(heading_level("*bold* text"), None);
        assert_eq!(heading_level(" * indented list item"), None);
        assert_eq!(heading_level("plain"), None);
        assert_eq!(heading_level(""), None);
    }

    #[test]
    fn headlines_build_paths() {
        let org = "intro\n* A\nbody a\n** B\nbody b\n* C\nc\n";
        assert_eq!(paths(org), [vec![], vec!["A"], vec!["A", "B"], vec!["C"]]);
    }

    #[test]
    fn sections_record_their_headline_line() {
        let org = "intro\n* A\nbody a\n** B\nbody b\n* C\nc\n";
        let lines: Vec<u32> = parse_org(org).sections.into_iter().map(|s| s.line).collect();
        assert_eq!(lines, [1, 2, 4, 6]);
    }

    #[test]
    fn body_keeps_line_breaks() {
        let parsed = parse_org("* A\nline one\nline two\n");
        assert_eq!(parsed.sections[0].text, "line one\nline two");
        assert_eq!(parsed.first_h1.as_deref(), Some("A"));
    }
}
