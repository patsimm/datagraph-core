use core::f32;
use std::time::Duration;

use cpal::SampleRate;

use crate::{
    frequency::{Frequency, FromHz},
    helpers::ToSamples,
    node::Source,
    param::Param,
};

pub struct Oscillator {
    sample_rate: SampleRate,
    pub frequency: Param,
}

impl Oscillator {
    pub fn new(frequency: Frequency, sample_rate: SampleRate) -> Self {
        Self {
            sample_rate,
            frequency: Param::from(*frequency),
        }
    }

    pub fn samples_per_cycle(&mut self, sample_num: usize) -> f32 {
        let duration = Duration::from_hz(self.frequency.get(sample_num));
        duration.to_samples(self.sample_rate) as f32
    }
}

impl Source for Oscillator {
    // sample_rate = samples/second
    // frequency = cycles/second
    // val(t) = sin(pi * frequency * t)
    // t = sample_index / sample_rate
    // val(sample_index) = sin(pi * frequency * sample_index / sample_rate)
    fn output(&mut self, sample_num: usize) -> f32 {
        f32::sin(f32::consts::PI * (sample_num as f32 / self.samples_per_cycle(sample_num)))
    }
}
