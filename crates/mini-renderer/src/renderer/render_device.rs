use crate::wrapper::{render_resource_wrapper, WgpuWrapper};

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
