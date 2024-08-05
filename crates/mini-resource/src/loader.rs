use std::{error::Error, path::Path, sync::Arc};

use mini_core::{
    downcast::Downcast,
    future::{BoxedFuture, ConditionalSendFuture},
    prelude::TypeUuidProvider,
    uuid::Uuid,
};

use crate::{
    io::{AssetPath, Reader},
    manager::ResourceManager,
    meta::{ResourceMeta, ResourceMetaDyn, ResourceSettings},
    prelude::Resource,
    resource::{ErasedResourceData, ResourceData},
};

#[derive(Default, Clone)]
pub struct ResourceLoaders {
    loaders: Vec<Arc<dyn ErasedResourceLoader>>,
}

impl ResourceLoaders {
    pub fn push<T: ResourceLoader>(&mut self, loader: T) {
        self.loaders.push(Arc::new(loader));
    }

    pub fn find(&self, extension: &str) -> Option<Arc<dyn ErasedResourceLoader>> {
        self.loaders
            .iter()
            .find(|loader| loader.supports_extension(extension))
            .and_then(|loader| Some(loader.clone()))
    }

    pub fn find_loader(&self, path: &Path) -> Option<Arc<dyn ErasedResourceLoader>> {
        path.extension()
            .and_then(|extension| self.find(&extension.to_string_lossy()))
    }
}

pub struct LoadedResource<T: ResourceData> {
    pub(crate) value: T,
}

pub struct ErasedLoadedResource {
    pub(crate) value: Box<dyn ErasedResourceData>,
}

impl<R: ResourceData> From<LoadedResource<R>> for ErasedLoadedResource {
    fn from(resource: LoadedResource<R>) -> Self {
        ErasedLoadedResource {
            value: Box::new(resource.value),
        }
    }
}

pub struct LoadContext<'a> {
    pub(crate) resource_mananger: &'a ResourceManager,
    asset_path: AssetPath<'static>,
}

impl<'a> LoadContext<'a> {
    pub fn path(&self) -> &Path {
        self.asset_path.path()
    }

    /// Creates a new [`LoadContext`] instance.
    pub(crate) fn new(
        resource_mananger: &'a ResourceManager,
        asset_path: AssetPath<'static>,
    ) -> Self {
        Self {
            resource_mananger,
            asset_path,
        }
    }

    pub async fn load_sub_resource<'b, R: ResourceData>(
        &self,
        path: impl Into<AssetPath<'b>>,
    ) -> Resource<R> {
        self.resource_mananger.load_async::<R>(path).await
    }

    pub fn finish<R: ResourceData>(self, value: R) -> LoadedResource<R> {
        LoadedResource { value }
    }
}

pub trait ResourceLoader: 'static + Send + Sync {
    type ResourceData: ResourceData;
    type Settings: ResourceSettings + Default + Clone;
    type Error: Error + Send + Sync + 'static;

    //支持的文件
    fn extensions(&self) -> &[&str];

    fn load<'a>(
        &'a self,
        reader: &'a mut dyn Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::ResourceData, Self::Error>>;

    //用于向上转换
    fn data_type_uuid() -> Uuid {
        <Self::ResourceData as TypeUuidProvider>::type_uuid()
    }
}

pub trait ErasedResourceLoader: 'static + Sync + Downcast + Send {
    fn load<'a>(
        &'a self,
        reader: &'a mut dyn Reader,
        meta: Box<dyn ResourceMetaDyn>,
        load_context: LoadContext<'a>,
    ) -> BoxedFuture<
        'a,
        Result<ErasedLoadedResource, Box<dyn std::error::Error + Send + Sync + 'static>>,
    >;

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
    fn load<'a>(
        &'a self,
        reader: &'a mut dyn Reader,
        meta: Box<dyn ResourceMetaDyn>,
        mut load_context: LoadContext<'a>,
    ) -> BoxedFuture<
        'a,
        Result<ErasedLoadedResource, Box<dyn std::error::Error + Send + Sync + 'static>>,
    > {
        Box::pin(async move {
            let settings = meta
                .loader_settings()
                .expect("Loader settings should exist")
                .downcast::<T::Settings>()
                .expect("AssetLoader settings should match the loader type");
            let asset = <T as ResourceLoader>::load(self, reader, settings, &mut load_context)
                .await
                .map_err(|e| Box::new(e))?;
            Ok(load_context.finish(asset).into())
        })
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
