use std::sync::Arc;

use mini_core::{
    type_uuid::TypeUuidProvider,
    uuid::{uuid, Uuid},
};
use mini_resource::{
    io::ResourceIo,
    loader::{ResourceData, ResourceLoader},
};

#[derive(Debug, TypeUuidProvider)]
#[type_uuid(id = "21613484-7145-4d1c-87d8-62fa767560ab")]
pub struct Image {
    pub data: Vec<u8>,
}

impl ResourceData for Image {}

pub struct PngLoader;

impl ResourceLoader for PngLoader {
    fn load(
        &self,
        path: std::path::PathBuf,
        io: Arc<dyn ResourceIo>,
    ) -> mini_resource::prelude::BoxedLoaderFuture {
        todo!()
    }

    fn extensions(&self) -> &[&str] {
        &[".png"]
    }

    fn data_type_uuid(&self) -> Uuid {
        Image::type_uuid()
    }
}
