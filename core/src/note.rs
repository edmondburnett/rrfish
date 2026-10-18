use std::fs::read_to_string;
use std::path::Path;

use crate::doc::{Format, Link, NoteFile, Section};
use crate::markdown::{FrontMatter, ParsedMarkdown, parse_front_matter, parse_markdown, resolve_title};
use crate::org::{ParsedOrg, parse_org};

/// A parsed note, the same shape whichever format it came from.
#[derive(Debug)]
pub struct Note {
    pub title: String,
    pub tags: Vec<String>,
    pub sections: Vec<Section>,
    pub links: Vec<Link>,
}

/// Read a note from disk and parse it with the parser for its format.
pub fn read_note(file: &NoteFile) -> std::io::Result<Note> {
    let source = read_to_string(&file.path)?;
    let note = match file.format {
        Format::Md => from_markdown(parse_markdown(&source), &file.path),
        Format::Org => from_org(parse_org(&source), &file.path),
    };
    Ok(note)
}

fn from_markdown(parsed: ParsedMarkdown, path: &Path) -> Note {
    let front_matter = front_matter_of(&parsed);
    let (fm_title, tags) = match front_matter {
        Some(fm) => (fm.title, fm.tags),
        None => (None, Vec::new()),
    };
    Note {
        title: resolve_title(fm_title.as_deref(), parsed.first_h1.as_deref(), path),
        tags,
        sections: parsed.sections,
        links: parsed.links,
    }
}

/// The note's front matter, if it has some and it parses.
fn front_matter_of(parsed: &ParsedMarkdown) -> Option<FrontMatter> {
    let yaml = parsed.front_matter.as_deref()?;
    parse_front_matter(yaml).ok()
}

fn from_org(parsed: ParsedOrg, path: &Path) -> Note {
    Note {
        title: resolve_title(parsed.title.as_deref(), parsed.first_h1.as_deref(), path),
        tags: parsed.tags,
        sections: parsed.sections,
        links: parsed.links,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_title_and_tags_come_from_front_matter() {
        let parsed = parse_markdown("---\ntitle: Linux Audio\ntags: [linux]\n---\n# Heading\n\nbody\n");
        let note = from_markdown(parsed, Path::new("x/linux-audio.md"));
        assert_eq!(note.title, "Linux Audio");
        assert_eq!(note.tags, ["linux"]);
    }

    #[test]
    fn markdown_without_front_matter_falls_back_to_h1() {
        let note = from_markdown(parse_markdown("# Heading\n\nbody\n"), Path::new("x/n.md"));
        assert_eq!(note.title, "Heading");
        assert!(note.tags.is_empty());
    }

    #[test]
    fn invalid_front_matter_still_gives_a_note() {
        let parsed = parse_markdown("---\ntitle: Rust: lifetimes\n---\nbody\n");
        let note = from_markdown(parsed, Path::new("x/rust-lifetimes.md"));
        assert_eq!(note.title, "rust-lifetimes");
    }

    #[test]
    fn org_title_and_tags_come_from_keywords() {
        let parsed = parse_org("#+TITLE: Org Note\n#+FILETAGS: :a:b:\n* H\nx\n");
        let note = from_org(parsed, Path::new("x/n.org"));
        assert_eq!(note.title, "Org Note");
        assert_eq!(note.tags, ["a", "b"]);
    }
}
