use std::f32::consts::{PI, TAU};

use cpal::SampleRate;

use crate::{
    frequency::{Frequency, FromCv},
    graph::Node,
};

pub struct OscillatorCore {
    sample_rate: SampleRate,
    phi: f32,
}

impl OscillatorCore {
    pub fn new(sample_rate: SampleRate) -> Self {
        Self {
            sample_rate,
            phi: 0.0,
        }
    }

    pub fn advance_phase(&mut self, frequency: &Frequency) -> f32 {
        self.phi += (frequency.hz() / self.sample_rate as f32) * TAU;
        if self.phi > TAU {
            self.phi -= TAU;
        }
        self.phi
    }
}

pub trait Oscillator: Node<1, 1> {
    fn core(&mut self) -> &OscillatorCore;
    fn core_mut(&mut self) -> &mut OscillatorCore;
    fn oscillate(&mut self, phi: f32) -> f32;
}

impl<T: Oscillator> Node<1, 1> for T {
    const INPUT_NAMES: [&'static str; 1] = ["frequency"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 1], _: usize) -> [f32; 1] {
        let frequency = Frequency::from_cv(input[0]);
        let phi = self.core_mut().advance_phase(&frequency);
        [self.oscillate(phi)]
    }
}

pub struct Sin {
    core: OscillatorCore,
}

impl Sin {
    pub fn new(sample_rate: SampleRate) -> Self {
        Self {
            core: OscillatorCore::new(sample_rate),
        }
    }
}

impl Oscillator for Sin {
    fn oscillate(&mut self, phi: f32) -> f32 {
        f32::sin(phi)
    }

    fn core(&mut self) -> &OscillatorCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut OscillatorCore {
        &mut self.core
    }
}

pub struct Saw {
    core: OscillatorCore,
}

impl Saw {
    pub fn new(sample_rate: SampleRate) -> Self {
        Self {
            core: OscillatorCore::new(sample_rate),
        }
    }
}

impl Oscillator for Saw {
    fn oscillate(&mut self, phi: f32) -> f32 {
        2.0 * (phi / TAU) - 1.0
    }

    fn core(&mut self) -> &OscillatorCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut OscillatorCore {
        &mut self.core
    }
}

pub struct Square {
    core: OscillatorCore,
}

impl Square {
    pub fn new(sample_rate: SampleRate) -> Self {
        Self {
            core: OscillatorCore::new(sample_rate),
        }
    }
}

impl Oscillator for Square {
    fn oscillate(&mut self, phi: f32) -> f32 {
        if phi < PI { 1.0 } else { -1.0 }
    }

    fn core(&mut self) -> &OscillatorCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut OscillatorCore {
        &mut self.core
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        frequency::{FromHz, ToCv},
        helpers::ToSamples,
    };

    use super::*;

    const SR: u32 = 44100;

    #[test]
    fn new_starts_with_zero_phase() {
        let osc = Sin::new(SR);
        assert_eq!(osc.core.phi, 0.0);
    }

    #[test]
    fn process_output_is_bounded() {
        let mut osc = Sin::new(SR);
        for _ in 0..SR {
            let [out] = osc.process([0.0], 0);
            assert!((-1.0..=1.0).contains(&out), "output out of range: {out}");
        }
    }

    #[test]
    fn process_phase_stays_bounded() {
        let mut osc = Sin::new(SR);
        for _ in 0..SR {
            osc.process([0.0], 0);
            assert!(
                osc.core.phi >= 0.0 && osc.core.phi <= TAU + f32::EPSILON,
                "phi out of range: {}",
                osc.core.phi
            );
        }
    }

    #[test]
    fn process_is_periodic() {
        let cv = Frequency::from_hz(441.0).to_cv();
        let cycle_len = Duration::from(Frequency::from_cv(cv)).to_samples(SR);

        let mut osc = Sin::new(SR);
        let outputs: Vec<f32> = (0..cycle_len * 2)
            .map(|_| osc.process([cv], 0)[0])
            .collect();

        for i in 0..cycle_len {
            assert!(
                (outputs[i] - outputs[i + cycle_len]).abs() < 1e-5,
                "periodicity broken at sample {i}: {} vs {}",
                outputs[i],
                outputs[i + cycle_len]
            );
        }
    }
}
