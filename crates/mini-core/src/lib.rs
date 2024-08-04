pub mod sparse;
pub mod time_to_live;
pub mod type_uuid;
pub mod utils;

pub use bitflags;
pub use bytemuck;
pub use downcast_rs as downcast;
pub use futures_lite;
pub use parking_lot;
pub use thiserror;
pub use uuid;

pub mod prelude {
    pub use crate::sparse::*;
    pub use crate::time_to_live::*;
    pub use crate::type_uuid::*;
    pub use crate::utils::*;

    pub use mini_core_macros::{Deref, DerefMut, EnumVariantMeta};
}
