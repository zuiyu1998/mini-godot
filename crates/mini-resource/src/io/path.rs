use crate::io::ResourceSourceId;
use mini_core::cow_arc::CowArc;
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::Deref,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// Represents a path to an asset in a "virtual filesystem".
///
/// Asset paths consist of three main parts:
/// * [`ResourcePath::source`]: The name of the [`ResourceSource`](crate::io::ResourceSource) to load the asset from.
///     This is optional. If one is not set the default source will be used (which is the `assets` folder by default).
/// * [`ResourcePath::path`]: The "virtual filesystem path" pointing to an asset source file.
/// * [`ResourcePath::label`]: An optional "named sub asset". When assets are loaded, they are
///     allowed to load "sub assets" of any type, which are identified by a named "label".
///
/// Asset paths are generally constructed (and visualized) as strings:
///
/// ```no_run
/// # use bevy_asset::{Asset, AssetServer, Handle};
/// # use bevy_reflect::TypePath;
/// #
/// # #[derive(Asset, TypePath, Default)]
/// # struct Mesh;
/// #
/// # #[derive(Asset, TypePath, Default)]
/// # struct Scene;
/// #
/// # let asset_server: AssetServer = panic!();
/// // This loads the `my_scene.scn` base asset from the default asset source.
/// let scene: Handle<Scene> = asset_server.load("my_scene.scn");
///
/// // This loads the `PlayerMesh` labeled asset from the `my_scene.scn` base asset in the default asset source.
/// let mesh: Handle<Mesh> = asset_server.load("my_scene.scn#PlayerMesh");
///
/// // This loads the `my_scene.scn` base asset from a custom 'remote' asset source.
/// let scene: Handle<Scene> = asset_server.load("remote://my_scene.scn");
/// ```
///
/// [`ResourcePath`] implements [`From`] for `&'static str`, `&'static Path`, and `&'a String`,
/// which allows us to optimize the static cases.
/// This means that the common case of `asset_server.load("my_scene.scn")` when it creates and
/// clones internal owned [`AssetPaths`](ResourcePath).
/// This also means that you should use [`ResourcePath::parse`] in cases where `&str` is the explicit type.
#[derive(Eq, PartialEq, Hash, Clone, Default)]
pub struct ResourcePath<'a> {
    source: ResourceSourceId<'a>,
    path: CowArc<'a, Path>,
    label: Option<CowArc<'a, str>>,
}

impl<'a> Debug for ResourcePath<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<'a> Display for ResourcePath<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let ResourceSourceId::Name(name) = self.source() {
            write!(f, "{name}://")?;
        }
        write!(f, "{}", self.path.display())?;
        if let Some(label) = &self.label {
            write!(f, "#{label}")?;
        }
        Ok(())
    }
}

/// An error that occurs when parsing a string type to create an [`ResourcePath`] fails, such as during [`ResourcePath::parse`].
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseAssetPathError {
    /// Error that occurs when the [`ResourcePath::source`] section of a path string contains the [`ResourcePath::label`] delimiter `#`. E.g. `bad#source://file.test`.
    #[error("Asset source must not contain a `#` character")]
    InvalidSourceSyntax,
    /// Error that occurs when the [`ResourcePath::label`] section of a path string contains the [`ResourcePath::source`] delimiter `://`. E.g. `source://file.test#bad://label`.
    #[error("Asset label must not contain a `://` substring")]
    InvalidLabelSyntax,
    /// Error that occurs when a path string has an [`ResourcePath::source`] delimiter `://` with no characters preceding it. E.g. `://file.test`.
    #[error("Asset source must be at least one character. Either specify the source before the '://' or remove the `://`")]
    MissingSource,
    /// Error that occurs when a path string has an [`ResourcePath::label`] delimiter `#` with no characters succeeding it. E.g. `file.test#`
    #[error("Asset label must be at least one character. Either specify the label after the '#' or remove the '#'")]
    MissingLabel,
}

impl<'a> ResourcePath<'a> {
    /// Creates a new [`ResourcePath`] from a string in the asset path format:
    /// * An asset at the root: `"scene.gltf"`
    /// * An asset nested in some folders: `"some/path/scene.gltf"`
    /// * An asset with a "label": `"some/path/scene.gltf#Mesh0"`
    /// * An asset with a custom "source": `"custom://some/path/scene.gltf#Mesh0"`
    ///
    /// Prefer [`From<'static str>`] for static strings, as this will prevent allocations
    /// and reference counting for [`ResourcePath::into_owned`].
    ///
    /// # Panics
    /// Panics if the asset path is in an invalid format. Use [`ResourcePath::try_parse`] for a fallible variant
    pub fn parse(asset_path: &'a str) -> ResourcePath<'a> {
        Self::try_parse(asset_path).unwrap()
    }

    /// Creates a new [`ResourcePath`] from a string in the asset path format:
    /// * An asset at the root: `"scene.gltf"`
    /// * An asset nested in some folders: `"some/path/scene.gltf"`
    /// * An asset with a "label": `"some/path/scene.gltf#Mesh0"`
    /// * An asset with a custom "source": `"custom://some/path/scene.gltf#Mesh0"`
    ///
    /// Prefer [`From<'static str>`] for static strings, as this will prevent allocations
    /// and reference counting for [`ResourcePath::into_owned`].
    ///
    /// This will return a [`ParseAssetPathError`] if `asset_path` is in an invalid format.
    pub fn try_parse(asset_path: &'a str) -> Result<ResourcePath<'a>, ParseAssetPathError> {
        let (source, path, label) = Self::parse_internal(asset_path)?;
        Ok(Self {
            source: match source {
                Some(source) => ResourceSourceId::Name(CowArc::Borrowed(source)),
                None => ResourceSourceId::Default,
            },
            path: CowArc::Borrowed(path),
            label: label.map(CowArc::Borrowed),
        })
    }

    // Attempts to Parse a &str into an `ResourcePath`'s `ResourcePath::source`, `ResourcePath::path`, and `ResourcePath::label` components.
    fn parse_internal(
        asset_path: &str,
    ) -> Result<(Option<&str>, &Path, Option<&str>), ParseAssetPathError> {
        let chars = asset_path.char_indices();
        let mut source_range = None;
        let mut path_range = 0..asset_path.len();
        let mut label_range = None;

        // Loop through the characters of the passed in &str to accomplish the following:
        // 1. Search for the first instance of the `://` substring. If the `://` substring is found,
        //  store the range of indices representing everything before the `://` substring as the `source_range`.
        // 2. Search for the last instance of the `#` character. If the `#` character is found,
        //  store the range of indices representing everything after the `#` character as the `label_range`
        // 3. Set the `path_range` to be everything in between the `source_range` and `label_range`,
        //  excluding the `://` substring and `#` character.
        // 4. Verify that there are no `#` characters in the `ResourcePath::source` and no `://` substrings in the `ResourcePath::label`
        let mut source_delimiter_chars_matched = 0;
        let mut last_found_source_index = 0;
        for (index, char) in chars {
            match char {
                ':' => {
                    source_delimiter_chars_matched = 1;
                }
                '/' => {
                    match source_delimiter_chars_matched {
                        1 => {
                            source_delimiter_chars_matched = 2;
                        }
                        2 => {
                            // If we haven't found our first `ResourcePath::source` yet, check to make sure it is valid and then store it.
                            if source_range.is_none() {
                                // If the `ResourcePath::source` contains a `#` character, it is invalid.
                                if label_range.is_some() {
                                    return Err(ParseAssetPathError::InvalidSourceSyntax);
                                }
                                source_range = Some(0..index - 2);
                                path_range.start = index + 1;
                            }
                            last_found_source_index = index - 2;
                            source_delimiter_chars_matched = 0;
                        }
                        _ => {}
                    }
                }
                '#' => {
                    path_range.end = index;
                    label_range = Some(index + 1..asset_path.len());
                    source_delimiter_chars_matched = 0;
                }
                _ => {
                    source_delimiter_chars_matched = 0;
                }
            }
        }
        // If we found an `ResourcePath::label`
        if let Some(range) = label_range.clone() {
            // If the `ResourcePath::label` contained a `://` substring, it is invalid.
            if range.start <= last_found_source_index {
                return Err(ParseAssetPathError::InvalidLabelSyntax);
            }
        }
        // Try to parse the range of indices that represents the `ResourcePath::source` portion of the `ResourcePath` to make sure it is not empty.
        // This would be the case if the input &str was something like `://some/file.test`
        let source = match source_range {
            Some(source_range) => {
                if source_range.is_empty() {
                    return Err(ParseAssetPathError::MissingSource);
                }
                Some(&asset_path[source_range])
            }
            None => None,
        };
        // Try to parse the range of indices that represents the `ResourcePath::label` portion of the `ResourcePath` to make sure it is not empty.
        // This would be the case if the input &str was something like `some/file.test#`.
        let label = match label_range {
            Some(label_range) => {
                if label_range.is_empty() {
                    return Err(ParseAssetPathError::MissingLabel);
                }
                Some(&asset_path[label_range])
            }
            None => None,
        };

        let path = Path::new(&asset_path[path_range]);
        Ok((source, path, label))
    }

    /// Creates a new [`ResourcePath`] from a [`Path`].
    #[inline]
    pub fn from_path(path: &'a Path) -> ResourcePath<'a> {
        ResourcePath {
            path: CowArc::Borrowed(path),
            source: ResourceSourceId::Default,
            label: None,
        }
    }

    /// Gets the "asset source", if one was defined. If none was defined, the default source
    /// will be used.
    #[inline]
    pub fn source(&self) -> &ResourceSourceId {
        &self.source
    }

    /// Gets the "sub-asset label".
    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Gets the "sub-asset label".
    #[inline]
    pub fn label_cow(&self) -> Option<CowArc<'a, str>> {
        self.label.clone()
    }

    /// Gets the path to the asset in the "virtual filesystem".
    #[inline]
    pub fn path(&self) -> &Path {
        self.path.deref()
    }

    /// Gets the path to the asset in the "virtual filesystem" without a label (if a label is currently set).
    #[inline]
    pub fn without_label(&self) -> ResourcePath<'_> {
        Self {
            source: self.source.clone(),
            path: self.path.clone(),
            label: None,
        }
    }

    /// Removes a "sub-asset label" from this [`ResourcePath`], if one was set.
    #[inline]
    pub fn remove_label(&mut self) {
        self.label = None;
    }

    /// Takes the "sub-asset label" from this [`ResourcePath`], if one was set.
    #[inline]
    pub fn take_label(&mut self) -> Option<CowArc<'a, str>> {
        self.label.take()
    }

    /// Returns this asset path with the given label. This will replace the previous
    /// label if it exists.
    #[inline]
    pub fn with_label(self, label: impl Into<CowArc<'a, str>>) -> ResourcePath<'a> {
        ResourcePath {
            source: self.source,
            path: self.path,
            label: Some(label.into()),
        }
    }

    /// Returns this asset path with the given asset source. This will replace the previous asset
    /// source if it exists.
    #[inline]
    pub fn with_source(self, source: impl Into<ResourceSourceId<'a>>) -> ResourcePath<'a> {
        ResourcePath {
            source: source.into(),
            path: self.path,
            label: self.label,
        }
    }

    /// Returns an [`ResourcePath`] for the parent folder of this path, if there is a parent folder in the path.
    pub fn parent(&self) -> Option<ResourcePath<'a>> {
        let path = match &self.path {
            CowArc::Borrowed(path) => CowArc::Borrowed(path.parent()?),
            CowArc::Static(path) => CowArc::Static(path.parent()?),
            CowArc::Owned(path) => path.parent()?.to_path_buf().into(),
        };
        Some(ResourcePath {
            source: self.source.clone(),
            label: None,
            path,
        })
    }

    /// Converts this into an "owned" value. If internally a value is borrowed, it will be cloned into an "owned [`Arc`]".
    /// If internally a value is a static reference, the static reference will be used unchanged.
    /// If internally a value is an "owned [`Arc`]", it will remain unchanged.
    ///
    /// [`Arc`]: std::sync::Arc
    pub fn into_owned(self) -> ResourcePath<'static> {
        ResourcePath {
            source: self.source.into_owned(),
            path: self.path.into_owned(),
            label: self.label.map(CowArc::into_owned),
        }
    }

    /// Clones this into an "owned" value. If internally a value is borrowed, it will be cloned into an "owned [`Arc`]".
    /// If internally a value is a static reference, the static reference will be used unchanged.
    /// If internally a value is an "owned [`Arc`]", the [`Arc`] will be cloned.
    ///
    /// [`Arc`]: std::sync::Arc
    #[inline]
    pub fn clone_owned(&self) -> ResourcePath<'static> {
        self.clone().into_owned()
    }

    /// Resolves a relative asset path via concatenation. The result will be an `ResourcePath` which
    /// is resolved relative to this "base" path.
    ///
    /// ```
    /// # use bevy_asset::ResourcePath;
    /// assert_eq!(ResourcePath::parse("a/b").resolve("c"), Ok(ResourcePath::parse("a/b/c")));
    /// assert_eq!(ResourcePath::parse("a/b").resolve("./c"), Ok(ResourcePath::parse("a/b/c")));
    /// assert_eq!(ResourcePath::parse("a/b").resolve("../c"), Ok(ResourcePath::parse("a/c")));
    /// assert_eq!(ResourcePath::parse("a/b").resolve("c.png"), Ok(ResourcePath::parse("a/b/c.png")));
    /// assert_eq!(ResourcePath::parse("a/b").resolve("/c"), Ok(ResourcePath::parse("c")));
    /// assert_eq!(ResourcePath::parse("a/b.png").resolve("#c"), Ok(ResourcePath::parse("a/b.png#c")));
    /// assert_eq!(ResourcePath::parse("a/b.png#c").resolve("#d"), Ok(ResourcePath::parse("a/b.png#d")));
    /// ```
    ///
    /// There are several cases:
    ///
    /// If the `path` argument begins with `#`, then it is considered an asset label, in which case
    /// the result is the base path with the label portion replaced.
    ///
    /// If the path argument begins with '/', then it is considered a 'full' path, in which
    /// case the result is a new `ResourcePath` consisting of the base path asset source
    /// (if there is one) with the path and label portions of the relative path. Note that a 'full'
    /// asset path is still relative to the asset source root, and not necessarily an absolute
    /// filesystem path.
    ///
    /// If the `path` argument begins with an asset source (ex: `http://`) then the entire base
    /// path is replaced - the result is the source, path and label (if any) of the `path`
    /// argument.
    ///
    /// Otherwise, the `path` argument is considered a relative path. The result is concatenated
    /// using the following algorithm:
    ///
    /// * The base path and the `path` argument are concatenated.
    /// * Path elements consisting of "/." or "&lt;name&gt;/.." are removed.
    ///
    /// If there are insufficient segments in the base path to match the ".." segments,
    /// then any left-over ".." segments are left as-is.
    pub fn resolve(&self, path: &str) -> Result<ResourcePath<'static>, ParseAssetPathError> {
        self.resolve_internal(path, false)
    }

    /// Resolves an embedded asset path via concatenation. The result will be an `ResourcePath` which
    /// is resolved relative to this path. This is similar in operation to `resolve`, except that
    /// the 'file' portion of the base path (that is, any characters after the last '/')
    /// is removed before concatenation, in accordance with the behavior specified in
    /// IETF RFC 1808 "Relative URIs".
    ///
    /// The reason for this behavior is that embedded URIs which start with "./" or "../" are
    /// relative to the *directory* containing the asset, not the asset file. This is consistent
    /// with the behavior of URIs in `JavaScript`, CSS, HTML and other web file formats. The
    /// primary use case for this method is resolving relative paths embedded within asset files,
    /// which are relative to the asset in which they are contained.
    ///
    /// ```
    /// # use bevy_asset::ResourcePath;
    /// assert_eq!(ResourcePath::parse("a/b").resolve_embed("c"), Ok(ResourcePath::parse("a/c")));
    /// assert_eq!(ResourcePath::parse("a/b").resolve_embed("./c"), Ok(ResourcePath::parse("a/c")));
    /// assert_eq!(ResourcePath::parse("a/b").resolve_embed("../c"), Ok(ResourcePath::parse("c")));
    /// assert_eq!(ResourcePath::parse("a/b").resolve_embed("c.png"), Ok(ResourcePath::parse("a/c.png")));
    /// assert_eq!(ResourcePath::parse("a/b").resolve_embed("/c"), Ok(ResourcePath::parse("c")));
    /// assert_eq!(ResourcePath::parse("a/b.png").resolve_embed("#c"), Ok(ResourcePath::parse("a/b.png#c")));
    /// assert_eq!(ResourcePath::parse("a/b.png#c").resolve_embed("#d"), Ok(ResourcePath::parse("a/b.png#d")));
    /// ```
    pub fn resolve_embed(&self, path: &str) -> Result<ResourcePath<'static>, ParseAssetPathError> {
        self.resolve_internal(path, true)
    }

    fn resolve_internal(
        &self,
        path: &str,
        replace: bool,
    ) -> Result<ResourcePath<'static>, ParseAssetPathError> {
        if let Some(label) = path.strip_prefix('#') {
            // It's a label only
            Ok(self.clone_owned().with_label(label.to_owned()))
        } else {
            let (source, rpath, rlabel) = ResourcePath::parse_internal(path)?;
            let mut base_path = PathBuf::from(self.path());
            if replace && !self.path.to_str().unwrap().ends_with('/') {
                // No error if base is empty (per RFC 1808).
                base_path.pop();
            }

            // Strip off leading slash
            let mut is_absolute = false;
            let rpath = match rpath.strip_prefix("/") {
                Ok(p) => {
                    is_absolute = true;
                    p
                }
                _ => rpath,
            };

            let mut result_path = if !is_absolute && source.is_none() {
                base_path
            } else {
                PathBuf::new()
            };
            result_path.push(rpath);
            result_path = normalize_path(result_path.as_path());

            Ok(ResourcePath {
                source: match source {
                    Some(source) => ResourceSourceId::Name(CowArc::Owned(source.into())),
                    None => self.source.clone_owned(),
                },
                path: CowArc::Owned(result_path.into()),
                label: rlabel.map(|l| CowArc::Owned(l.into())),
            })
        }
    }

    /// Returns the full extension (including multiple '.' values).
    /// Ex: Returns `"config.ron"` for `"my_asset.config.ron"`
    ///
    /// Also strips out anything following a `?` to handle query parameters in URIs
    pub fn get_full_extension(&self) -> Option<String> {
        let file_name = self.path().file_name()?.to_str()?;
        let index = file_name.find('.')?;
        let mut extension = file_name[index + 1..].to_lowercase();

        // Strip off any query parameters
        let query = extension.find('?');
        if let Some(offset) = query {
            extension.truncate(offset);
        }

        Some(extension)
    }

    pub(crate) fn iter_secondary_extensions(full_extension: &str) -> impl Iterator<Item = &str> {
        full_extension.chars().enumerate().filter_map(|(i, c)| {
            if c == '.' {
                Some(&full_extension[i + 1..])
            } else {
                None
            }
        })
    }
}

impl From<&'static str> for ResourcePath<'static> {
    #[inline]
    fn from(asset_path: &'static str) -> Self {
        let (source, path, label) = Self::parse_internal(asset_path).unwrap();
        ResourcePath {
            source: source.into(),
            path: CowArc::Static(path),
            label: label.map(CowArc::Static),
        }
    }
}

impl<'a> From<&'a String> for ResourcePath<'a> {
    #[inline]
    fn from(asset_path: &'a String) -> Self {
        ResourcePath::parse(asset_path.as_str())
    }
}

impl From<String> for ResourcePath<'static> {
    #[inline]
    fn from(asset_path: String) -> Self {
        ResourcePath::parse(asset_path.as_str()).into_owned()
    }
}

impl From<&'static Path> for ResourcePath<'static> {
    #[inline]
    fn from(path: &'static Path) -> Self {
        Self {
            source: ResourceSourceId::Default,
            path: CowArc::Static(path),
            label: None,
        }
    }
}

impl From<PathBuf> for ResourcePath<'static> {
    #[inline]
    fn from(path: PathBuf) -> Self {
        Self {
            source: ResourceSourceId::Default,
            path: path.into(),
            label: None,
        }
    }
}

impl<'a, 'b> From<&'a ResourcePath<'b>> for ResourcePath<'b> {
    fn from(value: &'a ResourcePath<'b>) -> Self {
        value.clone()
    }
}

impl<'a> From<ResourcePath<'a>> for PathBuf {
    fn from(value: ResourcePath<'a>) -> Self {
        value.path().to_path_buf()
    }
}

/// Normalizes the path by collapsing all occurrences of '.' and '..' dot-segments where possible
/// as per [RFC 1808](https://datatracker.ietf.org/doc/html/rfc1808)
pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    let mut result_path = PathBuf::new();
    for elt in path.iter() {
        if elt == "." {
            // Skip
        } else if elt == ".." {
            if !result_path.pop() {
                // Preserve ".." if insufficient matches (per RFC 1808).
                result_path.push(elt);
            }
        } else {
            result_path.push(elt);
        }
    }
    result_path
}
