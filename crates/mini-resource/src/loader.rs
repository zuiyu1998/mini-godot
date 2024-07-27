use std::{fmt::Debug, future::Future, path::PathBuf, pin::Pin, sync::Arc};

use mini_core::{prelude::TypeUuidProvider, uuid::Uuid};

use crate::io::ResourceIo;

pub struct ResourceLoaders {
    loaders: Vec<Box<dyn ResourceLoader>>,
}

impl ResourceLoaders {
    pub fn iter(&self) -> impl Iterator<Item = &dyn ResourceLoader> {
        self.loaders.iter().map(|boxed| &**boxed)
    }

    /// Returns an iterator yielding mutable references to "untyped" resource loaders.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn ResourceLoader> {
        self.loaders.iter_mut().map(|boxed| &mut **boxed)
    }
}

pub type BoxedLoaderFuture = Pin<Box<dyn Future<Output = Result<LoaderPayload, LoadError>> + Send>>;

pub struct LoaderPayload(pub(crate) Box<dyn ResourceData>);

impl LoaderPayload {
    pub fn new<T: ResourceData>(data: T) -> Self {
        Self(Box::new(data))
    }
}

pub trait ResourceLoader: 'static {
    //支持的文件
    fn extensions(&self) -> &[&str];

    fn supports_extension(&self, ext: &str) -> bool {
        self.extensions()
            .iter()
            .any(|e| mini_core::utils::cmp_strings_case_insensitive(e, ext))
    }

    fn load(&self, path: PathBuf, io: Arc<dyn ResourceIo>) -> BoxedLoaderFuture;

    //用于向上转换
    fn data_type_uuid(&self) -> Uuid;
}

#[derive(Debug, Clone)]
pub struct LoadError(pub Option<Arc<dyn ResourceLoadError>>);

impl LoadError {
    /// Creates new loading error from a value of the given type.
    pub fn new<T: ResourceLoadError>(value: T) -> Self {
        Self(Some(Arc::new(value)))
    }
}

pub trait ResourceLoadError: 'static + Debug + Send + Sync {}

impl<T> ResourceLoadError for T where T: 'static + Debug + Send + Sync {}

pub trait TypedResourceData: TypeUuidProvider + ResourceData {}

pub trait ResourceData: 'static + Debug + Send {
    //用于向上转换
    fn type_uuid(&self) -> Uuid;
}
