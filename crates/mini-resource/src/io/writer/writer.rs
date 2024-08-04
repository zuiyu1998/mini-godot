use std::path::Path;

use mini_core::{
    future::{BoxedFuture, ConditionalSendFuture},
    futures_io::AsyncWrite,
    futures_lite::AsyncWriteExt,
    thiserror::Error,
};

pub type Writer = dyn AsyncWrite + Unpin + Send + Sync;

#[derive(Error, Debug)]
pub enum AssetWriterError {
    /// Encountered an I/O error while loading an asset.
    #[error("encountered an io error while loading asset: {0}")]
    Io(#[from] std::io::Error),
}

/// Preforms write operations on an asset storage. [`AssetWriter`] exposes a "virtual filesystem"
/// API, where asset bytes and asset metadata bytes are both stored and accessible for a given
/// `path`. This trait is not object safe, if needed use a dyn [`ErasedAssetWriter`] instead.
///
/// Also see [`AssetReader`].
pub trait AssetWriter: Send + Sync + 'static {
    /// Writes the full asset bytes at the provided path.
    fn write<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<Box<Writer>, AssetWriterError>>;
    /// Writes the full asset meta bytes at the provided path.
    /// This _should not_ include storage specific extensions like `.meta`.
    fn write_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<Box<Writer>, AssetWriterError>>;
    /// Removes the asset stored at the given path.
    fn remove<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<(), AssetWriterError>>;
    /// Removes the asset meta stored at the given path.
    /// This _should not_ include storage specific extensions like `.meta`.
    fn remove_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<(), AssetWriterError>>;
    /// Renames the asset at `old_path` to `new_path`
    fn rename<'a>(
        &'a self,
        old_path: &'a Path,
        new_path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<(), AssetWriterError>>;
    /// Renames the asset meta for the asset at `old_path` to `new_path`.
    /// This _should not_ include storage specific extensions like `.meta`.
    fn rename_meta<'a>(
        &'a self,
        old_path: &'a Path,
        new_path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<(), AssetWriterError>>;
    /// Removes the directory at the given path, including all assets _and_ directories in that directory.
    fn remove_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<(), AssetWriterError>>;
    /// Removes the directory at the given path, but only if it is completely empty. This will return an error if the
    /// directory is not empty.
    fn remove_empty_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<(), AssetWriterError>>;
    /// Removes all assets (and directories) in this directory, resulting in an empty directory.
    fn remove_assets_in_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<(), AssetWriterError>>;
    /// Writes the asset `bytes` to the given `path`.
    fn write_bytes<'a>(
        &'a self,
        path: &'a Path,
        bytes: &'a [u8],
    ) -> impl ConditionalSendFuture<Output = Result<(), AssetWriterError>> {
        async {
            let mut writer = self.write(path).await?;
            writer.write_all(bytes).await?;
            writer.flush().await?;
            Ok(())
        }
    }
    /// Writes the asset meta `bytes` to the given `path`.
    fn write_meta_bytes<'a>(
        &'a self,
        path: &'a Path,
        bytes: &'a [u8],
    ) -> impl ConditionalSendFuture<Output = Result<(), AssetWriterError>> {
        async {
            let mut meta_writer = self.write_meta(path).await?;
            meta_writer.write_all(bytes).await?;
            meta_writer.flush().await?;
            Ok(())
        }
    }
}

/// Equivalent to an [`AssetWriter`] but using boxed futures, necessary eg. when using a `dyn AssetWriter`,
/// as [`AssetWriter`] isn't currently object safe.
pub trait ErasedAssetWriter: Send + Sync + 'static {
    /// Writes the full asset bytes at the provided path.
    fn write<'a>(&'a self, path: &'a Path) -> BoxedFuture<Result<Box<Writer>, AssetWriterError>>;
    /// Writes the full asset meta bytes at the provided path.
    /// This _should not_ include storage specific extensions like `.meta`.
    fn write_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<Result<Box<Writer>, AssetWriterError>>;
    /// Removes the asset stored at the given path.
    fn remove<'a>(&'a self, path: &'a Path) -> BoxedFuture<Result<(), AssetWriterError>>;
    /// Removes the asset meta stored at the given path.
    /// This _should not_ include storage specific extensions like `.meta`.
    fn remove_meta<'a>(&'a self, path: &'a Path) -> BoxedFuture<Result<(), AssetWriterError>>;
    /// Renames the asset at `old_path` to `new_path`
    fn rename<'a>(
        &'a self,
        old_path: &'a Path,
        new_path: &'a Path,
    ) -> BoxedFuture<Result<(), AssetWriterError>>;
    /// Renames the asset meta for the asset at `old_path` to `new_path`.
    /// This _should not_ include storage specific extensions like `.meta`.
    fn rename_meta<'a>(
        &'a self,
        old_path: &'a Path,
        new_path: &'a Path,
    ) -> BoxedFuture<Result<(), AssetWriterError>>;
    /// Removes the directory at the given path, including all assets _and_ directories in that directory.
    fn remove_directory<'a>(&'a self, path: &'a Path) -> BoxedFuture<Result<(), AssetWriterError>>;
    /// Removes the directory at the given path, but only if it is completely empty. This will return an error if the
    /// directory is not empty.
    fn remove_empty_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<Result<(), AssetWriterError>>;
    /// Removes all assets (and directories) in this directory, resulting in an empty directory.
    fn remove_assets_in_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<Result<(), AssetWriterError>>;
    /// Writes the asset `bytes` to the given `path`.
    fn write_bytes<'a>(
        &'a self,
        path: &'a Path,
        bytes: &'a [u8],
    ) -> BoxedFuture<Result<(), AssetWriterError>>;
    /// Writes the asset meta `bytes` to the given `path`.
    fn write_meta_bytes<'a>(
        &'a self,
        path: &'a Path,
        bytes: &'a [u8],
    ) -> BoxedFuture<Result<(), AssetWriterError>>;
}

impl<T: AssetWriter> ErasedAssetWriter for T {
    fn write<'a>(&'a self, path: &'a Path) -> BoxedFuture<Result<Box<Writer>, AssetWriterError>> {
        Box::pin(Self::write(self, path))
    }
    fn write_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<Result<Box<Writer>, AssetWriterError>> {
        Box::pin(Self::write_meta(self, path))
    }
    fn remove<'a>(&'a self, path: &'a Path) -> BoxedFuture<Result<(), AssetWriterError>> {
        Box::pin(Self::remove(self, path))
    }
    fn remove_meta<'a>(&'a self, path: &'a Path) -> BoxedFuture<Result<(), AssetWriterError>> {
        Box::pin(Self::remove_meta(self, path))
    }
    fn rename<'a>(
        &'a self,
        old_path: &'a Path,
        new_path: &'a Path,
    ) -> BoxedFuture<Result<(), AssetWriterError>> {
        Box::pin(Self::rename(self, old_path, new_path))
    }
    fn rename_meta<'a>(
        &'a self,
        old_path: &'a Path,
        new_path: &'a Path,
    ) -> BoxedFuture<Result<(), AssetWriterError>> {
        Box::pin(Self::rename_meta(self, old_path, new_path))
    }
    fn remove_directory<'a>(&'a self, path: &'a Path) -> BoxedFuture<Result<(), AssetWriterError>> {
        Box::pin(Self::remove_directory(self, path))
    }
    fn remove_empty_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<Result<(), AssetWriterError>> {
        Box::pin(Self::remove_empty_directory(self, path))
    }
    fn remove_assets_in_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<Result<(), AssetWriterError>> {
        Box::pin(Self::remove_assets_in_directory(self, path))
    }
    fn write_bytes<'a>(
        &'a self,
        path: &'a Path,
        bytes: &'a [u8],
    ) -> BoxedFuture<Result<(), AssetWriterError>> {
        Box::pin(Self::write_bytes(self, path, bytes))
    }
    fn write_meta_bytes<'a>(
        &'a self,
        path: &'a Path,
        bytes: &'a [u8],
    ) -> BoxedFuture<Result<(), AssetWriterError>> {
        Box::pin(Self::write_meta_bytes(self, path, bytes))
    }
}
