use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

use mini_window::window::{ErasedWindow, WindowId};
use wgpu::{
    Surface, SurfaceConfiguration, SurfaceTargetUnsafe, SurfaceTexture, TextureView,
    TextureViewDescriptor,
};

pub use crate::{
    renderer::{RenderAdapter, RenderDevice, RenderInstance},
    wrapper::WgpuWrapper,
};

pub struct SurfaceData {
    //画板
    pub surface: WgpuWrapper<Surface<'static>>,
    pub configuration: SurfaceConfiguration,

    pub swap_chain_texture_view: Option<TextureView>,

    pub swap_chain_texture: Option<SurfaceTexture>,
}

impl SurfaceData {
    pub fn set_swapchain_texture(&mut self) {
        let frame = self.surface.get_current_texture().unwrap();

        let texture_view_descriptor = TextureViewDescriptor {
            format: Some(frame.texture.format().add_srgb_suffix()),
            ..Default::default()
        };
        self.swap_chain_texture_view = Some(TextureView::from(
            frame.texture.create_view(&texture_view_descriptor),
        ));
        self.swap_chain_texture = Some(SurfaceTexture::from(frame));
    }

    pub fn present(&mut self) {
        let swap_chain_texture = self.swap_chain_texture.take().unwrap();
        swap_chain_texture.present();
    }

    pub fn initialize_surface_data(
        device: &RenderDevice,
        instance: &RenderInstance,
        adapter: &RenderAdapter,
        window: &ErasedWindow,
    ) -> Self {
        let size = window.window.physical_size();

        let surface_target = SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: window.raw_handle_wrapper.display_handle,
            raw_window_handle: window.raw_handle_wrapper.window_handle,
        };

        // SAFETY: The window handles in ExtractedWindows will always be valid objects to create surfaces on
        let surface = unsafe {
            // NOTE: On some OSes this MUST be called from the main thread.
            // As of wgpu 0.15, only fallible if the given window is a HTML canvas and obtaining a WebGPU or WebGL2 context fails.

            instance
                .create_surface_unsafe(surface_target)
                .expect("Failed to create wgpu surface")
        };
        let caps = surface.get_capabilities(&adapter);

        let surface_format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.x,
            height: size.y,
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device.wgpu_device(), &config);

        Self {
            surface: WgpuWrapper::new(surface),
            configuration: config,
            swap_chain_texture: None,
            swap_chain_texture_view: None,
        }
    }
}

#[derive(Default)]
pub struct WindowSurfaceDatas {
    surface_datas: HashMap<WindowId, SurfaceData>,
    initialized_windows: HashSet<WindowId>,
}

impl Deref for WindowSurfaceDatas {
    type Target = HashMap<WindowId, SurfaceData>;

    fn deref(&self) -> &Self::Target {
        &self.surface_datas
    }
}

impl DerefMut for WindowSurfaceDatas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.surface_datas
    }
}

impl WindowSurfaceDatas {
    pub fn initialize_window(&mut self, window: &ErasedWindow, surface_data: SurfaceData) {
        if self.initialized_windows.contains(&window.id) {
            return;
        }
        self.surface_datas.insert(window.id, surface_data);

        self.initialized_windows.insert(window.id);
    }
}
