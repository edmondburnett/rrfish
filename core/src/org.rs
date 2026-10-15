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

/// split an org document into sections, one per headline.
pub fn parse_org(source: &str) -> ParsedOrg {
    let mut builder = SectionBuilder::new();
    let mut title = None;
    let mut tags = Vec::new();
    let mut in_block = false;
    for (index, line) in source.lines().enumerate() {
        let line_number = index as u32 + 1;
        if in_block {
            if is_block_end(line) {
                in_block = false;
            } else {
                builder.push_text(line);
                builder.push_text("\n");
            }
        } else if let Some(args) = block_begin(line) {
            in_block = true;
            if let Some(lang) = src_lang(args) {
                builder.push_code_lang(lang);
            }
        } else if let Some((key, value)) = keyword(line) {
            match key.as_str() {
                "TITLE" => title = Some(value.to_string()),
                "FILETAGS" => tags = filetags(value),
                _ => {}
            }
        } else if let Some((level, heading)) = heading_level(line) {
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

/// `"#+begin_src rust :results output"` → `some("src rust :results output")`. case-insensitive.
fn block_begin(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let prefix = trimmed.get(..8)?;
    if !prefix.eq_ignore_ascii_case("#+begin_") {
        return None;
    }
    Some(&trimmed[8..])
}

/// `"#+end_src"` / `"#+end_quote"` → true.
fn is_block_end(line: &str) -> bool {
    match line.trim_start().get(..6) {
        Some(prefix) => prefix.eq_ignore_ascii_case("#+end_"),
        None => false,
    }
}

/// `"SRC rust :results output"` → `Some("rust")`; `"QUOTE"` or `"SRC :results x"` → `None`.
fn src_lang(block_args: &str) -> Option<&str> {
    let mut words = block_args.split_whitespace();
    let kind = words.next()?;
    if !kind.eq_ignore_ascii_case("src") {
        return None;
    }
    words.next().filter(|lang| !lang.starts_with(':'))
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

    #[test]
    fn keywords_are_case_insensitive() {
        assert_eq!(
            keyword("#+TITLE: Linux Audio"),
            Some(("TITLE".to_string(), "Linux Audio"))
        );
        assert_eq!(keyword("#+title: x"), Some(("TITLE".to_string(), "x")));
    }

    #[test]
    fn non_keywords() {
        assert_eq!(keyword("#+begin_src rust :results output"), None);
        assert_eq!(keyword("# a comment"), None);
        assert_eq!(keyword("plain: text"), None);
    }

    #[test]
    fn filetags_split_on_colons() {
        assert_eq!(filetags(":linux:audio:"), ["linux", "audio"]);
        assert!(filetags("").is_empty());
    }

    #[test]
    fn title_and_tags_are_read_and_keywords_stay_out_of_the_body() {
        let parsed = parse_org("#+TITLE: Linux Audio\n#+FILETAGS: :linux:audio:\n#+AUTHOR: me\nintro\n* H\nx\n");
        assert_eq!(parsed.title.as_deref(), Some("Linux Audio"));
        assert_eq!(parsed.tags, ["linux", "audio"]);
        assert_eq!(parsed.sections[0].text, "intro");
    }

    #[test]
    fn block_begin_returns_args() {
        assert_eq!(block_begin("#+BEGIN_SRC rust"), Some("SRC rust"));
        assert_eq!(block_begin("#+begin_quote"), Some("quote"));
        assert_eq!(block_begin("#+TITLE: x"), None);
        assert_eq!(block_begin(""), None);
    }

    #[test]
    fn block_end_is_case_insensitive() {
        assert!(is_block_end("#+END_SRC"));
        assert!(is_block_end("#+end_quote"));
        assert!(!is_block_end("text"));
    }

    #[test]
    fn src_lang_takes_the_word_after_src() {
        assert_eq!(src_lang("SRC rust :results output"), Some("rust"));
        assert_eq!(src_lang("src :results output"), None);
        assert_eq!(src_lang("SRC"), None);
        assert_eq!(src_lang("QUOTE"), None);
    }

    #[test]
    fn src_blocks_keep_code_and_record_language() {
        let org = "* A\n#+begin_src Rust\n* not a heading\nfn main() {}\n#+end_src\nafter\n";
        let parsed = parse_org(org);
        assert_eq!(parsed.sections.len(), 1);
        assert_eq!(parsed.sections[0].heading_path, ["A"]);
        assert_eq!(parsed.sections[0].text, "* not a heading\nfn main() {}\nafter");
        assert_eq!(parsed.sections[0].code_langs, ["rust"]);
    }
}
