use mini_core::{
    bitflags, bytemuck,
    prelude::TypeUuidProvider,
    uuid::{uuid, Uuid},
};
use mini_resource::prelude::ResourceData;

use super::prelude::TextureError;
use crate::renderer::prelude::MiniDefault;

use image::DynamicImage;
use wgpu::{Extent3d, TextureDimension, TextureFormat};

///图片资源
#[derive(TypeUuidProvider, ResourceData, Debug)]
#[type_uuid(id = "5fb10a22-4ea9-4a13-a58c-82f2734aefd8")]
pub struct Image {
    //数据
    pub data: Vec<u8>,

    // TODO: this nesting makes accessing Image metadata verbose. Either flatten out descriptor or add accessors
    pub texture_descriptor: wgpu::TextureDescriptor<'static>,

    //图形的采样信息
    pub sampler: ImageSampler,
    pub texture_view_descriptor: Option<wgpu::TextureViewDescriptor<'static>>,
}

impl Default for Image {
    /// default is a 1x1x1 all '1.0' texture
    fn default() -> Self {
        let format = TextureFormat::mini_default();
        let data = vec![255; format.pixel_size()];
        Image {
            data,
            texture_descriptor: wgpu::TextureDescriptor {
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                format,
                dimension: TextureDimension::D2,
                label: None,
                mip_level_count: 1,
                sample_count: 1,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            sampler: ImageSampler::Default,
            texture_view_descriptor: None,
        }
    }
}

impl Image {
    /// Creates a new image from raw binary data and the corresponding metadata.
    ///
    /// # Panics
    /// Panics if the length of the `data`, volume of the `size` and the size of the `format`
    /// do not match.
    pub fn new(
        size: Extent3d,
        dimension: TextureDimension,
        data: Vec<u8>,
        format: TextureFormat,
    ) -> Self {
        debug_assert_eq!(
            size.volume() * format.pixel_size(),
            data.len(),
            "Pixel data, size and format have to match",
        );
        let mut image = Self {
            data,
            ..Default::default()
        };
        image.texture_descriptor.dimension = dimension;
        image.texture_descriptor.size = size;
        image.texture_descriptor.format = format;
        image
    }

    /// Converts a [`DynamicImage`] to an [`Image`].
    pub fn from_dynamic(dyn_img: DynamicImage, is_srgb: bool) -> Image {
        use bytemuck::cast_slice;
        let width;
        let height;

        let data: Vec<u8>;
        let format: TextureFormat;

        match dyn_img {
            DynamicImage::ImageLuma8(image) => {
                let i = DynamicImage::ImageLuma8(image).into_rgba8();
                width = i.width();
                height = i.height();
                format = if is_srgb {
                    TextureFormat::Rgba8UnormSrgb
                } else {
                    TextureFormat::Rgba8Unorm
                };

                data = i.into_raw();
            }
            DynamicImage::ImageLumaA8(image) => {
                let i = DynamicImage::ImageLumaA8(image).into_rgba8();
                width = i.width();
                height = i.height();
                format = if is_srgb {
                    TextureFormat::Rgba8UnormSrgb
                } else {
                    TextureFormat::Rgba8Unorm
                };

                data = i.into_raw();
            }
            DynamicImage::ImageRgb8(image) => {
                let i = DynamicImage::ImageRgb8(image).into_rgba8();
                width = i.width();
                height = i.height();
                format = if is_srgb {
                    TextureFormat::Rgba8UnormSrgb
                } else {
                    TextureFormat::Rgba8Unorm
                };

                data = i.into_raw();
            }
            DynamicImage::ImageRgba8(image) => {
                width = image.width();
                height = image.height();
                format = if is_srgb {
                    TextureFormat::Rgba8UnormSrgb
                } else {
                    TextureFormat::Rgba8Unorm
                };

                data = image.into_raw();
            }
            DynamicImage::ImageLuma16(image) => {
                width = image.width();
                height = image.height();
                format = TextureFormat::R16Uint;

                let raw_data = image.into_raw();

                data = cast_slice(&raw_data).to_owned();
            }
            DynamicImage::ImageLumaA16(image) => {
                width = image.width();
                height = image.height();
                format = TextureFormat::Rg16Uint;

                let raw_data = image.into_raw();

                data = cast_slice(&raw_data).to_owned();
            }
            DynamicImage::ImageRgb16(image) => {
                let i = DynamicImage::ImageRgb16(image).into_rgba16();
                width = i.width();
                height = i.height();
                format = TextureFormat::Rgba16Unorm;

                let raw_data = i.into_raw();

                data = cast_slice(&raw_data).to_owned();
            }
            DynamicImage::ImageRgba16(image) => {
                width = image.width();
                height = image.height();
                format = TextureFormat::Rgba16Unorm;

                let raw_data = image.into_raw();

                data = cast_slice(&raw_data).to_owned();
            }
            DynamicImage::ImageRgb32F(image) => {
                width = image.width();
                height = image.height();
                format = TextureFormat::Rgba32Float;

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

    pub fn from_buffer(
        buffer: &[u8],
        image_type: ImageType,
        #[allow(unused_variables)] supported_compressed_formats: CompressedImageFormats,
        is_srgb: bool,
        image_sampler: ImageSampler,
    ) -> Result<Image, TextureError> {
        let format = image_type.to_image_format()?;

        // Load the image in the expected format.
        // Some formats like PNG allow for R or RG textures too, so the texture
        // format needs to be determined. For RGB textures an alpha channel
        // needs to be added, so the image data needs to be converted in those
        // cases.

        let mut image = match format {
            _ => {
                let image_crate_format = format
                    .as_image_crate_format()
                    .ok_or_else(|| TextureError::UnsupportedTextureFormat(format!("{format:?}")))?;
                let mut reader = image::ImageReader::new(std::io::Cursor::new(buffer));
                reader.set_format(image_crate_format);
                reader.no_limits();
                let dyn_img = reader.decode()?;
                Self::from_dynamic(dyn_img, is_srgb)
            }
        };
        image.sampler = image_sampler;
        Ok(image)
    }
}

/// Used to calculate the volume of an item.
pub trait Volume {
    fn volume(&self) -> usize;
}

impl Volume for Extent3d {
    /// Calculates the volume of the [`Extent3d`].
    fn volume(&self) -> usize {
        (self.width * self.height * self.depth_or_array_layers) as usize
    }
}

/// Extends the wgpu [`TextureFormat`] with information about the pixel.
pub trait TextureFormatPixelInfo {
    /// Returns the size of a pixel in bytes of the format.
    fn pixel_size(&self) -> usize;
}

impl TextureFormatPixelInfo for TextureFormat {
    fn pixel_size(&self) -> usize {
        let info = self;
        match info.block_dimensions() {
            (1, 1) => info.block_copy_size(None).unwrap() as usize,
            _ => panic!("Using pixel_size for compressed textures is invalid"),
        }
    }
}

bitflags::bitflags! {
    #[derive(Default, Clone, Copy, Eq, PartialEq, Debug)]
    #[repr(transparent)]
    pub struct CompressedImageFormats: u32 {
        const NONE     = 0;
        const ASTC_LDR = 1 << 0;
        const BC       = 1 << 1;
        const ETC2     = 1 << 2;
    }
}

#[derive(Debug)]
pub enum ImageType<'a> {
    /// The mime type of an image, for example `"image/png"`.
    MimeType(&'a str),
    /// The extension of an image file, for example `"png"`.
    Extension(&'a str),
    /// The direct format of the image
    Format(ImageFormat),
}

impl<'a> ImageType<'a> {
    pub fn to_image_format(&self) -> Result<ImageFormat, TextureError> {
        match self {
            ImageType::MimeType(mime_type) => ImageFormat::from_mime_type(mime_type)
                .ok_or_else(|| TextureError::InvalidImageMimeType(mime_type.to_string())),
            ImageType::Extension(extension) => ImageFormat::from_extension(extension)
                .ok_or_else(|| TextureError::InvalidImageExtension(extension.to_string())),
            ImageType::Format(format) => Ok(*format),
        }
    }
}

//支持的图片格式
#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Avif,
    Basis,
    Bmp,
    Dds,
    Farbfeld,
    Gif,
    OpenExr,
    Hdr,
    Ico,
    Jpeg,
    Ktx2,
    Png,
    Pnm,
    Tga,
    Tiff,
    WebP,
}

impl ImageFormat {
    pub fn from_mime_type(mime_type: &str) -> Option<Self> {
        Some(match mime_type.to_ascii_lowercase().as_str() {
            "image/avif" => ImageFormat::Avif,
            "image/bmp" | "image/x-bmp" => ImageFormat::Bmp,
            "image/vnd-ms.dds" => ImageFormat::Dds,
            "image/vnd.radiance" => ImageFormat::Hdr,
            "image/gif" => ImageFormat::Gif,
            "image/x-icon" => ImageFormat::Ico,
            "image/jpeg" => ImageFormat::Jpeg,
            "image/ktx2" => ImageFormat::Ktx2,
            "image/png" => ImageFormat::Png,
            "image/x-exr" => ImageFormat::OpenExr,
            "image/x-portable-bitmap"
            | "image/x-portable-graymap"
            | "image/x-portable-pixmap"
            | "image/x-portable-anymap" => ImageFormat::Pnm,
            "image/x-targa" | "image/x-tga" => ImageFormat::Tga,
            "image/tiff" => ImageFormat::Tiff,
            "image/webp" => ImageFormat::WebP,
            _ => return None,
        })
    }

    pub fn from_extension(extension: &str) -> Option<Self> {
        Some(match extension.to_ascii_lowercase().as_str() {
            "avif" => ImageFormat::Avif,
            "basis" => ImageFormat::Basis,
            "bmp" => ImageFormat::Bmp,
            "dds" => ImageFormat::Dds,
            "ff" | "farbfeld" => ImageFormat::Farbfeld,
            "gif" => ImageFormat::Gif,
            "exr" => ImageFormat::OpenExr,
            "hdr" => ImageFormat::Hdr,
            "ico" => ImageFormat::Ico,
            "jpg" | "jpeg" => ImageFormat::Jpeg,
            "ktx2" => ImageFormat::Ktx2,
            "pbm" | "pam" | "ppm" | "pgm" => ImageFormat::Pnm,
            "png" => ImageFormat::Png,
            "tga" => ImageFormat::Tga,
            "tif" | "tiff" => ImageFormat::Tiff,
            "webp" => ImageFormat::WebP,
            _ => return None,
        })
    }

    pub fn as_image_crate_format(&self) -> Option<image::ImageFormat> {
        Some(match self {
            ImageFormat::Avif => image::ImageFormat::Avif,
            ImageFormat::Bmp => image::ImageFormat::Bmp,
            ImageFormat::Dds => image::ImageFormat::Dds,
            ImageFormat::Farbfeld => image::ImageFormat::Farbfeld,
            ImageFormat::Gif => image::ImageFormat::Gif,
            ImageFormat::OpenExr => image::ImageFormat::OpenExr,
            ImageFormat::Hdr => image::ImageFormat::Hdr,
            ImageFormat::Ico => image::ImageFormat::Ico,
            ImageFormat::Jpeg => image::ImageFormat::Jpeg,
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::Pnm => image::ImageFormat::Pnm,
            ImageFormat::Tga => image::ImageFormat::Tga,
            ImageFormat::Tiff => image::ImageFormat::Tiff,
            ImageFormat::WebP => image::ImageFormat::WebP,
            ImageFormat::Basis | ImageFormat::Ktx2 => return None,
        })
    }

    pub fn from_image_crate_format(format: image::ImageFormat) -> Option<ImageFormat> {
        Some(match format {
            image::ImageFormat::Avif => ImageFormat::Avif,
            image::ImageFormat::Bmp => ImageFormat::Bmp,
            image::ImageFormat::Dds => ImageFormat::Dds,
            image::ImageFormat::Farbfeld => ImageFormat::Farbfeld,
            image::ImageFormat::Gif => ImageFormat::Gif,
            image::ImageFormat::OpenExr => ImageFormat::OpenExr,
            image::ImageFormat::Hdr => ImageFormat::Hdr,
            image::ImageFormat::Ico => ImageFormat::Ico,
            image::ImageFormat::Jpeg => ImageFormat::Jpeg,
            image::ImageFormat::Png => ImageFormat::Png,
            image::ImageFormat::Pnm => ImageFormat::Pnm,
            image::ImageFormat::Tga => ImageFormat::Tga,
            image::ImageFormat::Tiff => ImageFormat::Tiff,
            image::ImageFormat::WebP => ImageFormat::WebP,
            _ => return None,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub enum ImageSampler {
    #[default]
    Default,
    Descriptor(ImageSamplerDescriptor),
}

#[derive(Debug, Clone)]
pub struct ImageSamplerDescriptor {
    pub label: Option<String>,
    /// How to deal with out of bounds accesses in the u (i.e. x) direction.
    pub address_mode_u: ImageAddressMode,
    /// How to deal with out of bounds accesses in the v (i.e. y) direction.
    pub address_mode_v: ImageAddressMode,
    /// How to deal with out of bounds accesses in the w (i.e. z) direction.
    pub address_mode_w: ImageAddressMode,
    /// How to filter the texture when it needs to be magnified (made larger).
    pub mag_filter: ImageFilterMode,
    /// How to filter the texture when it needs to be minified (made smaller).
    pub min_filter: ImageFilterMode,
    /// How to filter between mip map levels
    pub mipmap_filter: ImageFilterMode,
    /// Minimum level of detail (i.e. mip level) to use.
    pub lod_min_clamp: f32,
    /// Maximum level of detail (i.e. mip level) to use.
    pub lod_max_clamp: f32,
    /// If this is enabled, this is a comparison sampler using the given comparison function.
    pub compare: Option<ImageCompareFunction>,
    /// Must be at least 1. If this is not 1, all filter modes must be linear.
    pub anisotropy_clamp: u16,
    /// Border color to use when `address_mode` is [`ImageAddressMode::ClampToBorder`].
    pub border_color: Option<ImageSamplerBorderColor>,
}

impl Default for ImageSamplerDescriptor {
    fn default() -> Self {
        Self {
            address_mode_u: Default::default(),
            address_mode_v: Default::default(),
            address_mode_w: Default::default(),
            mag_filter: Default::default(),
            min_filter: Default::default(),
            mipmap_filter: Default::default(),
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
            label: None,
        }
    }
}

/// Comparison function used for depth and stencil operations.
///
/// This type mirrors [`wgpu::CompareFunction`].
#[derive(Clone, Copy, Debug)]
pub enum ImageCompareFunction {
    /// Function never passes
    Never,
    /// Function passes if new value less than existing value
    Less,
    /// Function passes if new value is equal to existing value. When using
    /// this compare function, make sure to mark your Vertex Shader's `@builtin(position)`
    /// output as `@invariant` to prevent artifacting.
    Equal,
    /// Function passes if new value is less than or equal to existing value
    LessEqual,
    /// Function passes if new value is greater than existing value
    Greater,
    /// Function passes if new value is not equal to existing value. When using
    /// this compare function, make sure to mark your Vertex Shader's `@builtin(position)`
    /// output as `@invariant` to prevent artifacting.
    NotEqual,
    /// Function passes if new value is greater than or equal to existing value
    GreaterEqual,
    /// Function always passes
    Always,
}

/// Color variation to use when the sampler addressing mode is [`ImageAddressMode::ClampToBorder`].
///
/// This type mirrors [`wgpu::SamplerBorderColor`].
#[derive(Clone, Copy, Debug)]
pub enum ImageSamplerBorderColor {
    /// RGBA color `[0, 0, 0, 0]`.
    TransparentBlack,
    /// RGBA color `[0, 0, 0, 1]`.
    OpaqueBlack,
    /// RGBA color `[1, 1, 1, 1]`.
    OpaqueWhite,
    /// On the Metal wgpu backend, this is equivalent to [`Self::TransparentBlack`] for
    /// textures that have an alpha component, and equivalent to [`Self::OpaqueBlack`]
    /// for textures that do not have an alpha component. On other backends,
    /// this is equivalent to [`Self::TransparentBlack`]. Requires
    /// [`wgpu::Features::ADDRESS_MODE_CLAMP_TO_ZERO`]. Not supported on the web.
    Zero,
}

/// Texel mixing mode when sampling between texels.
///
/// This type mirrors [`wgpu::FilterMode`].
#[derive(Clone, Copy, Debug, Default)]
pub enum ImageFilterMode {
    /// Nearest neighbor sampling.
    ///
    /// This creates a pixelated effect when used as a mag filter.
    #[default]
    Nearest,
    /// Linear Interpolation.
    ///
    /// This makes textures smooth but blurry when used as a mag filter.
    Linear,
}

#[derive(Debug, Default, Clone)]
pub enum ImageAddressMode {
    /// Clamp the value to the edge of the texture.
    ///
    /// -0.25 -> 0.0
    /// 1.25  -> 1.0
    #[default]
    ClampToEdge,
    /// Repeat the texture in a tiling fashion.
    ///
    /// -0.25 -> 0.75
    /// 1.25 -> 0.25
    Repeat,
    /// Repeat the texture, mirroring it every repeat.
    ///
    /// -0.25 -> 0.25
    /// 1.25 -> 0.75
    MirrorRepeat,
    /// Clamp the value to the border of the texture
    /// Requires the wgpu feature [`wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER`].
    ///
    /// -0.25 -> border
    /// 1.25 -> border
    ClampToBorder,
}
