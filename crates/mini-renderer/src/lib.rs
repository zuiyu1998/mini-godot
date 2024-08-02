use std::sync::Arc;

use mini_core::{futures_lite, parking_lot::Mutex};
use mini_window::window::ErasedWindow;
use wgpu::{Device, DeviceDescriptor, Instance, MemoryHints, Queue, Surface, SurfaceTargetUnsafe};

pub struct InitializedGraphicsContext {
    device: Device,
    queue: Queue,
    instance: Instance,
    surface: Surface<'static>,
    render_pipeline: wgpu::RenderPipeline,
}

impl InitializedGraphicsContext {
    pub fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
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

            render_pass.set_pipeline(&self.render_pipeline); // 2.
            render_pass.draw(0..3, 0..1); // 3.
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

pub enum GraphicsContext {
    Initialized(InitializedGraphicsContext),

    Uninitialized,
}

type FutureRendererResources = Arc<
    Mutex<
        Option<(
            Device,
            Queue,
            Instance,
            Surface<'static>,
            wgpu::RenderPipeline,
        )>,
    >,
>;

impl GraphicsContext {
    pub fn initialize_graphics_context(&mut self, window: &ErasedWindow) {
        let size = window.window.physical_size();

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

            let surface_caps = surface.get_capabilities(&adapter);

            let surface_format = surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.x,
                height: size.y,
                present_mode: surface_caps.present_modes[0],
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            surface.configure(&device, &config);

            let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                        format: config.format,
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
            });

            let mut future_renderer_resources_inner = future_renderer_resources_clone.lock();
            *future_renderer_resources_inner =
                Some((device, queue, instance, surface, render_pipeline));
        };

        futures_lite::future::block_on(async_renderer);

        let (device, queue, instance, surface, render_pipeline) =
            future_renderer_resources.lock().take().unwrap();

        *self = GraphicsContext::Initialized(InitializedGraphicsContext {
            device,
            queue,
            instance,
            surface,
            render_pipeline,
        })
    }

    pub fn render(&mut self) {
        if let GraphicsContext::Initialized(context) = self {
            context.render();
        }
    }
}
