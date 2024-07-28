use mini_core::downcast::Downcast;

pub trait ObjectTrait: Downcast + Clone {}

pub trait ErasedObjectTrait: Downcast {}

pub struct Object(Box<dyn ErasedObjectTrait>);
