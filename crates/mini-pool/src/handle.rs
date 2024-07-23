use std::marker::PhantomData;

///索引
pub struct Handle<T> {
    pub(crate) index: u32,

    pub(super) generation: u32,

    pub(crate) type_marker: PhantomData<T>,
}
