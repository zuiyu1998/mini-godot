use mini_window::window::ErasedWindow;
use wgpu::RenderPipeline;

use super::{RenderAdapter, RenderDevice, RenderInstance, RenderQueue};

use crate::surface_data::{SurfaceData, WindowSurfaceDatas};

pub struct Renderer {
    pub render_pipeline: Option<RenderPipeline>,
    pub device: RenderDevice,
    pub queue: RenderQueue,
    pub instance: RenderInstance,
    pub adapter: RenderAdapter,
    pub window_surface_datas: WindowSurfaceDatas,
    //网格
}

impl Renderer {
    pub fn render(&mut self) {
        for surface_data in self.window_surface_datas.values_mut() {
            surface_data.set_swapchain_texture();
        }

        for surface_data in self.window_surface_datas.values_mut() {
            surface_data.present();
        }
    }

    pub fn initialize_window(&mut self, window: &ErasedWindow) {
        let surface_data = SurfaceData::initialize_surface_data(
            &self.device,
            &self.instance,
            &self.adapter,
            window,
        );

        self.window_surface_datas
            .initialize_window(window, surface_data);
    }

    pub fn new(
        device: RenderDevice,
        queue: RenderQueue,
        instance: RenderInstance,
        adapter: RenderAdapter,
    ) -> Self {
        Renderer {
            device,
            render_pipeline: None,
            queue,
            instance,
            adapter,
            window_surface_datas: Default::default(),
        }
    }
}
