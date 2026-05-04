use std::time::Duration;

use cpal::SampleRate;

use crate::{
    helpers::ToSamples,
    node::Effect,
    param::{Param, Ramp},
};

pub struct Gain {
    pub param: Param,
}

impl Effect for Gain {
    fn process(&mut self, input: f32, sample_num: usize) -> f32 {
        self.param.get(sample_num) * input
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

    pub fn gain(&mut self, sample_num: usize) -> f32 {
        if let Some(start_time) = self.start_time {
            if start_time > sample_num {
                return 0.0; // Note hasn't started yet
            }
            if self.attack.is_active(sample_num) {
                return self.attack.update(sample_num).unwrap_or_default();
            }
            if self.decay.is_active(sample_num) {
                return self.decay.update(sample_num).unwrap_or_default();
            }
            return self.sustain;
        }
        if let Some(stop_time) = self.stop_time {
            if stop_time >= sample_num {
                return self.sustain; // Note hasn't stopped yet
            }
            if self.release.is_active(sample_num) {
                return self.release.update(sample_num).unwrap_or_default();
            }
            self.stop_time = None;
        }
        0.0
    }

    pub fn start(&mut self, sample_num: usize) {
        self.start_time = Some(sample_num);
        self.attack.start(sample_num);
        self.decay.start(sample_num + self.attack.duration());
    }

    pub fn stop(&mut self, sample_num: usize) {
        self.stop_time = Some(sample_num);
        self.release.start(sample_num);
        self.start_time = None;
    }
}

impl Effect for ADSR {
    fn process(&mut self, input: f32, sample_num: usize) -> f32 {
        input * self.gain(sample_num)
    }
}
