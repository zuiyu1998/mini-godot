use std::sync::Arc;

use mini_core::{futures_lite, parking_lot::Mutex};
use mini_resource::prelude::ResourceManager;
use mini_window::window::ErasedWindow;

use crate::{
    renderer::{RenderAdapter, RenderDevice, RenderInstance, RenderQueue, Renderer},
    wrapper::WgpuWrapper,
};

use wgpu::{DeviceDescriptor, Instance, MemoryHints, Surface, SurfaceTargetUnsafe};

pub struct InitializedGraphicsContext {
    renderer: Renderer,
}

impl InitializedGraphicsContext {
    pub fn render(&mut self) {
        self.renderer.render()
    }
}

pub enum GraphicsContext {
    Initialized(InitializedGraphicsContext),
    Uninitialized,
}

type FutureRendererResources =
    Arc<Mutex<Option<(RenderDevice, RenderQueue, RenderInstance, RenderAdapter)>>>;

impl GraphicsContext {
    pub fn initialize(&mut self, window: &ErasedWindow, resource_manager: &ResourceManager) {
        self.initialize_graphics_context(window);
        self.build_resource_manager(resource_manager);
    }

    pub fn build_resource_manager(&mut self, _resource_manager: &ResourceManager) {}

    fn initialize_graphics_context(&mut self, window: &ErasedWindow) {
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
                backends: wgpu::Backends::VULKAN,
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
            renderer: Renderer::new(device, queue, instance, adapter),
        })
    }

    pub fn initialize_window(&mut self, window: &ErasedWindow) {
        if let GraphicsContext::Initialized(context) = self {
            context.renderer.initialize_window(window)
        }
    }

    pub fn render(&mut self) {
        if let GraphicsContext::Initialized(context) = self {
            context.render();
        }
    }
}
