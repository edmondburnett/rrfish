use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Md,
    Org,
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
    pub text: String,
}
