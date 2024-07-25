use std::{future::Future, io::Error as IoError, path::PathBuf, pin::Pin};
use thiserror::Error;

mod fs;

pub type ResourceIoFuture<'a, V> = Pin<Box<dyn Future<Output = V> + Send + 'a>>;

#[derive(Debug, Error)]
pub enum FileLoadError {
    #[error("{0}")]
    IoError(#[from] IoError),
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
