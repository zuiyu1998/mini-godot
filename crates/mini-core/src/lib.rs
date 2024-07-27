pub mod type_uuid;
pub mod utils;

pub use parking_lot;
pub use uuid;

pub mod prelude {
    pub use crate::type_uuid::*;
    pub use crate::utils::*;
}
