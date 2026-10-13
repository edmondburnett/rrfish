use std::collections::BTreeMap;
use std::path::Path;

use rrfish_core::doc::{Format, NoteFile};
use rrfish_core::markdown::{parse_front_matter, parse_markdown};
use rrfish_core::walk::discover;

enum Status {
    Ok,
    Missing,
    Invalid(String),
}

fn main() {
    let root = std::env::args().nth(1).expect("usage: frontmatter <notes-dir>");
    let files = discover(Path::new(&root));
    let mut missing_per_collection: BTreeMap<&str, usize> = BTreeMap::new();

    let (mut ok, mut missing, mut invalid) = (0, 0, 0);
    for file in files.iter().filter(|f| f.format == Format::Md) {
        match check(file) {
            Status::Ok => ok += 1,
            Status::Missing => {
                missing += 1;
                *missing_per_collection
                    .entry(file.collection.as_deref().unwrap_or("(root)"))
                    .or_default() += 1;
            }
            Status::Invalid(reason) => {
                invalid += 1;
                println!("invalid  {}\n         {reason}", file.rel_path.display());
            }
        }
    }
    println!("\n{ok} ok, {missing} missing frontmatter, {invalid} invalid");
    for (name, count) in missing_per_collection {
        println!("  {name}, {count}")
    }
}

/// Read one note and classify its front matter.
fn check(file: &NoteFile) -> Status {
    let source = std::fs::read_to_string(&file.path).expect("read note");
    let Some(yaml) = parse_markdown(&source).front_matter else {
        return Status::Missing;
    };
    match parse_front_matter(&yaml) {
        Ok(_) => Status::Ok,
        Err(e) => Status::Invalid(first_line(&e.to_string())),
    }
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or_default().to_string()
}
