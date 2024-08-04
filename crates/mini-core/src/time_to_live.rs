use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, PartialEq)]
pub struct TimeToLive(pub f32);

impl Default for TimeToLive {
    fn default() -> Self {
        Self(TimeToLive::DEFAULT_LIFETIME)
    }
}

impl TimeToLive {
    pub const DEFAULT_LIFETIME: f32 = 60.0;
}

impl Deref for TimeToLive {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TimeToLive {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
