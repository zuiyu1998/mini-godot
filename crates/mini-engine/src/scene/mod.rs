pub mod material;
pub mod mesh;
pub mod node;
pub mod object;

use mini_renderer::prelude::Shader;
use mini_resource::prelude::Resource;

pub struct Scene {
    pub shader: Option<Resource<Shader>>,
}

pub mod prelude {
    pub use super::material::*;
    pub use super::node::*;
    pub use super::object::*;
}
