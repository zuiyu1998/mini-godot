use std::{fmt::Debug, future::Future, path::PathBuf, pin::Pin, sync::Arc};

use mini_core::{downcast::Downcast, prelude::TypeUuidProvider, uuid::Uuid};

use crate::{
    io::ResourceIo,
    meta::{ResourceMeta, ResourceMetaDyn, ResourceSettings},
    resource::{ErasedResourceData, ResourceData},
};

#[derive(Default)]
pub struct ResourceLoaders {
    loaders: Vec<Box<dyn ErasedResourceLoader>>,
}

impl ResourceLoaders {
    pub fn push<T: ResourceLoader>(&mut self, loader: T) {
        self.loaders.push(Box::new(loader));
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn ErasedResourceLoader> {
        self.loaders.iter().map(|boxed| &**boxed)
    }

    /// Returns an iterator yielding mutable references to "untyped" resource loaders.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn ErasedResourceLoader> {
        self.loaders.iter_mut().map(|boxed| &mut **boxed)
    }
}

pub type BoxedLoaderFuture = Pin<Box<dyn Future<Output = Result<LoaderPayload, LoadError>> + Send>>;

pub struct LoaderPayload(pub(crate) Box<dyn ErasedResourceData>);

impl LoaderPayload {
    pub fn new<T: ResourceData>(data: T) -> Self {
        Self(Box::new(data))
    }
}

pub trait ResourceLoader: 'static + Send + Sync {
    type ResourceData: ResourceData;
    type Settings: ResourceSettings + Default + Clone;
    type Error: ResourceLoadError;

    //支持的文件
    fn extensions(&self) -> &[&str];

    fn load(
        &self,
        path: PathBuf,
        io: Arc<dyn ResourceIo>,
        settings: &Self::Settings,
    ) -> impl Future<Output = Result<Self::ResourceData, Self::Error>>;

    //用于向上转换
    fn data_type_uuid() -> Uuid {
        <Self::ResourceData as TypeUuidProvider>::type_uuid()
    }
}

pub trait ErasedResourceLoader: 'static + Sync + Downcast {
    fn load(
        &self,
        path: PathBuf,
        io: Arc<dyn ResourceIo>,
        meta: Box<dyn ResourceMetaDyn>,
    ) -> BoxedLoaderFuture;

    fn default_meta(&self) -> Box<dyn ResourceMetaDyn>;

    fn extensions(&self) -> &[&str];

    fn supports_extension(&self, ext: &str) -> bool {
        self.extensions()
            .iter()
            .any(|e| mini_core::utils::cmp_strings_case_insensitive(e, ext))
    }

    fn data_type_uuid(&self) -> Uuid;

    fn default_meta_from_dyn(&self, meta: &dyn ResourceMetaDyn)
        -> Option<Box<dyn ResourceMetaDyn>>;
}

impl<T> ErasedResourceLoader for T
where
    T: ResourceLoader,
{
    fn load(
        &self,
        path: PathBuf,
        io: Arc<dyn ResourceIo>,
        meta: Box<dyn ResourceMetaDyn>,
    ) -> BoxedLoaderFuture {
        todo!()
    }

    fn default_meta(&self) -> Box<dyn ResourceMetaDyn> {
        Box::new(ResourceMeta::<T>::new())
    }

    fn extensions(&self) -> &[&str] {
        <T as ResourceLoader>::extensions(self)
    }

    fn data_type_uuid(&self) -> Uuid {
        <T as ResourceLoader>::data_type_uuid()
    }

    fn default_meta_from_dyn(
        &self,
        meta: &dyn ResourceMetaDyn,
    ) -> Option<Box<dyn ResourceMetaDyn>> {
        meta.loader_settings().and_then(|settings| {
            ResourceMeta::<T>::new_settings(settings)
                .and_then(|meta| Some(Box::new(meta) as Box<dyn ResourceMetaDyn>))
        })
    }
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
