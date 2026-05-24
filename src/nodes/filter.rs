use std::time::Duration;

use cpal::SampleRate;

use crate::{graph::Node, helpers::ToSamples};

pub struct OnePoleLowPass {
    prev_output: f32,
    alpha: f32,
}

impl OnePoleLowPass {
    pub fn new(alpha: f32) -> Self {
        Self {
            prev_output: 0.0,
            alpha,
        }
    }

    pub fn from_smoothing_time(time: Duration, sample_rate: SampleRate) -> Self {
        let alpha = 1.0 - (-1.0 / time.to_samples(sample_rate) as f32).exp();
        Self::new(alpha)
    }
}

impl Node<1, 1> for OnePoleLowPass {
    const INPUT_NAMES: [&'static str; 1] = ["input"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 1]) -> [f32; 1] {
        let result = self.alpha * input[0] + (1.0 - self.alpha) * self.prev_output;
        self.prev_output = result;
        [result]
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        graph::{Graph, Node},
        nodes::param::Param,
    };

    use super::*;

    #[test]
    fn alpha_one_passes_signal_through() {
        let mut filter = OnePoleLowPass::new(1.0);
        assert_eq!(filter.process([0.75]), [0.75]);
        assert_eq!(filter.process([0.0]), [0.0]);
    }

    #[test]
    fn alpha_zero_always_outputs_zero() {
        let mut filter = OnePoleLowPass::new(0.0);
        assert_eq!(filter.process([1.0]), [0.0]);
        assert_eq!(filter.process([1.0]), [0.0]);
    }

    #[test]
    fn step_response_converges_to_target() {
        let mut filter = OnePoleLowPass::new(0.1);
        let mut output = 0.0;
        for _ in 0..1000 {
            output = filter.process([1.0])[0];
        }
        assert!(
            (output - 1.0).abs() < 0.001,
            "Expected ~1.0, got {}",
            output
        );
    }

    // After exactly N samples (the smoothing time constant in samples), the step
    // response of a one-pole filter reaches 1 - 1/e ≈ 63.2% of the target.
    #[test]
    fn smoothing_time_constant_reaches_63_percent_at_tau() {
        let sample_rate: cpal::SampleRate = 44100;
        let n_samples: usize = 100;
        let time = Duration::from_secs_f64(n_samples as f64 / 44100.0);
        let mut filter = OnePoleLowPass::from_smoothing_time(time, sample_rate);

        let mut output = 0.0;
        for _ in 0..n_samples {
            output = filter.process([1.0])[0];
        }

        let expected = 1.0 - (-1.0_f32).exp(); // 1 - 1/e ≈ 0.6321
        assert!(
            (output - expected).abs() < 0.01,
            "Expected ~{:.4} (1 - 1/e), got {:.4}",
            expected,
            output
        );
    }

    #[test]
    fn param_change_is_smoothed_through_graph() {
        let mut graph = Graph::new();
        let mut param = Param::new(0.0);
        let param_id = graph.add(param.node());
        let filter_id = graph.add(OnePoleLowPass::new(0.5));
        graph.connect(param_id, 0, filter_id, 0).unwrap();

        // Settle at 0.0
        for _ in 0..10 {
            graph.tick();
        }
        assert_eq!(
            *graph
                .port_value(filter_id, 0, crate::graph::PortType::Output)
                .unwrap(),
            0.0
        );

        // Step param to 1.0 — filter should NOT jump instantly
        // With double-buffered graph, the param cache updates on tick 10,
        // and the filter sees the new value on tick 11.
        param.set(1.0);
        graph.tick();
        graph.tick();
        let first = *graph
            .port_value(filter_id, 0, crate::graph::PortType::Output)
            .unwrap();
        // alpha=0.5: expected output = 0.5 * 1.0 + 0.5 * 0.0 = 0.5
        assert!(
            (first - 0.5).abs() < 0.001,
            "Expected 0.5 on first tick after param change, got {}",
            first
        );

        // Filter must not have jumped straight to 1.0 — that would mean no smoothing
        assert!(
            first < 1.0,
            "Filter output jumped to 1.0 immediately — no smoothing is happening"
        );

        // Converges to 1.0 after many ticks
        for _ in 12..200 {
            graph.tick();
        }
        let final_out = *graph
            .port_value(filter_id, 0, crate::graph::PortType::Output)
            .unwrap();
        assert!(
            (final_out - 1.0).abs() < 0.001,
            "Expected ~1.0 after settling, got {}",
            final_out
        );
    }
}
