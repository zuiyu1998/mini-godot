use mini_core::{parking_lot::Mutex, prelude::FxHashMap};
use mini_task::TaskPool;
use std::sync::Arc;

use crate::{
    error::{LoadError, ResourceError},
    io::{Reader, ResourcePath, ResourceSourceBuilders, ResourceSources},
    loader::{ErasedResourceLoader, LoadContext, ResourceLoader, ResourceLoaders},
    meta::{ResourceMetaDyn, ResourceMetas},
    resource::{Resource, ResourceData, ResourceKind, ResourceState, UntypedResource},
};

#[derive(Clone)]
pub struct ResourceManager {
    state: Arc<ResourceManagerState>,
}

impl ResourceManager {
    pub fn new(task_pool: Arc<TaskPool>) -> Self {
        Self {
            state: Arc::new(ResourceManagerState::new(task_pool)),
        }
    }

    pub fn load<'a, T>(&self, path: impl Into<ResourcePath<'a>>) -> Resource<T>
    where
        T: ResourceData,
    {
        let untyped = self.load_untyped(path);
        Resource::new(untyped)
    }

    pub fn load_built_in(
        &self,
        path: &ResourcePath<'_>,
        kind: ResourceKind,
    ) -> Result<UntypedResource, Arc<dyn ErasedResourceLoader>> {
        {
            let built_in_resources = self.state.built_in_resources.lock();
            if let Some(built_in_resource) = built_in_resources.get(&path) {
                return Ok(built_in_resource.clone());
            }
        }

        let loaders = self.state.loaders.lock();

        if let Some(loader) = loaders.find_loader(&path.path()) {
            return Err(loader);
        } else {
            let err = LoadError::new(format!("There's no resource loader for {kind} resource!",));
            return Ok(UntypedResource::new_load_error(
                kind,
                err,
                Default::default(),
            ));
        }
    }

    pub async fn load_async<'a, T: ResourceData>(
        &self,
        path: impl Into<ResourcePath<'a>>,
    ) -> Resource<T> {
        let path: ResourcePath<'a> = path.into();
        let path: ResourcePath<'static> = path.into_owned();

        let kind = ResourceKind::External(path.clone());
        let loader = match self.load_built_in(&path, kind.clone()) {
            Ok(resource) => {
                return Resource::new(resource);
            }

            Err(loader) => loader,
        };

        let resource = UntypedResource::new_pending(kind, loader.data_type_uuid());

        self.load_internal(path, resource.clone(), loader).await;

        Resource::new(resource)
    }

    pub fn load_untyped<'a>(&self, path: impl Into<ResourcePath<'a>>) -> UntypedResource {
        let path: ResourcePath<'a> = path.into();
        let path: ResourcePath<'static> = path.into_owned();

        let kind = ResourceKind::External(path.clone());

        let loader = match self.load_built_in(&path, kind.clone()) {
            Ok(resource) => {
                return resource;
            }

            Err(loader) => loader,
        };

        let resource = UntypedResource::new_pending(kind, loader.data_type_uuid());

        self.spawn_loading_task(path, resource.clone(), loader, false);

        resource
    }

    pub async fn get_meta_and_reader<'a>(
        &'a self,
        path: &'a ResourcePath<'_>,
        loader: &'a Arc<dyn ErasedResourceLoader>,
    ) -> Result<(Box<dyn ResourceMetaDyn>, Box<dyn Reader + 'a>), ResourceError> {
        let value = self.state.get_meta_and_reader(&path, &loader).await?;

        Ok(value)
    }

    async fn load_internal(
        &self,
        path: ResourcePath<'static>,
        resource: UntypedResource,
        loader: Arc<dyn ErasedResourceLoader>,
    ) {
        let (meta, mut reader) = match self.get_meta_and_reader(&path, &loader).await {
            Ok((meta, reader)) => (meta, reader),
            Err(e) => {
                return resource.commit_error(e);
            }
        };

        let load_context = LoadContext::new(self, path.clone());
        match loader.load(&mut (*reader), meta, load_context).await {
            Err(e) => {
                return resource.commit_error(e);
            }

            Ok(loaded_resource) => {
                let mut mutex_guard = resource.0.lock();
                assert_eq!(mutex_guard.type_uuid, loaded_resource.value.type_uuid());
                assert!(mutex_guard.kind.is_external());
                mutex_guard
                    .state
                    .commit(ResourceState::Ok(loaded_resource.value));
            }
        }
    }

    fn spawn_loading_task(
        &self,
        path: ResourcePath<'static>,
        resource: UntypedResource,
        loader: Arc<dyn ErasedResourceLoader>,
        _reload: bool,
    ) {
        let resource_manger = (*self).clone();

        self.task_pool().spawn_task(async move {
            resource_manger.load_internal(path, resource, loader).await;
        });
    }

    pub fn add_loader<L: ResourceLoader>(&self, loader: L) {
        self.state.add_loader(loader);
    }
    pub fn task_pool(&self) -> Arc<TaskPool> {
        self.state.task_pool()
    }
}

pub struct ResourceManagerState {
    pub loaders: Mutex<ResourceLoaders>,
    pub metas: Mutex<ResourceMetas>,
    //内置资源
    pub built_in_resources: Mutex<FxHashMap<ResourcePath<'static>, UntypedResource>>,

    pub asset_sources: ResourceSources,

    task_pool: Arc<TaskPool>,
}

impl ResourceManagerState {
    pub fn add_loader<L: ResourceLoader>(&self, loader: L) {
        self.loaders.lock().push(loader);
        self.metas.lock().insert::<L>();
    }

    pub(crate) fn new(task_pool: Arc<TaskPool>) -> Self {
        let mut asset_source_builders = ResourceSourceBuilders::default();
        asset_source_builders.init_default_source("assets");

        Self {
            task_pool,
            loaders: Default::default(),
            metas: Default::default(),
            built_in_resources: Default::default(),
            asset_sources: asset_source_builders.build_sources(),
        }
    }

    pub fn task_pool(&self) -> Arc<TaskPool> {
        self.task_pool.clone()
    }

    pub async fn get_meta_and_reader<'a>(
        &'a self,
        path: &'a ResourcePath<'_>,
        loader: &'a Arc<dyn ErasedResourceLoader>,
    ) -> Result<(Box<dyn ResourceMetaDyn>, Box<dyn Reader + 'a>), ResourceError> {
        let source = self.asset_sources.get(path.source())?;

        let asset_reader = source.reader();
        let reader = asset_reader.read(path.path()).await?;

        let metas = self.metas.lock();

        let meta = metas
            .get(&loader.data_type_uuid())
            .and_then(|meta| loader.default_meta_from_dyn(meta.as_ref()))
            .unwrap_or_else(|| loader.default_meta());

        Ok((meta, reader))
    }
}
