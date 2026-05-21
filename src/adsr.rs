use std::time::Duration;

use cpal::SampleRate;

use crate::{graph::Node, helpers::ToSamples, ramp::Ramp};

#[derive(Clone, Copy)]
enum ADSRState {
    Idle,
    Attack(Ramp),
    Decay(Ramp),
    Sustain,
    Release(Ramp),
}

fn step_state(state: &mut ADSRState, sustain: f32, decay_dur: usize) -> (f32, ADSRState) {
    match *state {
        ADSRState::Idle => (0.0, ADSRState::Idle),
        ADSRState::Attack(mut ramp) => {
            let v = ramp.update().unwrap_or(1.0);
            if ramp.is_active() {
                (v, ADSRState::Attack(ramp))
            } else {
                let mut decay = Ramp::new(1.0, sustain, decay_dur);
                decay.start();
                (v, ADSRState::Decay(decay))
            }
        }
        ADSRState::Decay(mut ramp) => {
            let v = ramp.update().unwrap_or(sustain);
            if ramp.is_active() {
                (v, ADSRState::Decay(ramp))
            } else {
                (v, ADSRState::Sustain)
            }
        }
        ADSRState::Sustain => (sustain, ADSRState::Sustain),
        ADSRState::Release(mut ramp) => {
            let v = ramp.update().unwrap_or(0.0);
            if ramp.is_active() {
                (v, ADSRState::Release(ramp))
            } else {
                (v, ADSRState::Idle)
            }
        }
    }
}

pub struct ADSR {
    attack_dur: usize,
    decay_dur: usize,
    sustain: f32,
    release_dur: usize,
    state: ADSRState,
    prev_gate: bool,
    last_value: f32,
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
            attack_dur: attack.to_samples(sample_rate),
            decay_dur: decay.to_samples(sample_rate),
            sustain,
            release_dur: release.to_samples(sample_rate),
            state: ADSRState::Idle,
            prev_gate: false,
            last_value: 0.0,
        }
    }

    fn process_sample(&mut self, gate: bool) -> f32 {
        let gate_on = gate && !self.prev_gate;
        let gate_off = !gate && self.prev_gate;
        self.prev_gate = gate;

        if gate_on {
            let mut ramp = Ramp::new(self.last_value, 1.0, self.attack_dur);
            ramp.start();
            self.state = ADSRState::Attack(ramp);
        } else if gate_off && !matches!(self.state, ADSRState::Idle) {
            let mut ramp = Ramp::new(self.last_value, 0.0, self.release_dur);
            ramp.start();
            self.state = ADSRState::Release(ramp);
        }

        let (value, next_state) = step_state(&mut self.state, self.sustain, self.decay_dur);
        self.state = next_state;
        self.last_value = value;
        value
    }
}

impl Node<1, 1> for ADSR {
    const INPUT_NAMES: [&'static str; 1] = ["gate"];
    const OUTPUT_NAMES: [&'static str; 1] = ["envelope"];
    fn process(&mut self, input: [f32; 1], _: usize) -> [f32; 1] {
        [self.process_sample(input[0] > 0.5)]
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_adsr() {
        use super::*;
        let mut adsr = ADSR::new(
            16,
            Duration::from_secs_f32(0.25),
            Duration::from_secs_f32(0.25),
            0.5,
            Duration::from_secs_f32(0.25),
        );
        let mut out = adsr.process([0f32], 0);
        assert_eq!(out, [0.0]);

        // attack
        out = adsr.process([1f32], 0);
        assert_eq!(out, [0.0]);
        out = adsr.process([1f32], 0);
        assert_eq!(out, [0.25]);
        out = adsr.process([1f32], 0);
        assert_eq!(out, [0.5]);
        out = adsr.process([1f32], 0);
        assert_eq!(out, [0.75]);
        out = adsr.process([1f32], 0);
        assert_eq!(out, [1.0]);

        // decay
        out = adsr.process([1f32], 0);
        assert_eq!(out, [1.0]);
        out = adsr.process([1f32], 0);
        assert_eq!(out, [0.875]);
        out = adsr.process([1f32], 0);
        assert_eq!(out, [0.75]);
        out = adsr.process([1f32], 0);
        assert_eq!(out, [0.625]);
        out = adsr.process([1f32], 0);
        assert_eq!(out, [0.5]);

        // sustain
        for _ in 0..10 {
            out = adsr.process([1f32], 0);
            assert_eq!(out, [0.5]);
        }

        // release
        out = adsr.process([0f32], 0);
        assert_eq!(out, [0.5]);
        out = adsr.process([0f32], 0);
        assert_eq!(out, [0.375]);
        out = adsr.process([0f32], 0);
        assert_eq!(out, [0.25]);
        out = adsr.process([0f32], 0);
        assert_eq!(out, [0.125]);
        out = adsr.process([0f32], 0);
        assert_eq!(out, [0.0]);

        // idle
        out = adsr.process([0f32], 0);
        assert_eq!(out, [0.0]);
    }
}
