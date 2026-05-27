use std::f32::consts::TAU;

use crate::{
    dsp::{Frequency, FromBpm, State, StateMachine},
    graph::Node,
    nodes::oscillator::{Oscillator, OscillatorCore},
};

const PULSE_WIDTH: f32 = TAU * 0.05;

pub struct Clock {
    core: OscillatorCore,
}

impl Oscillator for Clock {
    const OSCILLATOR_INPUT_NAME: &'static str = "bpm";

    fn oscillate(&mut self, phi: f32) -> f32 {
        if phi < PULSE_WIDTH { 1.0 } else { 0.0 }
    }

    fn core_mut(&mut self) -> &mut OscillatorCore {
        &mut self.core
    }
    fn create(sample_rate: u32) -> Self {
        Self {
            core: OscillatorCore::new(sample_rate).with_frequency_calc(Frequency::from_bpm),
        }
    }
}

#[derive(Copy, Clone)]
enum ClockDividerState {
    High,
    Low(u32, bool),
}

impl Default for ClockDividerState {
    fn default() -> Self {
        ClockDividerState::Low(0, false)
    }
}

impl State<ClockDividerContext> for ClockDividerState {
    fn next_state(&mut self, context: ClockDividerContext) -> Self {
        match *self {
            ClockDividerState::High => {
                if !context.input_high {
                    ClockDividerState::Low(0, false)
                } else {
                    ClockDividerState::High
                }
            }
            ClockDividerState::Low(count, prev_input_high) => {
                if context.input_high && !prev_input_high {
                    let new_count = count + 1;
                    if count >= context.division - 1 {
                        return ClockDividerState::High;
                    }
                    ClockDividerState::Low(new_count, true)
                } else {
                    ClockDividerState::Low(count, context.input_high)
                }
            }
        }
    }
}

struct ClockDividerContext {
    input_high: bool,
    division: u32,
}

pub struct ClockDivider {
    state_machine: StateMachine<ClockDividerContext, ClockDividerState>,
}

impl Node<2, 1> for ClockDivider {
    const INPUT_NAMES: [&'static str; 2] = ["clock", "division"];
    const OUTPUT_NAMES: [&'static str; 1] = ["divided_clock"];
    fn process(&mut self, input: [f32; 2], output: &mut [f32; 1]) {
        if input[1] < 1.0 {
            output[0] = 0.0; // If division is less than 1, output is always low
            return;
        }
        let state = self.state_machine.next(ClockDividerContext {
            input_high: input[0] > 0.5,
            division: input[1] as u32,
        });
        match state {
            ClockDividerState::High => output[0] = 1.0,
            ClockDividerState::Low(_, _) => output[0] = 0.0,
        }
    }
    fn new(_: u32) -> Self {
        Self {
            state_machine: StateMachine::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;

    fn tick(node: &mut ClockDivider, clock: f32, division: f32) -> f32 {
        let mut out = [0.0];
        node.process([clock, division], &mut out);
        out[0]
    }

    fn run(node: &mut ClockDivider, clocks: &[f32], division: f32) -> Vec<f32> {
        clocks.iter().map(|&c| tick(node, c, division)).collect()
    }

    #[test]
    fn test_divide_by_1_is_passthrough() {
        let mut d = ClockDivider::new(44100);
        let input = [1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0];
        assert_eq!(run(&mut d, &input, 1.0), input.to_vec());
    }

    #[test]
    fn test_divide_by_2() {
        let mut d = ClockDivider::new(44100);
        // Two input pulses → output high only on the 2nd
        let input = [1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0];
        assert_eq!(
            run(&mut d, &input, 2.0),
            vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]
        );
    }

    #[test]
    fn test_divide_by_2_repeats_correctly() {
        let mut d = ClockDivider::new(44100);
        let input = [
            1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0,
        ];
        assert_eq!(
            run(&mut d, &input, 2.0),
            vec![
                0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0,
            ]
        );
    }

    #[test]
    fn test_divide_by_4() {
        let mut d = ClockDivider::new(44100);
        let input: Vec<f32> = (0..4).flat_map(|_| [1.0_f32, 1.0, 0.0, 0.0]).collect();
        assert_eq!(
            run(&mut d, &input, 4.0),
            vec![
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0,
            ]
        );
    }

    #[test]
    fn test_division_less_than_1_always_low() {
        let mut d = ClockDivider::new(44100);
        let input = [1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
        assert_eq!(run(&mut d, &input, 0.5), vec![0.0; 6]);
    }
}
