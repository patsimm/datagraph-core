use core::f32;
use std::time::Duration;

use cpal::SampleRate;

use crate::{
    frequency::{Frequency, FromCv},
    graph::Node,
    helpers::ToSamples,
};

pub struct Oscillator {
    sample_rate: SampleRate,
}

impl Oscillator {
    pub fn new(sample_rate: SampleRate) -> Self {
        Self { sample_rate }
    }

    pub fn samples_per_cycle(&mut self, frequency: Frequency) -> f32 {
        let duration = Duration::from(frequency);
        duration.to_samples(self.sample_rate) as f32
    }
}

impl Node<1, 1> for Oscillator {
    // sample_rate = samples/second
    // frequency = cycles/second
    // val(t) = sin(pi * frequency * t)
    // t = sample_index / sample_rate
    // val(sample_index) = sin(pi * frequency * sample_index / sample_rate)

    fn process(&mut self, input: [f32; 1], sample_num: usize) -> [f32; 1] {
        let frequency = Frequency::from_cv(input[0]);
        [f32::sin(
            f32::consts::PI * (sample_num as f32 / self.samples_per_cycle(frequency)),
        )]
    }
}
