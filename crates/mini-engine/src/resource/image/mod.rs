use std::{io, sync::Arc};

use mini_core::{
    bytemuck::cast_slice,
    thiserror::{self, Error},
    type_uuid::TypeUuidProvider,
    uuid::{uuid, Uuid},
};
use mini_resource::{
    io::{FileLoadError, ResourceIo},
    loader::ResourceLoader,
    resource::ResourceData,
};

use image::DynamicImage;

#[derive(Debug, TypeUuidProvider)]
#[type_uuid(id = "21613484-7145-4d1c-87d8-62fa767560ab")]
pub struct Image {
    pub data: Vec<u8>,
}

impl Image {
    pub fn from_buffer(buffer: &[u8], format: ImageFormat) -> Result<Self, ImageLoaderError> {
        let image_crate_format = format.as_image_crate_format();
        let mut reader = image::ImageReader::new(std::io::Cursor::new(buffer));
        reader.set_format(image_crate_format);
        reader.no_limits();
        let dyn_img = reader.decode()?;

        todo!()
    }

    pub fn from_dynamic(dyn_img: DynamicImage, is_srgb: bool) -> Image {
        let width;
        let height;

        let data: Vec<u8>;

        match dyn_img {
            DynamicImage::ImageLuma8(image) => {
                let i = DynamicImage::ImageLuma8(image).into_rgba8();
                width = i.width();
                height = i.height();

                data = i.into_raw();
            }
            DynamicImage::ImageLumaA8(image) => {
                let i = DynamicImage::ImageLumaA8(image).into_rgba8();
                width = i.width();
                height = i.height();

                data = i.into_raw();
            }
            DynamicImage::ImageRgb8(image) => {
                let i = DynamicImage::ImageRgb8(image).into_rgba8();
                width = i.width();
                height = i.height();

                data = i.into_raw();
            }
            DynamicImage::ImageRgba8(image) => {
                width = image.width();
                height = image.height();

                data = image.into_raw();
            }
            DynamicImage::ImageLuma16(image) => {
                width = image.width();
                height = image.height();

                let raw_data = image.into_raw();

                data = cast_slice(&raw_data).to_owned();
            }
            DynamicImage::ImageLumaA16(image) => {
                width = image.width();
                height = image.height();

                let raw_data = image.into_raw();

                data = cast_slice(&raw_data).to_owned();
            }
            DynamicImage::ImageRgb16(image) => {
                let i = DynamicImage::ImageRgb16(image).into_rgba16();
                width = i.width();
                height = i.height();

                let raw_data = i.into_raw();

                data = cast_slice(&raw_data).to_owned();
            }
            DynamicImage::ImageRgba16(image) => {
                width = image.width();
                height = image.height();

                let raw_data = image.into_raw();

                data = cast_slice(&raw_data).to_owned();
            }
            DynamicImage::ImageRgb32F(image) => {
                width = image.width();
                height = image.height();

                let mut local_data =
                    Vec::with_capacity(width as usize * height as usize * format.pixel_size());

                for pixel in image.into_raw().chunks_exact(3) {
                    // TODO: use the array_chunks method once stabilised
                    // https://github.com/rust-lang/rust/issues/74985
                    let r = pixel[0];
                    let g = pixel[1];
                    let b = pixel[2];
                    let a = 1f32;

                    local_data.extend_from_slice(&r.to_ne_bytes());
                    local_data.extend_from_slice(&g.to_ne_bytes());
                    local_data.extend_from_slice(&b.to_ne_bytes());
                    local_data.extend_from_slice(&a.to_ne_bytes());
                }

                data = local_data;
            }
            DynamicImage::ImageRgba32F(image) => {
                width = image.width();
                height = image.height();
                format = TextureFormat::Rgba32Float;

                let raw_data = image.into_raw();

                data = cast_slice(&raw_data).to_owned();
            }
            // DynamicImage is now non exhaustive, catch future variants and convert them
            _ => {
                let image = dyn_img.into_rgba8();
                width = image.width();
                height = image.height();
                format = TextureFormat::Rgba8UnormSrgb;

                data = image.into_raw();
            }
        }

        Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            format,
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ImageFormat {
    Png,
}

impl ImageFormat {
    pub fn from_image_crate_format(format: image::ImageFormat) -> Option<ImageFormat> {
        match format {
            image::ImageFormat::Png => Some(ImageFormat::Png),
            _ => None,
        }
    }

    pub fn as_image_crate_format(&self) -> image::ImageFormat {
        match *self {
            ImageFormat::Png => image::ImageFormat::Png,
        }
    }
}

#[derive(Debug, Error)]
pub enum ImageLoaderError {
    #[error("{0}")]
    FileLoadError(#[from] FileLoadError),

    #[error("{0}")]
    IoError(#[from] io::Error),

    #[error("{0}")]
    ImageError(#[from] image::error::ImageError),
}

impl ResourceData for Image {}

pub struct PngLoader;

impl ResourceLoader for PngLoader {
    type Error = ImageLoaderError;

    type Settings = ();

    type ResourceData = Image;

    async fn load(
        &self,
        path: std::path::PathBuf,
        io: Arc<dyn ResourceIo>,
        _settings: &Self::Settings,
    ) -> Result<Self::ResourceData, Self::Error> {
        let bytes = io.load_file(&path).await?;

        let format =
            image::guess_format(&bytes).map_err(|err| FileLoadError::Custom(err.to_string()))?;

        let format = ImageFormat::from_image_crate_format(format)
            .ok_or(FileLoadError::Custom("not suport".to_string()))?;

        todo!()
    }

    fn extensions(&self) -> &[&str] {
        &[".png"]
    }
}
