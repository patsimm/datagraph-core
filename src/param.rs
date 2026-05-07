use std::sync::Arc;
use wasm_bindgen::prelude::*;

use crate::{
    graph::Node,
    helpers::{AtomicF32, lerp},
};

#[wasm_bindgen]
pub struct Param {
    value: Arc<AtomicF32>,
}

impl From<f32> for Param {
    fn from(value: f32) -> Self {
        Self {
            value: Arc::new(AtomicF32::new(value)),
        }
    }
}

impl Param {
    pub fn new(value: f32) -> Self {
        Self {
            value: Arc::new(AtomicF32::new(value)),
        }
    }

    pub fn node(&self) -> ParamNode {
        ParamNode(self.value.clone())
    }

    pub fn set(&mut self, value: f32) {
        self.value
            .store(value, std::sync::atomic::Ordering::Relaxed);
    }
}

pub struct ParamNode(Arc<AtomicF32>);

impl From<&Param> for ParamNode {
    fn from(param: &Param) -> Self {
        param.node()
    }
}

impl Node<0, 1> for ParamNode {
    const NODE_TYPE: crate::graph::NodeType = crate::graph::NodeType::Param;
    const INPUT_NAMES: [&'static str; 0] = [];
    const OUTPUT_NAMES: [&'static str; 1] = ["value"];
    fn process(&mut self, _: [f32; 0], _: usize) -> [f32; 1] {
        [self.0.load(std::sync::atomic::Ordering::Relaxed)]
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
