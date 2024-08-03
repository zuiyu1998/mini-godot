use super::{render_resource_wrapper, WgpuWrapper};
use mini_core::prelude::{Deref, DerefMut};
use std::sync::Arc;
use wgpu::{Adapter, AdapterInfo, Instance, Queue};

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct RenderQueue(pub Arc<WgpuWrapper<Queue>>);

#[derive(Debug, Clone, Deref, DerefMut)]

pub struct RenderAdapter(pub Arc<WgpuWrapper<Adapter>>);

#[derive(Debug, Clone, Deref, DerefMut)]

pub struct RenderInstance(pub Arc<WgpuWrapper<Instance>>);

#[derive(Debug, Deref, DerefMut, Clone)]

pub struct RenderAdapterInfo(pub WgpuWrapper<AdapterInfo>);

render_resource_wrapper!(ErasedRenderDevice, wgpu::Device);

/// This GPU device is responsible for the creation of most rendering and compute resources.
#[derive(Clone)]
pub struct RenderDevice {
    device: WgpuWrapper<ErasedRenderDevice>,
}

impl From<wgpu::Device> for RenderDevice {
    fn from(device: wgpu::Device) -> Self {
        Self {
            device: WgpuWrapper::new(ErasedRenderDevice::new(device)),
        }
    }
}

impl RenderDevice {
    pub fn wgpu_device(&self) -> &wgpu::Device {
        &self.device
    }
}
