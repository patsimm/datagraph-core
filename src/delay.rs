use crate::{node::Effect, ring_buffer::RingBuffer};

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

impl Effect for Delay {
    fn process(&mut self, input: f32, _: usize) -> f32 {
        let old_value = self.ringbuf.pop().unwrap_or(0.0);
        let new_value = input + old_value * 0.6;
        self.ringbuf
            .push(new_value)
            .expect("Ring buffer should never be full");
        new_value
    }
}
