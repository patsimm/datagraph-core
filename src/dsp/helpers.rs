use std::sync::atomic::{AtomicU32, Ordering};

pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + t * (end - start)
}

pub trait ToSamples {
    fn to_samples(&self, sample_rate: u32) -> usize;
}

impl ToSamples for std::time::Duration {
    fn to_samples(&self, sample_rate: u32) -> usize {
        (self.as_secs_f32() * sample_rate as f32) as usize
    }
}

pub struct AtomicF32(AtomicU32);

impl AtomicF32 {
    pub fn new(val: f32) -> Self {
        Self(AtomicU32::new(val.to_bits()))
    }
    pub fn load(&self, order: Ordering) -> f32 {
        f32::from_bits(self.0.load(order))
    }
    pub fn store(&self, val: f32, order: Ordering) {
        self.0.store(val.to_bits(), order)
    }
}
