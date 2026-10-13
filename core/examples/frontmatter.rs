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

    let (mut ok, mut missing, mut invalid) = (0, 0, 0);
    for file in files.iter().filter(|f| f.format == Format::Md) {
        // match check(file) { ... } — bump a counter per arm;
        // print rel_path for Missing and Invalid (plus the reason for Invalid)
    }
    println!("\n{ok} ok, {missing} missing, {invalid} invalid");
}

/// Read one note and classify its front matter.
fn check(file: &NoteFile) -> Status {
    // 1. read the file
    // 2. let-else on parse_markdown(..).front_matter → Missing
    // 3. match parse_front_matter(&yaml): Ok(_) → Ok, Err(e) → Invalid(first_line(..))
    todo!()
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or_default().to_string()
}
