use std::path::PathBuf;

pub enum Format {
    Md,
    Org,
}

pub struct NoteFile {
    pub path: PathBuf,
    pub rel_path: PathBuf,
    pub format: Format,
    pub collection: Option<String>,
}
