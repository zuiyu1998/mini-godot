use crate::window_wrapper::RawHandleWrapperHolder;
use mini_math::UVec2;

#[derive(Debug)]
pub struct WindowResolution {
    /// Width of the window in physical pixels.
    physical_width: u32,
    /// Height of the window in physical pixels.
    physical_height: u32,
    /// Code-provided ratio of physical size to logical size.
    ///
    /// Should be used instead of `scale_factor` when set.
    scale_factor_override: Option<f32>,
    /// OS-provided ratio of physical size to logical size.
    ///
    /// Set automatically depending on the pixel density of the screen.
    scale_factor: f32,
}
impl WindowResolution {
    pub fn physical_size(&self) -> UVec2 {
        UVec2::new(self.physical_width, self.physical_height)
    }
}
impl Default for WindowResolution {
    fn default() -> Self {
        WindowResolution {
            physical_width: 1280,
            physical_height: 720,
            scale_factor_override: None,
            scale_factor: 1.0,
        }
    }
}

#[derive(Debug, Default)]
pub struct Window {
    pub resolution: WindowResolution,
}

impl Window {
    pub fn physical_size(&self) -> UVec2 {
        self.resolution.physical_size()
    }
}

#[derive(Debug, Default)]
pub struct PrimaryWindow {
    pub handle: RawHandleWrapperHolder,
    pub window: Window,
}
