use std::{ops::Deref, time::Duration};

pub struct Frequency(f32);

impl Deref for Frequency {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Frequency> for Duration {
    fn from(freq: Frequency) -> Self {
        Duration::from_secs_f32(1.0 / freq.0)
    }
}

impl From<Duration> for Frequency {
    fn from(duration: Duration) -> Self {
        Self(1.0 / duration.as_secs_f32())
    }
}

pub trait FromHz {
    fn from_hz(hz: impl Into<f32>) -> Self;
}

impl FromHz for std::time::Duration {
    fn from_hz(hz: impl Into<f32>) -> Self {
        std::time::Duration::from_secs_f32(1.0 / hz.into())
    }
}
impl FromHz for Frequency {
    fn from_hz(freq: impl Into<f32>) -> Self {
        Self(freq.into())
    }
}
