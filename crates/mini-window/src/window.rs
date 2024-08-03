use mini_math::UVec2;

use crate::prelude::{RawHandleWrapper, RawHandleWrapperHolder};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub struct WindowId(u64);

impl WindowId {
    pub fn new(id: u64) -> Self {
        WindowId(id)
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Window {
    pub resolution: WindowResolution,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct ErasedWindow {
    pub raw_handle_wrapper: RawHandleWrapper,
    pub raw_handle_wrapper_holder: RawHandleWrapperHolder,
    pub window: Window,
    pub id: WindowId,
}

impl Default for Window {
    fn default() -> Self {
        Window {
            resolution: Default::default(),
            title: "App".to_string(),
        }
    }
}

impl Window {
    pub fn physical_size(&self) -> UVec2 {
        self.resolution.physical_size()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLifecycle {
    /// The application is not started yet.
    Idle,
    /// The application is running.
    Running,
    /// The application is going to be suspended.
    /// Applications have one frame to react to this event before being paused in the background.
    WillSuspend,
    /// The application was suspended.
    Suspended,
    /// The application is going to be resumed.
    /// Applications have one extra frame to react to this event before being fully resumed.
    WillResume,
}
