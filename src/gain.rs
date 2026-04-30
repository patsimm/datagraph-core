use crate::node::Effect;

pub struct Ramp {
    target: f32,
    start: f32,
    duration_ms: u64,
    start_time: Option<std::time::Instant>,
}

impl Ramp {
    pub fn new(start: f32, target: f32, duration_ms: u64) -> Self {
        Self {
            target,
            start, // This should be set to the current gain when ramp starts
            duration_ms,
            start_time: None,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(std::time::Instant::now());
    }

    pub fn update(&mut self) -> f32 {
        let Some(start_time) = self.start_time else {
            return self.start; // Ramp hasn't started yet
        };
        let now = std::time::Instant::now();
        let delta_time = now - start_time;
        self.start
            + (self.target - self.start) * (delta_time.as_millis() as f32 / self.duration_ms as f32)
    }
}

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
