use wgpu::RenderPipeline;

use super::{RenderAdapter, RenderDevice, RenderInstance, RenderQueue};

pub struct Renderer {
    pub render_pipeline: Option<RenderPipeline>,
    pub device: RenderDevice,
    pub queue: RenderQueue,
    pub instance: RenderInstance,
    pub adapter: RenderAdapter,
    //网格
}

impl Renderer {
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
        }
    }
}
