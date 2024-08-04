mod resource_macros;
mod wrapper;

pub use wrapper::*;

pub use resource_macros::render_resource_wrapper;

pub trait MiniDefault {
    fn mini_default() -> Self;
}

impl MiniDefault for wgpu::TextureFormat {
    fn mini_default() -> Self {
        wgpu::TextureFormat::Rgba8UnormSrgb
    }
}
