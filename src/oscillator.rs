use core::f32;
use std::time::Duration;

use cpal::SampleRate;

use crate::{helpers::ToSamples, node::Source};

pub struct Oscillator {
    samples_per_cycle: f32,
}

impl Oscillator {
    pub fn new(frequency: Duration, sample_rate: SampleRate) -> Self {
        Self {
            samples_per_cycle: frequency.to_samples(sample_rate) as f32,
        }
    }
}

impl Source for Oscillator {
    // sample_rate = samples/second
    // frequency = cycles/second
    // val(t) = sin(pi * frequency * t)
    // t = sample_index / sample_rate
    // val(sample_index) = sin(pi * frequency * sample_index / sample_rate)
    fn output(&mut self, sample_num: usize) -> f32 {
        f32::sin(f32::consts::PI * (sample_num as f32 / self.samples_per_cycle))
    }
}
