use std::f32::consts::PI;
use wasm_bindgen::prelude::*;

use cpal::SampleRate;

use crate::{
    frequency::{Frequency, FromCv},
    graph::Node,
};

#[wasm_bindgen]
pub struct Oscillator {
    sample_rate: SampleRate,
    phi: f32,
}

#[wasm_bindgen]
impl Oscillator {
    #[wasm_bindgen(constructor)]
    pub fn _construct(sample_rate: SampleRate) -> Self {
        Self::new(sample_rate)
    }
}

impl Oscillator {
    pub fn new(sample_rate: SampleRate) -> Self {
        Self {
            sample_rate,
            phi: 0.0,
        }
    }
}

impl Node<1, 1> for Oscillator {
    const INPUT_NAMES: [&'static str; 1] = ["frequency"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];

    fn process(&mut self, input: [f32; 1], _: usize) -> [f32; 1] {
        let frequency = Frequency::from_cv(input[0]);
        self.phi += (*frequency / self.sample_rate as f32) * 2.0 * PI;
        if self.phi > 2.0 * PI {
            self.phi -= 2.0 * PI;
        }
        [f32::sin(self.phi)]
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
        let osc = Oscillator::new(SR);
        assert_eq!(osc.phi, 0.0);
    }

    #[test]
    fn process_output_is_bounded() {
        let mut osc = Oscillator::new(SR);
        for _ in 0..SR {
            let [out] = osc.process([0.0], 0);
            assert!((-1.0..=1.0).contains(&out), "output out of range: {out}");
        }
    }

    #[test]
    fn process_phase_stays_bounded() {
        let mut osc = Oscillator::new(SR);
        for _ in 0..SR {
            osc.process([0.0], 0);
            assert!(
                osc.phi >= 0.0 && osc.phi <= 2.0 * PI + f32::EPSILON,
                "phi out of range: {}",
                osc.phi
            );
        }
    }

    #[test]
    fn process_is_periodic() {
        let cv = Frequency::from_hz(441.0).to_cv();
        let cycle_len = Duration::from(Frequency::from_cv(cv)).to_samples(SR);

        let mut osc = Oscillator::new(SR);
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
