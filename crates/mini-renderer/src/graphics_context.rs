use std::sync::Arc;

use mini_core::{futures_lite, parking_lot::Mutex};
use mini_window::window::{ErasedWindow, WindowId};

use crate::{
    prelude::WindowSurfaceDatas,
    renderer::{RenderAdapter, RenderDevice, RenderInstance, RenderQueue, Renderer},
    wrapper::WgpuWrapper,
};

use wgpu::{DeviceDescriptor, Instance, MemoryHints, Surface, SurfaceTargetUnsafe};

pub struct InitializedGraphicsContext {
    window_surface_datas: WindowSurfaceDatas,
    renderer: Renderer,
}

pub struct RenderContext<'a> {
    pub renderer: &'a Renderer,
}

impl InitializedGraphicsContext {
    pub fn render(&mut self) {
        let render_context = RenderContext {
            renderer: &self.renderer,
        };

        for render_contex in self.window_surface_datas.values_mut() {
            render_contex.render(&render_context);
        }
    }

    pub fn add_render_pipeline(&mut self, window_id: &WindowId) {
        let surface_data = self.window_surface_datas.get(window_id).unwrap();

        let shader = self
            .renderer
            .device
            .wgpu_device()
            .create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout = self.renderer.device.wgpu_device().create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            },
        );

        let render_pipeline = self.renderer.device.wgpu_device().create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main", // 1.
                    buffers: &[],           // 2.
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    // 3.
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        // 4.
                        format: surface_data.configuration.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // 2.
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None, // 1.
                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview: None, // 5.
                cache: None,     // 6.
            },
        );

        self.renderer.render_pipeline = Some(render_pipeline);
    }
}

pub enum GraphicsContext {
    Initialized(InitializedGraphicsContext),

    Uninitialized,
}

type FutureRendererResources =
    Arc<Mutex<Option<(RenderDevice, RenderQueue, RenderInstance, RenderAdapter)>>>;

impl GraphicsContext {
    pub fn initialize_gpu_context(&mut self, window: &ErasedWindow) {
        let future_renderer_resources: FutureRendererResources = Arc::new(Mutex::new(None));

        let window_clone = window.raw_handle_wrapper_holder.clone();
        let future_renderer_resources_clone = future_renderer_resources.clone();

        let async_renderer = async move {
            let target = {
                let raw_handle = window_clone.0.lock();
                let (raw_display_handle, raw_window_handle) = (*raw_handle)
                    .as_ref()
                    .and_then(|raw_handle| {
                        Some((
                            raw_handle.display_handle.clone(),
                            raw_handle.window_handle.clone(),
                        ))
                    })
                    .unwrap();

                SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle,
                    raw_window_handle,
                }
            };

            let instance = Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                ..Default::default()
            });

            let surface = unsafe { instance.create_surface_unsafe(target).unwrap() };

            let options: wgpu::RequestAdapterOptionsBase<&Surface> = wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            };

            let adapter = instance.request_adapter(&options).await.unwrap();

            let (device, queue) = adapter
                .request_device(
                    &DeviceDescriptor {
                        required_features: wgpu::Features::empty(),

                        required_limits: wgpu::Limits::default(),
                        label: None,
                        memory_hints: MemoryHints::default(),
                    },
                    None, // Trace path
                )
                .await
                .unwrap();

            let device = RenderDevice::from(device);
            let queue = RenderQueue(Arc::new(WgpuWrapper::new(queue)));
            let instance = RenderInstance(Arc::new(WgpuWrapper::new(instance)));
            let adapter = RenderAdapter(Arc::new(WgpuWrapper::new(adapter)));

            let mut future_renderer_resources_inner = future_renderer_resources_clone.lock();
            *future_renderer_resources_inner = Some((device, queue, instance, adapter));
        };

        futures_lite::future::block_on(async_renderer);

        let (device, queue, instance, adapter) = future_renderer_resources.lock().take().unwrap();

        *self = GraphicsContext::Initialized(InitializedGraphicsContext {
            window_surface_datas: Default::default(),
            renderer: Renderer::new(device, queue, instance, adapter),
        })
    }

    pub fn initialize_windows(&mut self, window: &ErasedWindow) {
        if let GraphicsContext::Initialized(context) = self {
            context
                .window_surface_datas
                .initialize_window(&context.renderer, window)
        }
    }

    pub fn add_render_pipeline(&mut self, window_id: &WindowId) {
        if let GraphicsContext::Initialized(context) = self {
            context.add_render_pipeline(window_id);
        }
    }

    pub fn render(&mut self) {
        if let GraphicsContext::Initialized(context) = self {
            context.render();
        }
    }
}
