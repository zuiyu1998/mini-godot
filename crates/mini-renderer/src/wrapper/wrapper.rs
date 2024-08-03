use std::fmt::Debug;

use mini_core::prelude::{Deref, DerefMut};

#[derive(Deref, DerefMut)]
pub struct WgpuWrapper<T>(T);

impl<T> WgpuWrapper<T> {
    pub fn new(value: T) -> Self {
        WgpuWrapper(value)
    }
}

impl<T: Debug> Debug for WgpuWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Clone> Clone for WgpuWrapper<T> {
    fn clone(&self) -> Self {
        WgpuWrapper(self.0.clone())
    }
}
