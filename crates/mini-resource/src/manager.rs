use mini_core::{
    parking_lot::{Mutex, MutexGuard},
    prelude::{FxHashMap, TypeUuidProvider},
};
use mini_task::TaskPool;
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    io::{FsResourceIo, ResourceIo},
    loader::{ErasedResourceData, LoadError, ResourceData, ResourceLoader, ResourceLoaders},
    resource::{Resource, ResourceKind, ResourceState, UntypedResource},
};

pub struct ResourceManager {
    state: Arc<Mutex<ResourceManagerState>>,
}

impl ResourceManager {
    pub fn new(task_pool: Arc<TaskPool>) -> Self {
        Self {
            state: Arc::new(Mutex::new(ResourceManagerState::new(task_pool))),
        }
    }

    pub fn request<T>(&self, path: impl AsRef<Path>) -> Resource<T>
    where
        T: ResourceData,
    {
        let untyped = self.state().load(path);
        let actual_type_uuid = untyped.type_uuid();
        assert_eq!(actual_type_uuid, <T as TypeUuidProvider>::type_uuid());
        Resource {
            untyped,
            type_marker: PhantomData::<T>,
        }
    }

    pub fn state(&self) -> MutexGuard<'_, ResourceManagerState> {
        self.state.lock()
    }

    pub fn resource_io(&self) -> Arc<dyn ResourceIo> {
        let state = self.state();
        state.resource_io.clone()
    }

    pub fn task_pool(&self) -> Arc<TaskPool> {
        let state = self.state();
        state.task_pool()
    }
}

pub struct ResourceManagerState {
    pub loaders: ResourceLoaders,

    //内置资源
    pub built_in_resources: FxHashMap<PathBuf, UntypedResource>,

    pub resource_io: Arc<dyn ResourceIo>,

    task_pool: Arc<TaskPool>,
}

impl ResourceManagerState {
    pub(crate) fn new(task_pool: Arc<TaskPool>) -> Self {
        Self {
            task_pool,
            loaders: Default::default(),
            built_in_resources: Default::default(),
            // Use the file system resource io by default
            resource_io: Arc::new(FsResourceIo),
        }
    }

    pub fn task_pool(&self) -> Arc<TaskPool> {
        self.task_pool.clone()
    }

    fn find_loader(&self, path: &Path) -> Option<&dyn ResourceLoader> {
        path.extension().and_then(|extension| {
            self.loaders
                .iter()
                .find(|loader| loader.supports_extension(&extension.to_string_lossy()))
        })
    }

    fn spawn_loading_task(
        &self,
        path: PathBuf,
        resource: UntypedResource,
        loader: &dyn ResourceLoader,
        _reload: bool,
    ) {
        let loader_future = loader.load(path.clone(), self.resource_io.clone());
        self.task_pool.spawn_task(async move {
            match loader_future.await {
                Ok(data) => {
                    let data = data.0;

                    // Separate scope to keep mutex locking time at minimum.
                    {
                        let mut mutex_guard = resource.0.lock();
                        assert_eq!(mutex_guard.type_uuid, data.type_uuid());
                        assert!(mutex_guard.kind.is_external());
                        mutex_guard.state.commit(ResourceState::Ok(data));
                    }
                }
                Err(error) => {
                    resource.commit_error(error);
                }
            }
        });
    }

    pub fn load<P>(&self, path: P) -> UntypedResource
    where
        P: AsRef<Path>,
    {
        if let Some(built_in_resource) = self.built_in_resources.get(path.as_ref()) {
            return built_in_resource.clone();
        }

        let path = path.as_ref().to_owned();
        let kind = ResourceKind::External(path.clone());

        if let Some(loader) = self.find_loader(&path) {
            let resource = UntypedResource::new_pending(kind, loader.data_type_uuid());

            self.spawn_loading_task(path, resource.clone(), loader, false);

            resource
        } else {
            let err = LoadError::new(format!("There's no resource loader for {kind} resource!",));
            UntypedResource::new_load_error(kind, err, Default::default())
        }
    }
}
