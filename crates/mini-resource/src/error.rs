use crate::io::{AssetReaderError, MissingAssetSourceError};
use mini_core::thiserror::Error;
use std::{fmt::Debug, sync::Arc};

#[derive(Debug, Error)]
pub enum ResourceError {
    #[error(transparent)]
    MissingAssetSourceError(#[from] MissingAssetSourceError),
    #[error(transparent)]
    AssetReaderError(#[from] AssetReaderError),
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
