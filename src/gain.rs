use crate::node::Effect;

pub struct Gain {
    gain: f32,
}

impl Gain {
    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }
}

impl Default for Gain {
    fn default() -> Self {
        Self { gain: 1.0 }
    }
}

impl Effect for Gain {
    fn process(&mut self, input: f32) -> f32 {
        self.gain * input
    }
}
