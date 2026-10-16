use std::collections::BTreeMap;
use std::path::Path;

use rrfish_core::chunk::{MAX_CHUNK_BYTES, chunk_section};
use rrfish_core::doc::{Format, Link, NoteFile, Section};
use rrfish_core::markdown::parse_markdown;
use rrfish_core::org::parse_org;
use rrfish_core::walk::discover;

/// The parts of a parsed file this report needs, from either parser.
struct Parsed {
    sections: Vec<Section>,
    links: Vec<Link>,
}

fn main() {
    let root = std::env::args().nth(1).expect("usage: parse <notes-dir>");
    let files = discover(Path::new(&root));

    let mut empty_files = Vec::new();
    let mut sizes: Vec<(usize, String)> = Vec::new();
    let mut chunk_sizes: Vec<(usize, String)> = Vec::new();
    let mut langs: BTreeMap<String, usize> = BTreeMap::new();
    let mut link_kinds: BTreeMap<String, usize> = BTreeMap::new();

    for file in &files {
        let parsed = parse(file);
        if parsed.sections.is_empty() {
            empty_files.push(file.rel_path.display().to_string());
        }
        for section in &parsed.sections {
            let location = format!("{}:{}", file.rel_path.display(), section.line);
            sizes.push((section.text.chars().count(), location.clone()));
            for chunk in chunk_section(section, MAX_CHUNK_BYTES) {
                chunk_sizes.push((chunk.text.len(), format!("{}:{}", file.rel_path.display(), chunk.line)));
            }
            for lang in &section.code_langs {
                *langs.entry(lang.clone()).or_default() += 1;
            }
        }
        for link in &parsed.links {
            *link_kinds.entry(format!("{:?}", link.kind)).or_default() += 1;
        }
    }

    println!("{} files, {} sections", files.len(), sizes.len());
    print_list("files with no sections", &empty_files);
    print_sizes("section", &mut sizes);
    println!("\n{} chunks", chunk_sizes.len());
    print_sizes("chunk", &mut chunk_sizes);
    print_counts("code languages", &langs);
    print_counts("links", &link_kinds);
}

/// Read and parse one note with the parser for its format.
fn parse(file: &NoteFile) -> Parsed {
    let source = std::fs::read_to_string(&file.path).expect("read note");
    match file.format {
        Format::Md => {
            let parsed = parse_markdown(&source);
            Parsed {
                sections: parsed.sections,
                links: parsed.links,
            }
        }
        Format::Org => {
            let parsed = parse_org(&source);
            Parsed {
                sections: parsed.sections,
                links: parsed.links,
            }
        }
    }
}

/// Median, p90 and max section size, then the 10 largest sections.
fn print_sizes(label: &str, sizes: &mut [(usize, String)]) {
    if sizes.is_empty() {
        return;
    }
    sizes.sort();
    let median = sizes[sizes.len() / 2].0;
    let p90 = sizes[sizes.len() * 9 / 10].0;
    let max = sizes[sizes.len() - 1].0;
    println!("\n{label} size: median {median}, p90 {p90}, max {max}");
    println!("largest {label}s:");
    for (size, location) in sizes.iter().rev().take(10) {
        println!("  {size:>7}  {location}");
    }
}

fn print_list(title: &str, items: &[String]) {
    println!("\n{title}: {}", items.len());
    for item in items {
        println!("  {item}");
    }
}

fn print_counts(title: &str, counts: &BTreeMap<String, usize>) {
    println!("\n{title}:");
    for (name, count) in counts {
        println!("  {name:<20} {count}");
    }
}
