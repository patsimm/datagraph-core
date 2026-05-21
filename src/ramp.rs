use crate::helpers::lerp;

#[derive(Clone, Copy)]
pub struct Ramp {
    target: f32,
    start: f32,
    duration: usize,
    running_sample: Option<usize>,
}

impl Ramp {
    pub fn new(start: f32, target: f32, duration: usize) -> Self {
        Self {
            target,
            start,
            duration,
            running_sample: None,
        }
    }

    pub fn duration(&self) -> usize {
        self.duration
    }

    pub fn is_active(&self) -> bool {
        self.running_sample
            .map(|running_sample| running_sample <= self.duration)
            .unwrap_or(false)
    }

    pub fn start(&mut self) {
        self.running_sample = Some(0);
    }

    pub fn update(&mut self) -> Option<f32> {
        let Some(running_sample) = self.running_sample else {
            return None; // Ramp hasn't started yet
        };
        if running_sample >= self.duration {
            self.running_sample = None;
            return Some(self.target);
        }
        self.running_sample = Some(running_sample + 1);
        Some(lerp(
            self.start,
            self.target,
            running_sample as f32 / self.duration as f32,
        ))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_ramp() {
        let mut ramp = super::Ramp::new(0.0, 1.0, 4);
        assert_eq!(ramp.update(), None);
        ramp.start();
        assert_eq!(ramp.update(), Some(0.0));
        assert_eq!(ramp.update(), Some(0.25));
        assert_eq!(ramp.update(), Some(0.5));
        assert_eq!(ramp.update(), Some(0.75));
        assert_eq!(ramp.update(), Some(1.0));
        assert_eq!(ramp.update(), None);
    }
}
