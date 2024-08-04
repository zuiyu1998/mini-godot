use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

use mini_window::window::{ErasedWindow, WindowId};
use wgpu::{Surface, SurfaceConfiguration, SurfaceTargetUnsafe};

use crate::{renderer::Renderer, wrapper::WgpuWrapper};

pub struct SurfaceData {
    //画板
    pub surface: WgpuWrapper<Surface<'static>>,
    pub configuration: SurfaceConfiguration,
}

impl SurfaceData {
    pub fn render(&mut self, renderer: &Renderer) {
        let output = self.surface.get_current_texture().unwrap();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            renderer
                .device
                .wgpu_device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&renderer.render_pipeline.as_ref().unwrap()); // 2.
            render_pass.draw(0..3, 0..1); // 3.
        }

        renderer.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn initialize_surface_data(renderer: &Renderer, window: &ErasedWindow) -> Self {
        let size = window.window.physical_size();

        let surface_target = SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: window.raw_handle_wrapper.display_handle,
            raw_window_handle: window.raw_handle_wrapper.window_handle,
        };

        // SAFETY: The window handles in ExtractedWindows will always be valid objects to create surfaces on
        let surface = unsafe {
            // NOTE: On some OSes this MUST be called from the main thread.
            // As of wgpu 0.15, only fallible if the given window is a HTML canvas and obtaining a WebGPU or WebGL2 context fails.
            renderer
                .instance
                .create_surface_unsafe(surface_target)
                .expect("Failed to create wgpu surface")
        };
        let caps = surface.get_capabilities(&renderer.adapter);

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

        surface.configure(&renderer.device.wgpu_device(), &config);

        Self {
            surface: WgpuWrapper::new(surface),
            configuration: config,
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
    pub fn initialize_window(&mut self, renderer: &Renderer, window: &ErasedWindow) {
        if self.initialized_windows.contains(&window.id) {
            return;
        }

        let surface_data = SurfaceData::initialize_surface_data(renderer, window);

        self.surface_datas.insert(window.id, surface_data);

        self.initialized_windows.insert(window.id);
    }
}
