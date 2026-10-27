use serde::Serialize;

use crate::chunk::{MAX_CHUNK_BYTES, chunk_section};
use crate::doc::NoteFile;
use crate::note::Note;

/// One indexed document: a chunk plus the note-level metadata it belongs to.
#[derive(Debug, Serialize)]
pub struct IndexDoc {
    /// `"<rel_path>#<chunk number>"`, stable across re-indexing of unchanged files.
    pub id: String,
    pub path: String,
    pub collection: Option<String>,
    pub format: &'static str,
    pub title: String,
    pub tags: Vec<String>,
    pub heading_path: Vec<String>,
    pub line: u32,
    pub text: String,
    pub code_langs: Vec<String>,
}

/// Chunk every section of a note and attach the note's metadata to each chunk.
pub fn index_docs(file: &NoteFile, note: &Note) -> Vec<IndexDoc> {
    let path = file.rel_path.display().to_string();
    let mut docs = Vec::new();
    for section in &note.sections {
        for chunk in chunk_section(section, MAX_CHUNK_BYTES) {
            docs.push(IndexDoc {
                id: format!("{path}#{}", docs.len()),
                path: path.clone(),
                collection: file.collection.clone(),
                format: file.format.as_str(),
                title: note.title.clone(),
                tags: note.tags.clone(),
                heading_path: chunk.heading_path,
                line: chunk.line,
                text: chunk.text,
                code_langs: chunk.code_langs,
            });
        }
    }
    docs
}

/// The text sent to the embedding model: title and heading path as context, then the chunk.
pub fn embed_text(doc: &IndexDoc) -> String {
    with_context(&doc.title, &doc.heading_path, &doc.text)
}

/// `"Title > Heading > Sub\n\ntext"`, skipping a heading that just repeats the title.
pub fn with_context(title: &str, heading_path: &[String], text: &str) -> String {
    let mut context = vec![title];
    for heading in heading_path {
        if heading != title {
            context.push(heading);
        }
    }
    format!("{}\n\n{}", context.join(" > "), text)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::doc::{Format, Section};

    fn file() -> NoteFile {
        NoteFile {
            path: PathBuf::from("/notes/linux/audio.md"),
            rel_path: PathBuf::from("linux/audio.md"),
            format: Format::Md,
            collection: Some("linux".to_string()),
        }
    }

    fn section(heading_path: &[&str], line: u32, text: &str) -> Section {
        Section {
            heading_path: heading_path.iter().map(|h| h.to_string()).collect(),
            line,
            text: text.to_string(),
            code_langs: vec![],
        }
    }

    fn note() -> Note {
        Note {
            title: "Linux Audio".to_string(),
            tags: vec!["linux".to_string()],
            sections: vec![
                section(&["Linux Audio"], 1, "intro"),
                section(&["Linux Audio", "PipeWire"], 5, "use pw-top"),
            ],
            links: vec![],
        }
    }

    #[test]
    fn one_doc_per_chunk_with_note_metadata() {
        let docs = index_docs(&file(), &note());
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[1].id, "linux/audio.md#1");
        assert_eq!(docs[1].path, "linux/audio.md");
        assert_eq!(docs[1].collection.as_deref(), Some("linux"));
        assert_eq!(docs[1].format, "md");
        assert_eq!(docs[1].title, "Linux Audio");
        assert_eq!(docs[1].tags, ["linux"]);
        assert_eq!(docs[1].line, 5);
    }

    #[test]
    fn embed_text_adds_context_without_repeating_the_title() {
        let docs = index_docs(&file(), &note());
        assert_eq!(embed_text(&docs[1]), "Linux Audio > PipeWire\n\nuse pw-top");
    }
}
