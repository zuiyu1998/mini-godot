use super::TemporaryCache;

pub struct ShaderSet {}

#[derive(Default)]
pub struct ShaderCache {
    pub(super) cache: TemporaryCache<ShaderSet>,
}

impl ShaderCache {
    pub fn update(&mut self, dt: f32) {
        self.cache.update(dt)
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}
