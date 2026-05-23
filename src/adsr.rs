use std::time::Duration;

use cpal::SampleRate;

use crate::{
    graph::Node,
    helpers::ToSamples,
    ramp::Ramp,
    state_machine::{State, StateMachine},
};

struct ADSRContext {
    gate_on: bool,
    attack_dur: usize,
    decay_dur: usize,
    sustain: f32,
    release_dur: usize,
}

#[derive(Clone, Copy, Default)]
enum ADSRState {
    #[default]
    Idle,
    Attack(Ramp),
    Decay(Ramp),
    Sustain(f32),
    Release(Ramp),
}

impl ADSRState {
    fn value(&self) -> f32 {
        match self {
            ADSRState::Idle => 0.0,
            ADSRState::Attack(ramp) => ramp.value(),
            ADSRState::Decay(ramp) => ramp.value(),
            ADSRState::Sustain(sustain) => *sustain,
            ADSRState::Release(ramp) => ramp.value(),
        }
    }
}

impl State<ADSRContext> for ADSRState {
    fn tick(&mut self, context: ADSRContext) -> Self {
        match *self {
            ADSRState::Idle => {
                if context.gate_on {
                    let attack = Ramp::new(0.0, 1.0, context.attack_dur);
                    ADSRState::Attack(attack)
                } else {
                    *self
                }
            }
            ADSRState::Attack(mut ramp) => {
                if !context.gate_on {
                    let release = Ramp::new(ramp.value(), 0.0, context.release_dur);
                    return ADSRState::Release(release);
                };
                if ramp.tick() {
                    ADSRState::Attack(ramp)
                } else {
                    let decay = Ramp::new(1.0, context.sustain, context.decay_dur);
                    ADSRState::Decay(decay)
                }
            }
            ADSRState::Decay(mut ramp) => {
                if !context.gate_on {
                    let release = Ramp::new(ramp.value(), 0.0, context.release_dur);
                    return ADSRState::Release(release);
                }
                if ramp.tick() {
                    ADSRState::Decay(ramp)
                } else {
                    ADSRState::Sustain(context.sustain)
                }
            }
            ADSRState::Sustain(_) => {
                if !context.gate_on {
                    let release = Ramp::new(context.sustain, 0.0, context.release_dur);
                    return ADSRState::Release(release);
                }
                ADSRState::Sustain(context.sustain)
            }
            ADSRState::Release(mut ramp) => {
                let current_value = ramp.value();
                if context.gate_on {
                    let attack = Ramp::new(current_value, 1.0, context.decay_dur);
                    return ADSRState::Attack(attack);
                }
                if ramp.tick() {
                    ADSRState::Release(ramp)
                } else {
                    ADSRState::Idle
                }
            }
        }
    }
}

pub struct ADSR {
    attack_dur: usize,
    decay_dur: usize,
    sustain: f32,
    release_dur: usize,
    state_machine: StateMachine<ADSRContext, ADSRState>,
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
            state_machine: Default::default(),
        }
    }

    fn process_sample(&mut self, gate: bool) -> f32 {
        self.state_machine
            .step(ADSRContext {
                gate_on: gate,
                attack_dur: self.attack_dur,
                decay_dur: self.decay_dur,
                sustain: self.sustain,
                release_dur: self.release_dur,
            })
            .value()
    }
}

impl Node<1, 1> for ADSR {
    const INPUT_NAMES: [&'static str; 1] = ["gate"];
    const OUTPUT_NAMES: [&'static str; 1] = ["envelope"];
    fn process(&mut self, input: [f32; 1]) -> [f32; 1] {
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
        let mut out = adsr.process([0f32]);
        assert_eq!(out, [0.0]);

        // attack
        out = adsr.process([1f32]);
        assert_eq!(out, [0.0]);
        out = adsr.process([1f32]);
        assert_eq!(out, [0.25]);
        out = adsr.process([1f32]);
        assert_eq!(out, [0.5]);
        out = adsr.process([1f32]);
        assert_eq!(out, [0.75]);
        out = adsr.process([1f32]);
        assert_eq!(out, [1.0]);

        // decay
        out = adsr.process([1f32]);
        assert_eq!(out, [0.875]);
        out = adsr.process([1f32]);
        assert_eq!(out, [0.75]);
        out = adsr.process([1f32]);
        assert_eq!(out, [0.625]);
        out = adsr.process([1f32]);
        assert_eq!(out, [0.5]);

        // sustain
        for _ in 0..10 {
            out = adsr.process([1f32]);
            assert_eq!(out, [0.5]);
        }

        // release
        out = adsr.process([0f32]);
        assert_eq!(out, [0.5]);
        out = adsr.process([0f32]);
        assert_eq!(out, [0.375]);
        out = adsr.process([0f32]);
        assert_eq!(out, [0.25]);
        out = adsr.process([0f32]);
        assert_eq!(out, [0.125]);
        out = adsr.process([0f32]);
        assert_eq!(out, [0.0]);

        // idle
        out = adsr.process([0f32]);
        assert_eq!(out, [0.0]);
    }
}
