use mini_core::{
    prelude::TypeUuidProvider,
    uuid::{uuid, Uuid},
};
use mini_resource::prelude::ResourceData;

///图片资源
#[derive(TypeUuidProvider, ResourceData, Debug)]
#[type_uuid(id = "5fb10a22-4ea9-4a13-a58c-82f2734aefd8")]
pub struct Image {
    //数据
    pub data: Vec<u8>,

    // TODO: this nesting makes accessing Image metadata verbose. Either flatten out descriptor or add accessors
    pub texture_descriptor: wgpu::TextureDescriptor<'static>,

    //图形的采样信息
    pub sampler: ImageSampler,
    pub texture_view_descriptor: Option<wgpu::TextureViewDescriptor<'static>>,
}

#[derive(Debug)]
pub enum ImageSampler {}
