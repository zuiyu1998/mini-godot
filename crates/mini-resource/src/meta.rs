use mini_core::{downcast::Downcast, utils::FxHashMap, uuid::Uuid};

use crate::loader::ResourceLoader;

pub const META_FORMAT_VERSION: &str = "1.0";

pub trait ResourceMetaDyn: Downcast + Send + Sync {
    fn loader_settings(&self) -> Option<&dyn ResourceSettings>;
}

pub trait ResourceSettings: 'static + Send + Downcast + Sync {}

impl ResourceSettings for () {}

impl dyn ResourceSettings {
    pub fn is<T: ResourceSettings>(&self) -> bool {
        self.as_any().is::<T>()
    }
}

#[derive(Default)]
pub struct ResourceMetas {
    metas: FxHashMap<Uuid, Box<dyn ResourceMetaDyn>>,
}

impl ResourceMetas {
    pub fn insert<R: ResourceLoader>(&mut self) {
        self.metas
            .insert(R::data_type_uuid(), Box::new(ResourceMeta::<R>::new()));
    }

    pub fn get(&self, key: &Uuid) -> Option<&Box<dyn ResourceMetaDyn>> {
        self.metas.get(key)
    }
}

#[derive(Debug, Clone)]
pub struct ResourceMeta<R: ResourceLoader> {
    meta_format_version: String,
    settings: R::Settings,
}

impl<R: ResourceLoader> ResourceMetaDyn for ResourceMeta<R> {
    fn loader_settings(&self) -> Option<&dyn ResourceSettings> {
        return Some(&self.settings);
    }
}

impl<R: ResourceLoader> ResourceMeta<R> {
    pub fn new() -> Self {
        ResourceMeta {
            meta_format_version: META_FORMAT_VERSION.to_string(),
            settings: R::Settings::default(),
        }
    }

    pub fn new_settings(settings: &dyn ResourceSettings) -> Option<Self> {
        if settings.is::<R::Settings>() {
            let settings =
                <R::Settings>::clone(settings.as_any().downcast_ref::<R::Settings>().unwrap());

            Some(ResourceMeta {
                meta_format_version: META_FORMAT_VERSION.to_string(),
                settings,
            })
        } else {
            None
        }
    }
}
