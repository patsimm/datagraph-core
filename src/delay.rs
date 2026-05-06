use crate::{graph::Node, ring_buffer::RingBuffer};

pub struct Delay {
    ringbuf: RingBuffer<f32, 22050>,
}

impl Delay {
    pub fn new() -> Self {
        Self {
            ringbuf: RingBuffer::new().flooded(0.0),
        }
    }
}

impl Node<1, 1> for Delay {
    const INPUT_NAMES: [&'static str; 1] = ["input"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 1], _: usize) -> [f32; 1] {
        let old_value = self.ringbuf.pop().unwrap_or(0.0);
        let new_value = input[0] + old_value * 0.6;
        self.ringbuf
            .push(new_value)
            .expect("Ring buffer should never be full");
        [new_value]
    }
}
