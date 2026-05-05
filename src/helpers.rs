use cpal::SampleRate;

pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + t * (end - start)
}

pub trait ToSamples {
    fn to_samples(&self, sample_rate: SampleRate) -> usize;
}

impl ToSamples for std::time::Duration {
    fn to_samples(&self, sample_rate: SampleRate) -> usize {
        (self.as_secs_f32() * sample_rate as f32) as usize
    }
}
