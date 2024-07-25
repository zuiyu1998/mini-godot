use crate::io::ResourceIo;
use mini_core::uuid::Uuid;
use std::{any::Any, fmt::Debug, future::Future, path::PathBuf, pin::Pin, sync::Arc};

pub trait ResourceData: 'static {}

pub trait ResourceError: 'static + Debug {}

pub struct LoaderPayload(pub(crate) Box<dyn ResourceData>);

pub struct LoadError(Option<Arc<dyn ResourceError>>);

pub type BoxedLoaderFuture = Pin<Box<dyn Future<Output = Result<LoaderPayload, LoadError>> + Send>>;

pub trait ResourceLoaderType: 'static + Send {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> ResourceLoaderType for T
where
    T: ResourceLoader,
{
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait ResourceLoader: ResourceLoaderType {
    //支持的文件
    fn extensions(&self) -> &[&str];

    fn load(&self, path: PathBuf, io: Arc<dyn ResourceIo>) -> BoxedLoaderFuture;

    fn data_type_uuid(&self) -> Uuid;
}

#[derive(Default)]
pub struct ResourceLoadersContainer {
    loaders: Vec<Box<dyn ResourceLoader>>,
}

impl ResourceLoadersContainer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set<T>(&mut self, loader: T) -> Option<T>
    where
        T: ResourceLoader,
    {
        if let Some(existing_loader) = self
            .loaders
            .iter_mut()
            .find_map(|l| (**l).as_any_mut().downcast_mut::<T>())
        {
            Some(std::mem::replace(existing_loader, loader))
        } else {
            self.loaders.push(Box::new(loader));
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn ResourceLoader> {
        self.loaders.iter().map(|boxed| &**boxed)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn ResourceLoader> {
        self.loaders.iter_mut().map(|boxed| &mut **boxed)
    }
}
