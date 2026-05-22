use crate::helpers::lerp;

#[derive(Clone, Copy)]
pub struct Ramp {
    target: f32,
    start: f32,
    current: f32,
    duration: usize,
    running_sample: usize,
}

impl Ramp {
    pub fn new(start: f32, target: f32, duration: usize) -> Self {
        Self {
            target,
            start,
            current: start,
            duration,
            running_sample: 0,
        }
    }

    pub fn duration(&self) -> usize {
        self.duration
    }

    pub fn value(&self) -> f32 {
        self.current
    }

    pub fn tick(&mut self) -> bool {
        self.running_sample += 1;
        if self.running_sample >= self.duration {
            self.current = self.target;
            return false;
        }
        self.current = lerp(
            self.start,
            self.target,
            self.running_sample as f32 / self.duration as f32,
        );
        true
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_ramp() {
        let mut ramp = super::Ramp::new(0.0, 1.0, 4);
        assert_eq!(ramp.value(), 0.0);
        assert!(ramp.tick());
        assert_eq!(ramp.value(), 0.25);
        assert!(ramp.tick());
        assert_eq!(ramp.value(), 0.5);
        assert!(ramp.tick());
        assert_eq!(ramp.value(), 0.75);
        assert!(!ramp.tick());
        assert_eq!(ramp.value(), 1.0);
    }
}
