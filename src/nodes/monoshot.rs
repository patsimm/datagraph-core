use std::time::Duration;

use crate::{
    dsp::{State, StateMachine, ToSamples},
    graph::Node,
};

struct MonoshotContext {
    trigger: f32,
    duration_samples: usize,
}

enum MonoshotState {
    Idle(f32),
    High(usize),
}

impl Default for MonoshotState {
    fn default() -> Self {
        MonoshotState::Idle(0.0)
    }
}

impl MonoshotState {
    fn value(&self) -> f32 {
        match self {
            MonoshotState::Idle(_) => 0.0,
            MonoshotState::High(_) => 1.0,
        }
    }
}

impl State<MonoshotContext> for MonoshotState {
    fn next_state(&mut self, context: MonoshotContext) -> Self {
        match self {
            MonoshotState::Idle(prev) => {
                if *prev <= 0.5 && context.trigger > 0.5 {
                    MonoshotState::High(context.duration_samples.saturating_sub(1))
                } else {
                    MonoshotState::Idle(context.trigger)
                }
            }
            MonoshotState::High(remaining) => {
                if *remaining == 0 {
                    MonoshotState::Idle(context.trigger)
                } else {
                    MonoshotState::High(*remaining - 1)
                }
            }
        }
    }
}

pub struct Monoshot {
    sample_rate: u32,
    state_machine: StateMachine<MonoshotContext, MonoshotState>,
}

impl Node<2, 1> for Monoshot {
    const INPUT_NAMES: [&'static str; 2] = ["trigger", "duration seconds"];
    const OUTPUT_NAMES: [&'static str; 1] = ["gate"];

    fn process(&mut self, input: [f32; 2], output: &mut [f32; 1]) {
        let duration_samples =
            Duration::from_secs_f32(input[1].max(0.0)).to_samples(self.sample_rate);
        output[0] = self
            .state_machine
            .next(MonoshotContext {
                trigger: input[0],
                duration_samples,
            })
            .value();
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
    use super::*;

    fn process(node: &mut Monoshot, trigger: f32, duration: f32) -> f32 {
        let mut out = [0.0];
        node.process([trigger, duration], &mut out);
        out[0]
    }

    #[test]
    fn test_monoshot_holds_gate_for_duration() {
        // sample_rate=10, duration=0.3s → 3 samples high
        let mut m = Monoshot::new(10);

        assert_eq!(process(&mut m, 0.0, 0.3), 0.0); // idle
        assert_eq!(process(&mut m, 1.0, 0.3), 1.0); // rising edge → high
        assert_eq!(process(&mut m, 1.0, 0.3), 1.0);
        assert_eq!(process(&mut m, 0.0, 0.3), 1.0); // still high (trigger dropped)
        assert_eq!(process(&mut m, 0.0, 0.3), 0.0); // back to idle
    }

    #[test]
    fn test_monoshot_ignores_trigger_while_high() {
        let mut m = Monoshot::new(10);

        process(&mut m, 1.0, 0.3); // start gate
        assert_eq!(process(&mut m, 0.0, 0.3), 1.0);
        // re-trigger mid-hold should not restart
        assert_eq!(process(&mut m, 1.0, 0.3), 1.0);
        assert_eq!(process(&mut m, 0.0, 0.3), 0.0); // expires normally
    }

    #[test]
    fn test_monoshot_retrigger_after_idle() {
        let mut m = Monoshot::new(10);

        // 0.1s @ 10Hz = 1 sample high
        assert_eq!(process(&mut m, 1.0, 0.1), 1.0); // rising edge → 1 sample high
        assert_eq!(process(&mut m, 0.0, 0.1), 0.0); // idle

        // second trigger fires again after idle
        assert_eq!(process(&mut m, 1.0, 0.1), 1.0);
        assert_eq!(process(&mut m, 0.0, 0.1), 0.0);
    }
}
