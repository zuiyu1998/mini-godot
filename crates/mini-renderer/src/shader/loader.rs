use mini_core::thiserror::{self, Error};
use mini_resource::prelude::{LoadContext, ResourceLoader};

use super::Shader;

#[derive(Default)]
pub struct ShaderLoader;

#[derive(Debug, Error)]
pub enum ShaderLoaderError {
    #[error("Could not load shader: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse shader: {0}")]
    Parse(#[from] std::string::FromUtf8Error),
}

impl ResourceLoader for ShaderLoader {
    type ResourceData = Shader;

    type Settings = ();

    type Error = ShaderLoaderError;

    fn extensions(&self) -> &[&str] {
        &["wgsl"]
    }

    async fn load<'a>(
        &'a self,
        reader: &'a mut dyn mini_resource::prelude::Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::ResourceData, Self::Error> {
        let ext = load_context.path().extension().unwrap().to_str().unwrap();
        let path = load_context.resource_path().to_string();
        // On windows, the path will inconsistently use \ or /.
        // TODO: remove this once AssetPath forces cross-platform "slash" consistency. See #10511
        let path = path.replace(std::path::MAIN_SEPARATOR, "/");
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let shader = match ext {
            "wgsl" => Shader::from_wgsl(String::from_utf8(bytes)?, path),
            _ => {
                unimplemented!()
            }
        };

        return Ok(shader);
    }
}
