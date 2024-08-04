use super::object::ErasedObjectTrait;

pub trait NodeTrait: Clone {}

pub trait ErasedNodeTrait: ErasedObjectTrait {}

pub struct Node(Box<dyn ErasedNodeTrait>);
