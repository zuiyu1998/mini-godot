use std::path::{Path, PathBuf};

use mini_core::{
    future::{BoxedFuture, ConditionalSendFuture},
    futures_io::{AsyncRead, AsyncSeek},
    futures_lite::{self, Stream},
    stackfuture::StackFuture,
    thiserror::Error,
};

pub type PathStream = dyn Stream<Item = PathBuf> + Unpin + Send;

#[derive(Error, Debug)]
pub enum AssetReaderError {
    /// Path not found.
    #[error("Path not found: {0}")]
    NotFound(PathBuf),

    /// Encountered an I/O error while loading an asset.
    #[error("Encountered an I/O error while loading asset: {0}")]
    Io(std::io::Error),
}

impl PartialEq for AssetReaderError {
    /// Equality comparison for `AssetReaderError::Io` is not full (only through `ErrorKind` of inner error)
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NotFound(path), Self::NotFound(other_path)) => path == other_path,
            (Self::Io(error), Self::Io(other_error)) => error.kind() == other_error.kind(),
            _ => false,
        }
    }
}

impl Eq for AssetReaderError {}

impl From<std::io::Error> for AssetReaderError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub const STACK_FUTURE_SIZE: usize = 10 * std::mem::size_of::<&()>();

/// A type returned from [`AssetReader::read`], which is used to read the contents of a file
/// (or virtual file) corresponding to an asset.
///
/// This is essentially a trait alias for types implementing  [`AsyncRead`] and [`AsyncSeek`].
/// The only reason a blanket implementation is not provided for applicable types is to allow
/// implementors to override the provided implementation of [`Reader::read_to_end`].
pub trait Reader: AsyncRead + AsyncSeek + Unpin + Send + Sync {
    /// Reads the entire contents of this reader and appends them to a vec.
    ///
    /// # Note for implementors
    /// You should override the provided implementation if you can fill up the buffer more
    /// efficiently than the default implementation, which calls `poll_read` repeatedly to
    /// fill up the buffer 32 bytes at a time.
    fn read_to_end<'a>(
        &'a mut self,
        buf: &'a mut Vec<u8>,
    ) -> StackFuture<'a, std::io::Result<usize>, STACK_FUTURE_SIZE> {
        let future = futures_lite::AsyncReadExt::read_to_end(self, buf);
        StackFuture::from(future)
    }
}

impl Reader for Box<dyn Reader + '_> {
    fn read_to_end<'a>(
        &'a mut self,
        buf: &'a mut Vec<u8>,
    ) -> StackFuture<'a, std::io::Result<usize>, STACK_FUTURE_SIZE> {
        (**self).read_to_end(buf)
    }
}

/// A future that returns a value or an [`AssetReaderError`]
pub trait AssetReaderFuture:
    ConditionalSendFuture<Output = Result<Self::Value, AssetReaderError>>
{
    type Value;
}

impl<F, T> AssetReaderFuture for F
where
    F: ConditionalSendFuture<Output = Result<T, AssetReaderError>>,
{
    type Value = T;
}

/// Performs read operations on an asset storage. [`AssetReader`] exposes a "virtual filesystem"
/// API, where asset bytes and asset metadata bytes are both stored and accessible for a given
/// `path`. This trait is not object safe, if needed use a dyn [`ErasedAssetReader`] instead.
///
/// Also see [`AssetWriter`].
pub trait AssetReader: Send + Sync + 'static {
    /// Returns a future to load the full file data at the provided path.
    ///
    /// # Note for implementors
    /// The preferred style for implementing this method is an `async fn` returning an opaque type.
    ///
    /// ```no_run
    /// # use std::path::Path;
    /// # use bevy_asset::{prelude::*, io::{AssetReader, PathStream, Reader, AssetReaderError}};
    /// # struct MyReader;
    /// impl AssetReader for MyReader {
    ///     async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
    ///         // ...
    ///         # let val: Box<dyn Reader> = unimplemented!(); Ok(val)
    ///     }
    ///     # async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
    ///     #     let val: Box<dyn Reader> = unimplemented!(); Ok(val) }
    ///     # async fn read_directory<'a>(&'a self, path: &'a Path) -> Result<Box<PathStream>, AssetReaderError> { unimplemented!() }
    ///     # async fn is_directory<'a>(&'a self, path: &'a Path) -> Result<bool, AssetReaderError> { unimplemented!() }
    ///     # async fn read_meta_bytes<'a>(&'a self, path: &'a Path) -> Result<Vec<u8>, AssetReaderError> { unimplemented!() }
    /// }
    /// ```
    fn read<'a>(&'a self, path: &'a Path) -> impl AssetReaderFuture<Value: Reader + 'a>;
    /// Returns a future to load the full file data at the provided path.
    fn read_meta<'a>(&'a self, path: &'a Path) -> impl AssetReaderFuture<Value: Reader + 'a>;
    /// Returns an iterator of directory entry names at the provided path.
    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<Box<PathStream>, AssetReaderError>>;
    /// Returns true if the provided path points to a directory.
    fn is_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<bool, AssetReaderError>>;
    /// Reads asset metadata bytes at the given `path` into a [`Vec<u8>`]. This is a convenience
    /// function that wraps [`AssetReader::read_meta`] by default.
    fn read_meta_bytes<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<Vec<u8>, AssetReaderError>> {
        async {
            let mut meta_reader = self.read_meta(path).await?;
            let mut meta_bytes = Vec::new();
            meta_reader.read_to_end(&mut meta_bytes).await?;
            Ok(meta_bytes)
        }
    }
}

/// Equivalent to an [`AssetReader`] but using boxed futures, necessary eg. when using a `dyn AssetReader`,
/// as [`AssetReader`] isn't currently object safe.
pub trait ErasedAssetReader: Send + Sync + 'static {
    /// Returns a future to load the full file data at the provided path.
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<Result<Box<dyn Reader + 'a>, AssetReaderError>>;
    /// Returns a future to load the full file data at the provided path.
    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<Result<Box<dyn Reader + 'a>, AssetReaderError>>;
    /// Returns an iterator of directory entry names at the provided path.
    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<Result<Box<PathStream>, AssetReaderError>>;
    /// Returns true if the provided path points to a directory.
    fn is_directory<'a>(&'a self, path: &'a Path) -> BoxedFuture<Result<bool, AssetReaderError>>;
    /// Reads asset metadata bytes at the given `path` into a [`Vec<u8>`]. This is a convenience
    /// function that wraps [`ErasedAssetReader::read_meta`] by default.
    fn read_meta_bytes<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<Result<Vec<u8>, AssetReaderError>>;
}

impl<T: AssetReader> ErasedAssetReader for T {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<Result<Box<dyn Reader + 'a>, AssetReaderError>> {
        Box::pin(async {
            let reader = Self::read(self, path).await?;
            Ok(Box::new(reader) as Box<dyn Reader>)
        })
    }
    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<Result<Box<dyn Reader + 'a>, AssetReaderError>> {
        Box::pin(async {
            let reader = Self::read_meta(self, path).await?;
            Ok(Box::new(reader) as Box<dyn Reader>)
        })
    }
    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<Result<Box<PathStream>, AssetReaderError>> {
        Box::pin(Self::read_directory(self, path))
    }
    fn is_directory<'a>(&'a self, path: &'a Path) -> BoxedFuture<Result<bool, AssetReaderError>> {
        Box::pin(Self::is_directory(self, path))
    }
    fn read_meta_bytes<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<Result<Vec<u8>, AssetReaderError>> {
        Box::pin(Self::read_meta_bytes(self, path))
    }
}
