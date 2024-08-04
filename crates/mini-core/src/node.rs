use crate::downcast::Downcast;

pub trait NodeTrait {}

pub trait ErasedNodeTrait: Downcast {}

pub struct Node(Box<dyn ErasedNodeTrait>);
