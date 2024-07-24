use std::{fmt::Debug, path::PathBuf, sync::Arc};

pub enum IoError {}

pub trait ResourceIo: Send + Sync + 'static {
    fn exists(&self, path: &PathBuf) -> bool;

    fn load_file(&self, path: &PathBuf) -> Result<Vec<u8>, IoError>;

    fn is_file(&self, path: &PathBuf) -> bool;

    fn is_dir(&self, path: &PathBuf) -> bool;
}

pub trait ResourceData: 'static {}

pub trait ResourceError: 'static + Debug {}

pub struct LoaderPayload(pub(crate) Box<dyn ResourceData>);

pub struct LoaderError(Option<Box<dyn ResourceError>>);

pub trait ResourceLoader {
    //支持的文件
    fn extensions(&self) -> &[&str];

    fn load(&self, path: PathBuf, io: Arc<dyn ResourceIo>) -> Result<LoaderPayload, LoaderError>;
}
