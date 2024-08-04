use mini_core::downcast::Downcast;

pub trait ObjectTrait: Downcast + Clone {}

impl<T: ObjectTrait> ErasedObjectTrait for T {}

pub trait ErasedObjectTrait: Downcast {}

pub struct Object(Box<dyn ErasedObjectTrait>);
