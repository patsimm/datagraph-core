use std::time::Duration;

use crate::{
    dsp::{Ramp, State, StateMachine, ToSamples},
    graph::Node,
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
    fn next_state(&mut self, context: ADSRContext) -> Self {
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
    sample_rate: u32,
    state_machine: StateMachine<ADSRContext, ADSRState>,
}

impl ADSR {
    fn process_sample(
        &mut self,
        gate: bool,
        attack_s: f32,
        decay_s: f32,
        sustain: f32,
        release_s: f32,
    ) -> f32 {
        self.state_machine
            .tick(ADSRContext {
                gate_on: gate,
                attack_dur: Duration::from_secs_f32(attack_s.max(0.0)).to_samples(self.sample_rate),
                decay_dur: Duration::from_secs_f32(decay_s.max(0.0)).to_samples(self.sample_rate),
                sustain,
                release_dur: Duration::from_secs_f32(release_s.max(0.0)).to_samples(self.sample_rate),
            })
            .value()
    }
}

impl Node<5, 1> for ADSR {
    const INPUT_NAMES: [&'static str; 5] = [
        "gate",
        "attack seconds",
        "decay seconds",
        "sustain",
        "release seconds",
    ];
    const OUTPUT_NAMES: [&'static str; 1] = ["envelope"];
    fn process(&mut self, input: [f32; 5], output: &mut [f32; 1]) {
        output[0] = self.process_sample(input[0] > 0.5, input[1], input[2], input[3], input[4]);
    }
    fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            state_machine: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_adsr() {
        use super::*;

        let mut adsr = ADSR::new(16);

        let gate_off = |adsr: &mut ADSR| {
            let mut out = [0.0];
            adsr.process([0.0, 0.25, 0.25, 0.5, 0.25], &mut out);
            out
        };
        let gate_on = |adsr: &mut ADSR| {
            let mut out = [0.0];
            adsr.process([1.0, 0.25, 0.25, 0.5, 0.25], &mut out);
            out
        };

        assert_eq!(gate_off(&mut adsr), [0.0]);

        // attack
        assert_eq!(gate_on(&mut adsr), [0.0]);
        assert_eq!(gate_on(&mut adsr), [0.25]);
        assert_eq!(gate_on(&mut adsr), [0.5]);
        assert_eq!(gate_on(&mut adsr), [0.75]);
        assert_eq!(gate_on(&mut adsr), [1.0]);

        // decay
        assert_eq!(gate_on(&mut adsr), [0.875]);
        assert_eq!(gate_on(&mut adsr), [0.75]);
        assert_eq!(gate_on(&mut adsr), [0.625]);
        assert_eq!(gate_on(&mut adsr), [0.5]);

        // sustain
        for _ in 0..10 {
            assert_eq!(gate_on(&mut adsr), [0.5]);
        }

        // release
        assert_eq!(gate_off(&mut adsr), [0.5]);
        assert_eq!(gate_off(&mut adsr), [0.375]);
        assert_eq!(gate_off(&mut adsr), [0.25]);
        assert_eq!(gate_off(&mut adsr), [0.125]);
        assert_eq!(gate_off(&mut adsr), [0.0]);

        // idle
        assert_eq!(gate_off(&mut adsr), [0.0]);
    }
}
