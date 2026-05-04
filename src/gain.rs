use std::time::Duration;

use cpal::SampleRate;

use crate::{
    helpers::{ToSamples, lerp},
    node::Effect,
};

pub struct Ramp {
    target: f32,
    start: f32,
    duration: usize,
    start_sample: Option<usize>,
}

impl Ramp {
    pub fn new(start: f32, target: f32, duration: usize) -> Self {
        Self {
            target,
            start, // This should be set to the current gain when ramp starts
            duration,
            start_sample: None,
        }
    }

    pub fn is_active(&self, sample_num: usize) -> bool {
        if let Some(start_sample) = self.start_sample {
            return start_sample <= sample_num;
        }
        false
    }

    pub fn start(&mut self, at: usize) {
        self.start_sample = Some(at);
    }

    pub fn update(&mut self, now: usize) -> Option<f32> {
        let Some(start_time) = self.start_sample else {
            return None; // Ramp hasn't started yet
        };
        if start_time >= now {
            return None;
        }
        let delta_samples = now - start_time;
        if delta_samples >= self.duration {
            self.start_sample = None;
            return Some(self.target);
        }
        Some(lerp(
            self.start,
            self.target,
            delta_samples as f32 / self.duration as f32,
        ))
    }
}

pub struct Gain {
    gain: f32,
}

impl Gain {
    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }
}

impl Default for Gain {
    fn default() -> Self {
        Self { gain: 1.0 }
    }
}

impl Effect for Gain {
    fn process(&mut self, input: f32, _: usize) -> f32 {
        self.gain * input
    }
}

pub struct ADSR {
    attack: Ramp,
    decay: Ramp,
    sustain: f32,
    release: Ramp,
    start_time: Option<usize>,
    stop_time: Option<usize>,
}

impl ADSR {
    pub fn new(
        sample_rate: SampleRate,
        attack: Duration,
        decay: Duration,
        sustain: f32,
        release: Duration,
    ) -> Self {
        Self {
            attack: Ramp::new(0.0, 1.0, attack.to_samples(sample_rate)),
            decay: Ramp::new(1.0, sustain, decay.to_samples(sample_rate)),
            sustain,
            release: Ramp::new(sustain, 0.0, release.to_samples(sample_rate)),
            start_time: None,
            stop_time: None,
        }
    }

    pub fn update(&mut self, sample_num: usize) -> Option<f32> {
        if let Some(start_time) = self.start_time {
            if start_time > sample_num {
                return None; // Note hasn't started yet
            }
            if self.attack.is_active(sample_num) {
                return self.attack.update(sample_num);
            }
            if self.decay.is_active(sample_num) {
                return self.decay.update(sample_num);
            }
            return Some(self.sustain);
        }
        if let Some(stop_time) = self.stop_time {
            if stop_time > sample_num {
                return None; // Note hasn't stopped yet
            }
            if self.release.is_active(sample_num) {
                return self.release.update(sample_num);
            }
            self.stop_time = None;
        }
        None
    }

    pub fn start(&mut self, sample_num: usize) {
        self.start_time = Some(sample_num);
        self.attack.start(sample_num);
        self.decay.start(sample_num + self.attack.duration);
    }

    pub fn stop(&mut self, sample_num: usize) {
        self.start_time = None;
        self.stop_time = Some(sample_num);
        self.release.start(sample_num);
    }
}
