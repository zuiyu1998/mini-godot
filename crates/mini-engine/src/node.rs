use mini_core::downcast::Downcast;

pub trait NodeTrait {}

pub trait ErasedNodeTrait: Downcast {}

pub struct Node(Box<dyn ErasedNodeTrait>);
