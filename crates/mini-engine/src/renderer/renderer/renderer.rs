use mini_window::window::ErasedWindow;
use wgpu::RenderPipeline;

use super::{RenderAdapter, RenderDevice, RenderInstance, RenderQueue};

use crate::renderer::{prelude::SurfaceData, surface_data::WindowSurfaceDatas};

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
    pub fn render(&mut self) {}

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
