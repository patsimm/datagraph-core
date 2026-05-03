use std::time::{Duration, Instant};

use crate::node::Effect;

pub struct Ramp {
    target: f32,
    start: f32,
    duration: Duration,
    start_time: Option<std::time::Instant>,
}

impl Ramp {
    pub fn new(start: f32, target: f32, duration: Duration) -> Self {
        Self {
            target,
            start, // This should be set to the current gain when ramp starts
            duration,
            start_time: None,
        }
    }

    pub fn is_active(&self, now: Instant) -> bool {
        if let Some(start_time) = self.start_time {
            return start_time < now;
        }
        false
    }

    pub fn start(&mut self, at: Instant) {
        self.start_time = Some(at);
    }

    pub fn update(&mut self, now: Instant) -> Option<f32> {
        let Some(start_time) = self.start_time else {
            return None; // Ramp hasn't started yet
        };
        if start_time >= now {
            return None;
        }
        let delta_time = now - start_time;
        if delta_time >= self.duration {
            self.start_time = None;
            return Some(self.target);
        }
        Some(
            self.start
                + (self.target - self.start)
                    * (delta_time.as_millis() as f32 / self.duration.as_millis() as f32),
        )
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

pub struct ADSR {
    attack: Ramp,
    decay: Ramp,
    sustain: f32,
    release: Ramp,
    start_time: Option<std::time::Instant>,
    stop_time: Option<std::time::Instant>,
}

impl ADSR {
    pub fn new(attack: Duration, decay: Duration, sustain: f32, release: Duration) -> Self {
        Self {
            attack: Ramp::new(0.0, 1.0, attack),
            decay: Ramp::new(1.0, sustain, decay),
            sustain,
            release: Ramp::new(sustain, 0.0, release),
            start_time: None,
            stop_time: None,
        }
    }

    pub fn update(&mut self, now: Instant) -> Option<f32> {
        if let Some(start_time) = self.start_time {
            if start_time > now {
                return None; // Note hasn't started yet
            }
            if self.attack.is_active(now) {
                return self.attack.update(now);
            }
            if self.decay.is_active(now) {
                return self.decay.update(now);
            }
            return Some(self.sustain);
        }
        if let Some(stop_time) = self.stop_time {
            if stop_time > now {
                return None; // Note hasn't stopped yet
            }
            if self.release.is_active(now) {
                return self.release.update(now);
            }
            self.stop_time = None;
        }
        None
    }

    pub fn start(&mut self) {
        let now = Instant::now();
        self.start_time = Some(now);
        self.attack.start(now);
        self.decay.start(now + self.attack.duration);
    }

    pub fn stop(&mut self) {
        let now = Instant::now();
        self.start_time = None;
        self.stop_time = Some(now);
        self.release.start(now);
    }
}
