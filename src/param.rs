use std::sync::Arc;
use wasm_bindgen::prelude::*;

use crate::{graph::Node, helpers::AtomicF32};

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
    const INPUT_NAMES: [&'static str; 0] = [];
    const OUTPUT_NAMES: [&'static str; 1] = ["value"];
    fn process(&mut self, _: [f32; 0], _: usize) -> [f32; 1] {
        [self.0.load(std::sync::atomic::Ordering::Relaxed)]
    }
}
