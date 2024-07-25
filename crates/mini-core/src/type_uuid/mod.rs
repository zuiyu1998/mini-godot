mod std_impl;

use uuid::Uuid;

//为类型实现唯一的标识
pub trait TypeUuidProvider: Sized {
    fn type_uuid() -> Uuid;
}

#[macro_export]
macro_rules! uuid_provider {
    ($type:ident $(<$($generics:tt),*>)? = $uuid:expr) => {
        impl$(<$($generics),*>)? $crate::type_uuid::TypeUuidProvider for $type $(<$($generics),*>)? {
            fn type_uuid() -> $crate::uuid::Uuid {
                $crate::uuid::uuid!($uuid)
            }
        }
    };
}

impl<T: TypeUuidProvider> TypeUuidProvider for Option<T> {
    fn type_uuid() -> Uuid {
        combine_uuids(
            uuid::uuid!("ffe06d3b-0d07-42cd-886b-5248f6ca7f7d"),
            T::type_uuid(),
        )
    }
}

impl<T: TypeUuidProvider> TypeUuidProvider for Vec<T> {
    fn type_uuid() -> Uuid {
        combine_uuids(
            uuid::uuid!("51bc577b-5a50-4a97-9b31-eda2f3d46c9c"),
            T::type_uuid(),
        )
    }
}

pub fn combine_uuids(a: Uuid, b: Uuid) -> Uuid {
    let mut combined_bytes = a.into_bytes();

    for (src, dest) in b.into_bytes().into_iter().zip(combined_bytes.iter_mut()) {
        *dest ^= src;
    }

    Uuid::from_bytes(combined_bytes)
}
