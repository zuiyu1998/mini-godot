pub mod type_uuid;
pub mod utils;

pub use bitflags;
pub use bytemuck;
pub use downcast_rs as downcast;
pub use futures_lite;
pub use parking_lot;
pub use thiserror;
pub use uuid;
pub mod node;
pub mod object;

pub mod prelude {
    pub use crate::node::*;
    pub use crate::object::*;
    pub use crate::type_uuid::*;
    pub use crate::utils::*;

    pub use mini_core_macros::{Deref, DerefMut};
}
