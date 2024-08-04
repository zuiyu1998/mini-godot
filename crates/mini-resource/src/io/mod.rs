use std::{
    future::Future,
    io::Error as IoError,
    path::{Path, PathBuf},
    pin::Pin,
};
use thiserror::Error;

mod reader;
mod writer;

mod fs;

pub use reader::*;
pub use writer::*;

pub type ResourceIoFuture<'a, V> = Pin<Box<dyn Future<Output = V> + Send + 'a>>;

/// Appends `.meta` to the given path.
pub(crate) fn get_meta_path(path: &Path) -> PathBuf {
    let mut meta_path = path.to_path_buf();
    let mut extension = path.extension().unwrap_or_default().to_os_string();
    extension.push(".meta");
    meta_path.set_extension(extension);
    meta_path
}

#[derive(Debug, Error)]
pub enum FileLoadError {
    #[error("{0}")]
    IoError(#[from] IoError),
    #[error("{0}")]
    Custom(String),
}

pub trait ResourceIo: Send + Sync + 'static {
    fn exists<'a>(&'a self, path: &'a PathBuf) -> ResourceIoFuture<'a, bool>;

    fn load_file<'a>(
        &'a self,
        path: &'a PathBuf,
    ) -> ResourceIoFuture<'a, Result<Vec<u8>, FileLoadError>>;

    fn is_file<'a>(&'a self, path: &'a PathBuf) -> ResourceIoFuture<'a, bool>;

    fn is_dir<'a>(&'a self, path: &'a PathBuf) -> ResourceIoFuture<'a, bool>;
}

pub struct FsResourceIo;

impl ResourceIo for FsResourceIo {
    fn exists<'a>(&'a self, path: &'a PathBuf) -> ResourceIoFuture<'a, bool> {
        Box::pin(fs::exists(path))
    }

    fn load_file<'a>(
        &'a self,
        path: &'a PathBuf,
    ) -> ResourceIoFuture<'a, Result<Vec<u8>, FileLoadError>> {
        Box::pin(fs::load_file(path))
    }

    fn is_file<'a>(&'a self, path: &'a PathBuf) -> ResourceIoFuture<'a, bool> {
        Box::pin(fs::is_file(path))
    }

    fn is_dir<'a>(&'a self, path: &'a PathBuf) -> ResourceIoFuture<'a, bool> {
        Box::pin(fs::is_dir(path))
    }
}
