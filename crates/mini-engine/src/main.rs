use std::sync::Arc;

use mini_core::{futures_lite, parking_lot::Mutex};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use wgpu::{Device, DeviceDescriptor, Instance, MemoryHints, Queue, Surface, SurfaceTargetUnsafe};

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut state = WinitRunnerState::new();

    event_loop.run_app(&mut state).unwrap();
}

pub struct WinitRunnerState {
    graphics_context: GraphicsContext,
}

impl WinitRunnerState {
    pub fn new() -> Self {
        WinitRunnerState {
            graphics_context: GraphicsContext::Uninitialized,
        }
    }
}

impl WinitRunnerState {}

pub struct InitializedGraphicsContext {
    window: Arc<Mutex<Window>>,
    device: Device,
    queue: Queue,
    instance: Instance,
    surface: Surface<'static>,
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
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

pub enum GraphicsContext {
    Initialized(InitializedGraphicsContext),

    Uninitialized,
}

type FutureRendererResources = Arc<Mutex<Option<(Device, Queue, Instance, Surface<'static>)>>>;

impl GraphicsContext {
    pub fn initialize_graphics_context(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);

        let winit_window_attributes = Window::default_attributes();

        let window = event_loop.create_window(winit_window_attributes).unwrap();

        let window = Arc::new(Mutex::new(window));

        let future_renderer_resources: FutureRendererResources = Arc::new(Mutex::new(None));

        let window_clone = window.clone();
        let future_renderer_resources_clone = future_renderer_resources.clone();

        let async_renderer = async move {
            let instance = Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                ..Default::default()
            });

            let handle = window_clone.lock();
            let size = handle.inner_size();

            let surface = unsafe {
                let target = SurfaceTargetUnsafe::from_window(&*handle).unwrap();
                instance.create_surface_unsafe(target).unwrap()
            };

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
                width: size.width,
                height: size.height,
                present_mode: surface_caps.present_modes[0],
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            surface.configure(&device, &config);

            let mut future_renderer_resources_inner = future_renderer_resources_clone.lock();
            *future_renderer_resources_inner = Some((device, queue, instance, surface));
        };

        futures_lite::future::block_on(async_renderer);

        let (device, queue, instance, surface) = future_renderer_resources.lock().take().unwrap();

        *self = GraphicsContext::Initialized(InitializedGraphicsContext {
            window,
            device,
            queue,
            instance,
            surface,
        })
    }

    pub fn render(&mut self) {
        if let GraphicsContext::Initialized(context) = self {
            context.render();
        }
    }
}

impl ApplicationHandler for WinitRunnerState {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.graphics_context
            .initialize_graphics_context(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::RedrawRequested => self.graphics_context.render(),
            _ => {}
        }
    }
}
