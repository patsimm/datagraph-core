use std::time::Duration;

use crate::{dsp::ToSamples, graph::Node};

pub struct OnePoleLowPass {
    sample_rate: u32,
    prev_output: f32,
}

impl Node<2, 1> for OnePoleLowPass {
    const INPUT_NAMES: [&'static str; 2] = ["input", "smoothing time seconds"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 2], output: &mut [f32; 1]) {
        let alpha = 1.0
            - (-1.0 / Duration::from_secs_f32(input[1]).to_samples(self.sample_rate) as f32).exp();
        let result = alpha * input[0] + (1.0 - alpha) * self.prev_output;
        self.prev_output = result;
        output[0] = result;
    }
    fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            prev_output: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::{Graph, Node, PortValueAccess, Tickable};

    use super::*;

    #[test]
    fn smoothing_zero_passes_signal_through() {
        // smoothing_time=0 → alpha=1 → output equals input exactly
        let mut filter = OnePoleLowPass::new(44100);
        let mut out = [0.0];
        filter.process([0.75, 0.0], &mut out);
        assert_eq!(out, [0.75]);
        filter.process([0.0, 0.0], &mut out);
        assert_eq!(out, [0.0]);
    }

    #[test]
    fn long_smoothing_barely_moves() {
        // Huge smoothing time → alpha≈0 → output barely moves from 0
        let mut filter = OnePoleLowPass::new(1);
        let mut out = [0.0];
        filter.process([1.0, 1e10], &mut out);
        let out = out[0];
        assert!(out < 0.001, "expected near zero, got {out}");
    }

    #[test]
    fn step_response_converges_to_target() {
        // 44-sample smoothing time → fast but non-instant convergence
        let mut filter = OnePoleLowPass::new(44100);
        let mut out = [0.0];
        for _ in 0..10000 {
            filter.process([1.0, 0.001], &mut out);
        }
        let output = out[0];
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
        let sample_rate: u32 = 44100;
        let n_samples: usize = 100;
        let time_secs = n_samples as f32 / sample_rate as f32;
        let mut filter = OnePoleLowPass::new(sample_rate);

        let mut out = [0.0];
        for _ in 0..n_samples {
            filter.process([1.0, time_secs], &mut out);
        }
        let output = out[0];

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
        use crate::graph::{PortKey, PortType};
        let mut graph = Graph::new(1);
        let param_id = *graph.add_param(0.0).node_id();
        // sample_rate=1 Hz, smoothing_secs=1.0 → n_samples=1 → alpha = 1 - 1/e ≈ 0.632
        let filter_id = *graph.add::<OnePoleLowPass>().node_id();
        let smoothing_id = *graph.add_param(1.0).node_id();
        graph.connect(&param_id, 0, &filter_id, 0).unwrap();
        graph.connect(&smoothing_id, 0, &filter_id, 1).unwrap();

        // Settle at 0.0
        for _ in 0..10 {
            graph.tick();
        }
        assert_eq!(
            *graph
                .port_value(&PortKey::new(filter_id, 0, PortType::Output))
                .unwrap(),
            0.0
        );

        // Step param to 1.0 — filter should NOT jump instantly
        // With double-buffered graph, the param cache updates on tick 10,
        // and the filter sees the new value on tick 11.
        graph.set_param_value(&param_id, 1.0).unwrap();
        graph.tick();
        graph.tick();
        let first = *graph
            .port_value(&PortKey::new(filter_id, 0, PortType::Output))
            .unwrap();
        // alpha = 1 - 1/e ≈ 0.6321: expected output = alpha * 1.0 + (1-alpha) * 0.0
        let expected_alpha = 1.0 - (-1.0_f32).exp();
        assert!(
            (first - expected_alpha).abs() < 0.001,
            "Expected {expected_alpha:.4} on first tick after param change, got {first}"
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
            .port_value(&PortKey::new(filter_id, 0, PortType::Output))
            .unwrap();
        assert!(
            (final_out - 1.0).abs() < 0.001,
            "Expected ~1.0 after settling, got {}",
            final_out
        );
    }
}
