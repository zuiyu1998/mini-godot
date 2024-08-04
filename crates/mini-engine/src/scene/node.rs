use super::object::{ErasedObjectTrait, ObjectTrait};

pub trait NodeTrait: Clone {}

impl<T: NodeTrait + ObjectTrait> ErasedNodeTrait for T {}

pub trait ErasedNodeTrait: ErasedObjectTrait {}

pub struct Node(Box<dyn ErasedNodeTrait>);
