use std::time::Duration;

use cpal::SampleRate;

use crate::{graph::Node, helpers::ToSamples, param::Ramp};

pub struct Gain;

impl Node<2, 1> for Gain {
    const NODE_TYPE: crate::graph::NodeType = crate::graph::NodeType::Gain;
    const INPUT_NAMES: [&'static str; 2] = ["input", "gain"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 2], _: usize) -> [f32; 1] {
        [input[1] * input[0]]
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

    fn start(&mut self, sample_num: usize) {
        self.start_time = Some(sample_num);
        self.attack.start(sample_num);
        self.decay.start(sample_num + self.attack.duration());
    }

    fn stop(&mut self, sample_num: usize) {
        self.stop_time = Some(sample_num);
        self.release.start(sample_num);
        self.start_time = None;
    }
}

impl Node<1, 1> for ADSR {
    const NODE_TYPE: crate::graph::NodeType = crate::graph::NodeType::ADSR;
    const INPUT_NAMES: [&'static str; 1] = ["gate"];
    const OUTPUT_NAMES: [&'static str; 1] = ["envelope"];
    fn process(&mut self, input: [f32; 1], sample_num: usize) -> [f32; 1] {
        let gate = input[0];

        let is_on = gate > 0.5;
        if is_on && self.start_time.is_none() {
            self.start(sample_num);
        }
        if !is_on && self.start_time.is_some() && self.stop_time.is_none() {
            self.stop(sample_num);
        }

        [self.gain(sample_num)]
    }
}
