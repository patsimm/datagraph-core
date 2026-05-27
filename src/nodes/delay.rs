use crate::{dsp::RingBuffer, graph::Node};

const MAX_SECONDS: usize = 5;

pub struct Delay {
    sample_rate: u32,
    ringbuf: RingBuffer<f32>,
    last_delay_samples: usize,
}

impl Node<2, 1> for Delay {
    const INPUT_NAMES: [&'static str; 2] = ["input", "delay time ms"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 2], output: &mut [f32; 1]) {
        let delay_samples = input[1].max(0.0) as usize * self.sample_rate as usize / 1000;
        if delay_samples != self.last_delay_samples {
            let offset = self.last_delay_samples as isize - delay_samples as isize;
            self.ringbuf.move_read_index(offset);
            self.last_delay_samples = delay_samples;
        }
        self.ringbuf
            .push(input[0])
            .expect("Ring buffer should never be full");
        output[0] = self
            .ringbuf
            .pop()
            .expect("Ring buffer should never be empty");
    }
    fn new(sample_rate: u32) -> Self {
        let buffer_size = sample_rate as usize * MAX_SECONDS;
        let ringbuf = RingBuffer::new(buffer_size).flooded(0.0);
        ringbuf.move_read_index((buffer_size - 1) as isize);
        Self {
            sample_rate,
            ringbuf,
            last_delay_samples: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_zero_delay() {
        use super::*;
        use crate::graph::Node;
        let mut delay = Delay::new(1000);
        let mut out = [0.0];
        delay.process([1.0, 0.0], &mut out);
        assert_eq!(out, [1.0]);
        delay.process([0.5, 0.0], &mut out);
        assert_eq!(out, [0.5]);
        delay.process([0.0, 0.0], &mut out);
        assert_eq!(out, [0.0]);
    }

    #[test]
    fn test_100ms_delay() {
        use super::*;
        use crate::graph::Node;
        let mut delay = Delay::new(1000); // 1ms = 1 sample at 1000Hz
        let mut out = [0.0];

        // First 100 samples: pre-filled silence passes through the delay buffer
        for _ in 0..100 {
            delay.process([1.0, 100.0], &mut out);
            assert_eq!(out, [0.0]);
        }

        // Next 100 samples: the 1.0 inputs from 100ms ago come out
        for _ in 0..100 {
            delay.process([0.0, 100.0], &mut out);
            assert_eq!(out, [1.0]);
        }

        // Silence again once the delayed signal drains
        delay.process([0.0, 100.0], &mut out);
        assert_eq!(out, [0.0]);
    }
}
