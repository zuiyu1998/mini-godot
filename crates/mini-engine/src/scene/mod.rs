pub mod material;
pub mod node;
pub mod object;

pub struct Scene {}

pub mod prelude {
    pub use super::material::*;
    pub use super::node::*;
    pub use super::object::*;
}
