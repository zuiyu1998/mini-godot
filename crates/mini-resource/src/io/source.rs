use std::{collections::HashMap, fmt::Display, hash::Hash, time::Duration};

use mini_core::{cow_arc::CowArc, thiserror::Error};

use super::{ErasedAssetReader, ErasedAssetWriter};

/// A reference to an "asset source", which maps to an [`AssetReader`] and/or [`AssetWriter`].
///
/// * [`ResourceSourceId::Default`] corresponds to "default asset paths" that don't specify a source: `/path/to/asset.png`
/// * [`ResourceSourceId::Name`] corresponds to asset paths that _do_ specify a source: `remote://path/to/asset.png`, where `remote` is the name.
/// 资源路径的映射
#[derive(Default, Clone, Debug, Eq)]
pub enum ResourceSourceId<'a> {
    /// The default asset source.
    #[default]
    Default,
    /// A non-default named asset source.
    Name(CowArc<'a, str>),
}

impl<'a> Display for ResourceSourceId<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_str() {
            None => write!(f, "ResourceSourceId::Default"),
            Some(v) => write!(f, "ResourceSourceId::Name({v})"),
        }
    }
}

impl<'a> ResourceSourceId<'a> {
    /// Creates a new [`ResourceSourceId`]
    pub fn new(source: Option<impl Into<CowArc<'a, str>>>) -> ResourceSourceId<'a> {
        match source {
            Some(source) => ResourceSourceId::Name(source.into()),
            None => ResourceSourceId::Default,
        }
    }

    /// Returns [`None`] if this is [`ResourceSourceId::Default`] and [`Some`] containing the
    /// name if this is [`ResourceSourceId::Name`].
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ResourceSourceId::Default => None,
            ResourceSourceId::Name(v) => Some(v),
        }
    }

    /// If this is not already an owned / static id, create one. Otherwise, it will return itself (with a static lifetime).
    pub fn into_owned(self) -> ResourceSourceId<'static> {
        match self {
            ResourceSourceId::Default => ResourceSourceId::Default,
            ResourceSourceId::Name(v) => ResourceSourceId::Name(v.into_owned()),
        }
    }

    /// Clones into an owned [`ResourceSourceId<'static>`].
    /// This is equivalent to `.clone().into_owned()`.
    #[inline]
    pub fn clone_owned(&self) -> ResourceSourceId<'static> {
        self.clone().into_owned()
    }
}

impl From<&'static str> for ResourceSourceId<'static> {
    fn from(value: &'static str) -> Self {
        ResourceSourceId::Name(value.into())
    }
}

impl<'a, 'b> From<&'a ResourceSourceId<'b>> for ResourceSourceId<'b> {
    fn from(value: &'a ResourceSourceId<'b>) -> Self {
        value.clone()
    }
}

impl From<Option<&'static str>> for ResourceSourceId<'static> {
    fn from(value: Option<&'static str>) -> Self {
        match value {
            Some(value) => ResourceSourceId::Name(value.into()),
            None => ResourceSourceId::Default,
        }
    }
}

impl From<String> for ResourceSourceId<'static> {
    fn from(value: String) -> Self {
        ResourceSourceId::Name(value.into())
    }
}

impl<'a> Hash for ResourceSourceId<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<'a> PartialEq for ResourceSourceId<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(&other.as_str())
    }
}

/// Metadata about an "asset source", such as how to construct the [`AssetReader`] and [`AssetWriter`] for the source,
/// and whether or not the source is processed.
#[derive(Default)]
pub struct ResourceSourceBuilder {
    pub reader: Option<Box<dyn FnMut() -> Box<dyn ErasedAssetReader> + Send + Sync>>,
    pub writer: Option<Box<dyn FnMut(bool) -> Option<Box<dyn ErasedAssetWriter>> + Send + Sync>>,

    pub watch_warning: Option<&'static str>,
}

impl ResourceSourceBuilder {
    /// Builds a new [`ResourceSource`] with the given `id`. If `watch` is true, the unprocessed source will watch for changes.
    /// If `watch_processed` is true, the processed source will watch for changes.
    pub fn build(&mut self, id: ResourceSourceId<'static>) -> Option<ResourceSource> {
        let reader = self.reader.as_mut()?();
        let writer = self.writer.as_mut().and_then(|w| w(false));
        let mut source = ResourceSource {
            id: id.clone(),
            reader,
            writer,
        };

        Some(source)
    }

    /// Will use the given `reader` function to construct unprocessed [`AssetReader`] instances.
    pub fn with_reader(
        mut self,
        reader: impl FnMut() -> Box<dyn ErasedAssetReader> + Send + Sync + 'static,
    ) -> Self {
        self.reader = Some(Box::new(reader));
        self
    }

    /// Will use the given `writer` function to construct unprocessed [`AssetWriter`] instances.
    pub fn with_writer(
        mut self,
        writer: impl FnMut(bool) -> Option<Box<dyn ErasedAssetWriter>> + Send + Sync + 'static,
    ) -> Self {
        self.writer = Some(Box::new(writer));
        self
    }

    /// Enables a warning for the unprocessed source watcher, which will print when watching is enabled and the unprocessed source doesn't have a watcher.
    pub fn with_watch_warning(mut self, warning: &'static str) -> Self {
        self.watch_warning = Some(warning);
        self
    }

    /// Returns a builder containing the "platform default source" for the given `path` and `processed_path`.
    /// For most platforms, this will use [`FileAssetReader`](crate::io::file::FileAssetReader) / [`FileAssetWriter`](crate::io::file::FileAssetWriter),
    /// but some platforms (such as Android) have their own default readers / writers / watchers.
    pub fn platform_default(path: &str) -> Self {
        let default = Self::default()
            .with_reader(ResourceSource::get_default_reader(path.to_string()))
            .with_writer(ResourceSource::get_default_writer(path.to_string()))
            .with_watch_warning(ResourceSource::get_default_watch_warning());

        default
    }
}

/// A [`Resource`] that hold (repeatable) functions capable of producing new [`AssetReader`] and [`AssetWriter`] instances
/// for a given asset source.
#[derive(Default)]
pub struct ResourceSourceBuilders {
    sources: HashMap<CowArc<'static, str>, ResourceSourceBuilder>,
    default: Option<ResourceSourceBuilder>,
}

impl ResourceSourceBuilders {
    /// Inserts a new builder with the given `id`
    pub fn insert(
        &mut self,
        id: impl Into<ResourceSourceId<'static>>,
        source: ResourceSourceBuilder,
    ) {
        match id.into() {
            ResourceSourceId::Default => {
                self.default = Some(source);
            }
            ResourceSourceId::Name(name) => {
                self.sources.insert(name, source);
            }
        }
    }

    /// Gets a mutable builder with the given `id`, if it exists.
    pub fn get_mut<'a, 'b>(
        &'a mut self,
        id: impl Into<ResourceSourceId<'b>>,
    ) -> Option<&'a mut ResourceSourceBuilder> {
        match id.into() {
            ResourceSourceId::Default => self.default.as_mut(),
            ResourceSourceId::Name(name) => self.sources.get_mut(&name.into_owned()),
        }
    }

    /// Builds a new [`ResourceSources`] collection. If `watch` is true, the unprocessed sources will watch for changes.
    /// If `watch_processed` is true, the processed sources will watch for changes.
    pub fn build_sources(&mut self) -> ResourceSources {
        let mut sources = HashMap::new();
        for (id, source) in &mut self.sources {
            if let Some(data) = source.build(ResourceSourceId::Name(id.clone_owned())) {
                sources.insert(id.clone_owned(), data);
            }
        }

        ResourceSources {
            sources,
            default: self
                .default
                .as_mut()
                .and_then(|p| p.build(ResourceSourceId::Default))
                .expect(MISSING_DEFAULT_SOURCE),
        }
    }

    /// Initializes the default [`ResourceSourceBuilder`] if it has not already been set.
    pub fn init_default_source(&mut self, path: &str) {
        self.default
            .get_or_insert_with(|| ResourceSourceBuilder::platform_default(path));
    }
}

/// A collection of unprocessed and processed [`AssetReader`], [`AssetWriter`], and [`AssetWatcher`] instances
/// for a specific asset source, identified by an [`ResourceSourceId`].
pub struct ResourceSource {
    id: ResourceSourceId<'static>,
    reader: Box<dyn ErasedAssetReader>,
    writer: Option<Box<dyn ErasedAssetWriter>>,
}

impl ResourceSource {
    /// Starts building a new [`ResourceSource`].
    pub fn build() -> ResourceSourceBuilder {
        ResourceSourceBuilder::default()
    }

    /// Returns this source's id.
    #[inline]
    pub fn id(&self) -> ResourceSourceId<'static> {
        self.id.clone()
    }

    /// Return's this source's unprocessed [`AssetReader`].
    #[inline]
    pub fn reader(&self) -> &dyn ErasedAssetReader {
        &*self.reader
    }

    /// Return's this source's unprocessed [`AssetWriter`], if it exists.
    #[inline]
    pub fn writer(&self) -> Result<&dyn ErasedAssetWriter, MissingAssetWriterError> {
        self.writer
            .as_deref()
            .ok_or_else(|| MissingAssetWriterError(self.id.clone_owned()))
    }

    /// Returns a builder function for this platform's default [`AssetReader`]. `path` is the relative path to
    /// the asset root.
    pub fn get_default_reader(
        _path: String,
    ) -> impl FnMut() -> Box<dyn ErasedAssetReader> + Send + Sync {
        move || {
            return Box::new(super::file::FileAssetReader::new(&_path));
        }
    }

    /// Returns a builder function for this platform's default [`AssetWriter`]. `path` is the relative path to
    /// the asset root. This will return [`None`] if this platform does not support writing assets by default.
    pub fn get_default_writer(
        _path: String,
    ) -> impl FnMut(bool) -> Option<Box<dyn ErasedAssetWriter>> + Send + Sync {
        move |_create_root: bool| {
            return Some(Box::new(super::file::FileAssetWriter::new(
                &_path,
                _create_root,
            )));
        }
    }

    /// Returns the default non-existent [`AssetWatcher`] warning for the current platform.
    pub fn get_default_watch_warning() -> &'static str {
        #[cfg(target_arch = "wasm32")]
        return "Web does not currently support watching assets.";
        #[cfg(target_os = "android")]
        return "Android does not currently support watching assets.";
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_os = "android"),
            not(feature = "file_watcher")
        ))]
        return "Consider enabling the `file_watcher` feature.";
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_os = "android"),
            feature = "file_watcher"
        ))]
        return "Consider adding an \"assets\" directory.";
    }
}

/// A collection of [`ResourceSource`]s.
pub struct ResourceSources {
    sources: HashMap<CowArc<'static, str>, ResourceSource>,
    default: ResourceSource,
}

impl ResourceSources {
    /// Gets the [`ResourceSource`] with the given `id`, if it exists.
    pub fn get<'a, 'b>(
        &'a self,
        id: impl Into<ResourceSourceId<'b>>,
    ) -> Result<&'a ResourceSource, MissingAssetSourceError> {
        match id.into().into_owned() {
            ResourceSourceId::Default => Ok(&self.default),
            ResourceSourceId::Name(name) => self
                .sources
                .get(&name)
                .ok_or_else(|| MissingAssetSourceError(ResourceSourceId::Name(name))),
        }
    }

    /// Iterates all asset sources in the collection (including the default source).
    pub fn iter(&self) -> impl Iterator<Item = &ResourceSource> {
        self.sources.values().chain(Some(&self.default))
    }

    /// Mutably iterates all asset sources in the collection (including the default source).
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ResourceSource> {
        self.sources.values_mut().chain(Some(&mut self.default))
    }

    /// Iterates over the [`ResourceSourceId`] of every [`ResourceSource`] in the collection (including the default source).
    pub fn ids(&self) -> impl Iterator<Item = ResourceSourceId<'static>> + '_ {
        self.sources
            .keys()
            .map(|k| ResourceSourceId::Name(k.clone_owned()))
            .chain(Some(ResourceSourceId::Default))
    }
}

/// An error returned when an [`ResourceSource`] does not exist for a given id.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("Asset Source '{0}' does not exist")]
pub struct MissingAssetSourceError(ResourceSourceId<'static>);

/// An error returned when an [`AssetWriter`] does not exist for a given id.
#[derive(Error, Debug, Clone)]
#[error("Asset Source '{0}' does not have an AssetWriter.")]
pub struct MissingAssetWriterError(ResourceSourceId<'static>);

/// An error returned when a processed [`AssetReader`] does not exist for a given id.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("Asset Source '{0}' does not have a processed AssetReader.")]
pub struct MissingProcessedAssetReaderError(ResourceSourceId<'static>);

/// An error returned when a processed [`AssetWriter`] does not exist for a given id.
#[derive(Error, Debug, Clone)]
#[error("Asset Source '{0}' does not have a processed AssetWriter.")]
pub struct MissingProcessedAssetWriterError(ResourceSourceId<'static>);

const MISSING_DEFAULT_SOURCE: &str =
    "A default ResourceSource is required. Add one to `ResourceSourceBuilders`";
