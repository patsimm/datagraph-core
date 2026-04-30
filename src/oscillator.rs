use core::f32;

use cpal::SampleRate;

use crate::node::Source;

pub struct Oscillator {
    current_sample: usize,
    sample_rate: SampleRate,
    frequency: f32,
}

impl Oscillator {
    pub fn new(frequency: f32, sample_rate: SampleRate) -> Self {
        Self {
            frequency,
            sample_rate,
            current_sample: 0,
        }
    }
}

impl Source for Oscillator {
    // sample_rate = samples/second
    // frequency = cycles/second
    // val(t) = sin(pi * frequency * t)
    // t = sample_index / sample_rate
    // val(sample_index) = sin(pi * frequency * sample_index / sample_rate)
    fn output(&mut self) -> f32 {
        self.current_sample += 1;
        f32::sin(
            f32::consts::PI * self.frequency * self.current_sample as f32 / self.sample_rate as f32,
        )
    }
}
