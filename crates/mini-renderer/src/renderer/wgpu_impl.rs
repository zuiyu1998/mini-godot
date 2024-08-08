use crate::wrapper::WgpuWrapper;
use mini_core::prelude::{Deref, DerefMut};
use std::sync::Arc;
use wgpu::{Adapter, AdapterInfo, Instance, Queue};

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct RenderQueue(pub Arc<WgpuWrapper<Queue>>);

#[derive(Debug, Clone, Deref, DerefMut)]

pub struct RenderAdapter(pub Arc<WgpuWrapper<Adapter>>);

#[derive(Debug, Clone, Deref, DerefMut)]

pub struct RenderInstance(pub Arc<WgpuWrapper<Instance>>);

#[derive(Debug, Deref, DerefMut, Clone)]

pub struct RenderAdapterInfo(pub WgpuWrapper<AdapterInfo>);
