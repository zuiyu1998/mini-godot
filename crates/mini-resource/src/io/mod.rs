use std::path::{Path, PathBuf};

mod file;
mod path;
mod reader;
mod source;
mod writer;

pub use file::*;
pub use path::*;
pub use reader::*;
pub use source::*;
pub use writer::*;

/// Appends `.meta` to the given path.
pub(crate) fn get_meta_path(path: &Path) -> PathBuf {
    let mut meta_path = path.to_path_buf();
    let mut extension = path.extension().unwrap_or_default().to_os_string();
    extension.push(".meta");
    meta_path.set_extension(extension);
    meta_path
}
