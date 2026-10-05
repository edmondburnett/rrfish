use std::collections::BTreeMap;
use std::path::Path;

use rrfish_core::{doc::Format, walk::discover};

fn main() {
    let root = std::env::args().nth(1).expect("usage: discover <notes-dir>");
    let files = discover(Path::new(&root));

    let md = files.iter().filter(|f| f.format == Format::Md).count();
    let org = files.len() - md;
    println!("{} files ({md} md, {org} org)", files.len());

    let mut per_collection: BTreeMap<&str, usize> = BTreeMap::new();
    for f in &files {
        let name = f.collection.as_deref().unwrap_or("(root)");
        *per_collection.entry(name).or_default() += 1;
    }
    for (name, count) in per_collection {
        println!("  {name:<20} {count}");
    }
}
