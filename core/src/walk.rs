use crate::doc::{Format, NoteFile};
use ignore::WalkBuilder;
use std::path::Path;

/// Walks the path returning a vector of filesnames
pub fn discover(root: &Path) -> Vec<NoteFile> {
    WalkBuilder::new(root)
        .build()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let format = format_of(entry.path())?;
            let path = entry.path().to_path_buf();
            let rel_path = path.strip_prefix(root).ok()?.to_path_buf();
            let collection = collection_of(&rel_path);
            Some(NoteFile {
                path,
                rel_path,
                format,
                collection,
            })
        })
        .collect()
}

/// return valid file extensions or None
fn format_of(path: &Path) -> Option<Format> {
    let extension = path.extension()?;
    match extension.to_str() {
        Some("md") => Some(Format::Md),
        Some("org") => Some(Format::Org),
        _ => None,
    }
}

/// returns first-level directory name (a "collection"), or None
fn collection_of(rel_path: &Path) -> Option<String> {
    let mut components = rel_path.components();
    let first = components.next()?;
    components.next()?;
    Some(first.as_os_str().to_str()?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_from_extension() {
        assert_eq!(format_of(Path::new("a/b.md")), Some(Format::Md));
        assert_eq!(format_of(Path::new("b.org")), Some(Format::Org));
        assert!(format_of(Path::new("b.txt")).is_none());
        assert!(format_of(Path::new("Makefile")).is_none());
    }

    #[test]
    fn collection_is_first_dir() {
        assert_eq!(collection_of(Path::new("tech/ai.md")).as_deref(), Some("tech"));
        assert_eq!(collection_of(Path::new("tech/deep/x.md")).as_deref(), Some("tech"));
        assert_eq!(collection_of(Path::new("todo.md")), None);
    }
}
