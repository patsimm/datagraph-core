use crate::helpers::lerp;

#[derive(Default)]
pub struct Param {
    value: f32,
    ramp: Option<Ramp>,
}

impl Param {
    pub fn set(&mut self, value: f32) {
        self.value = value;
        self.ramp = None; // Cancel any active ramp
    }

    pub fn get(&mut self, sample_num: usize) -> f32 {
        let Some(ramp) = &mut self.ramp else {
            return self.value;
        };
        if !ramp.is_active(sample_num) {
            return self.value;
        }
        let Some(new_value) = ramp.update(sample_num) else {
            self.value = ramp.target;
            self.ramp = None;
            return self.value;
        };
        self.value = new_value;
        self.value
    }
}

impl From<f32> for Param {
    fn from(value: f32) -> Self {
        Self { value, ramp: None }
    }
}

pub struct Ramp {
    target: f32,
    start: f32,
    duration: usize,
    start_sample: Option<usize>,
}

impl Ramp {
    pub fn new(start: f32, target: f32, duration: usize) -> Self {
        Self {
            target,
            start,
            duration,
            start_sample: None,
        }
    }

    pub fn duration(&self) -> usize {
        self.duration
    }

    pub fn is_active(&self, sample_num: usize) -> bool {
        if let Some(start_sample) = self.start_sample {
            return start_sample <= sample_num;
        }
        false
    }

    pub fn start(&mut self, at: usize) {
        self.start_sample = Some(at);
    }

    pub fn update(&mut self, now: usize) -> Option<f32> {
        let Some(start_time) = self.start_sample else {
            return None; // Ramp hasn't started yet
        };
        if start_time >= now {
            return None;
        }
        let delta_samples = now - start_time;
        if delta_samples >= self.duration {
            self.start_sample = None;
            return Some(self.target);
        }
        Some(lerp(
            self.start,
            self.target,
            delta_samples as f32 / self.duration as f32,
        ))
    }
}
