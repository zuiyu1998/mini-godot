use std::{path::PathBuf, sync::Arc};

use super::prelude::{CompressedImageFormats, Image, ImageFormat, ImageSampler, ImageType};
use mini_core::thiserror::{self, Error};
use mini_resource::prelude::{FileLoadError, ResourceIo, ResourceLoader};

pub(crate) const IMG_FILE_EXTENSIONS: &[&str] = &["png"];

/// Loader for images that can be read by the `image` crate.
#[derive(Clone, Default)]
pub struct ImageLoader {
    supported_compressed_formats: CompressedImageFormats,
}

#[derive(Debug, Clone, Default)]
pub struct ImageLoaderSettings {
    pub format: ImageFormatSetting,
    pub is_srgb: bool,
    pub sampler: ImageSampler,
}

#[derive(Debug, Error)]
pub enum ImageLoaderError {
    #[error("Could load image: {0}")]
    Io(#[from] FileLoadError),
    #[error("Could not load texture file: {0}")]
    FileTexture(#[from] FileTextureError),
}

#[derive(Error, Debug)]
pub enum TextureError {
    #[error("invalid image mime type: {0}")]
    InvalidImageMimeType(String),
    #[error("invalid image extension: {0}")]
    InvalidImageExtension(String),
    #[error("failed to load an image: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("unsupported texture format: {0}")]
    UnsupportedTextureFormat(String),
    #[error("supercompression not supported: {0}")]
    SuperCompressionNotSupported(String),
    #[error("failed to load an image: {0}")]
    SuperDecompressionError(String),
    #[error("invalid data: {0}")]
    InvalidData(String),
    #[error("transcode error: {0}")]
    TranscodeError(String),
    #[error("format requires transcoding: {0:?}")]
    FormatRequiresTranscodingError(TranscodeFormat),
    /// Only cubemaps with six faces are supported.
    #[error("only cubemaps with six faces are supported")]
    IncompleteCubemap,
}

#[derive(Clone, Copy, Debug)]
pub enum TranscodeFormat {
    Etc1s,
    Uastc(DataFormat),
    // Has to be transcoded to R8Unorm for use with `wgpu`
    R8UnormSrgb,
    // Has to be transcoded to R8G8Unorm for use with `wgpu`
    Rg8UnormSrgb,
    // Has to be transcoded to Rgba8 for use with `wgpu`
    Rgb8,
}

#[derive(Clone, Copy, Debug)]
pub enum DataFormat {
    Rgb,
    Rgba,
    Rrr,
    Rrrg,
    Rg,
}

#[derive(Error, Debug)]
pub struct FileTextureError {
    error: TextureError,
    path: String,
}
impl std::fmt::Display for FileTextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Error reading image file {}: {}, this is an error in `mini_render`.",
            self.path, self.error
        )
    }
}

impl ResourceLoader for ImageLoader {
    type ResourceData = Image;
    type Settings = ImageLoaderSettings;
    type Error = ImageLoaderError;
    async fn load(
        &self,
        path: PathBuf,
        io: Arc<dyn ResourceIo>,
        settings: &Self::Settings,
    ) -> Result<Image, Self::Error> {
        let mut bytes = io.load_file(&path).await?;
        let image_type = match settings.format {
            ImageFormatSetting::FromExtension => {
                // use the file extension for the image type
                let ext = path.extension().unwrap().to_str().unwrap();
                ImageType::Extension(ext)
            }
            ImageFormatSetting::Format(format) => ImageType::Format(format),
            ImageFormatSetting::Guess => {
                let format = image::guess_format(&bytes).map_err(|err| FileTextureError {
                    error: err.into(),
                    path: format!("{}", path.display()),
                })?;
                ImageType::Format(ImageFormat::from_image_crate_format(format).ok_or_else(
                    || FileTextureError {
                        error: TextureError::UnsupportedTextureFormat(format!("{format:?}")),
                        path: format!("{}", path.display()),
                    },
                )?)
            }
        };
        Ok(Image::from_buffer(
            &bytes,
            image_type,
            self.supported_compressed_formats,
            settings.is_srgb,
            settings.sampler.clone(),
        )
        .map_err(|err| FileTextureError {
            error: err,
            path: format!("{}", path.display()),
        })?)
    }

    fn extensions(&self) -> &[&str] {
        IMG_FILE_EXTENSIONS
    }
}

#[derive(Default, Debug, Clone)]
pub enum ImageFormatSetting {
    #[default]
    FromExtension,
    Format(ImageFormat),
    Guess,
}
