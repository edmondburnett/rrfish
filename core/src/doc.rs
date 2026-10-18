use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Md,
    Org,
}

impl Format {
    /// Short name stored in the index: `"md"` or `"org"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Format::Md => "md",
            Format::Org => "org",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NoteFile {
    pub path: PathBuf,
    pub rel_path: PathBuf,
    pub format: Format,
    pub collection: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub heading_path: Vec<String>,
    pub line: u32,
    pub text: String,
    pub code_langs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub target: String,
    pub kind: LinkKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkKind {
    /// `[[target]]` or `[[target|label]]`
    Wiki,
    /// `[label](target)`, `<https://...>`, reference links
    Markdown,
    /// `[[target][label]]` or `[[target]]`
    Org,
}
