use std::sync::Arc;
use wasm_bindgen::prelude::*;

use crate::{graph::Node, helpers::AtomicF32};

pub struct ParamHandle {
    value: Arc<AtomicF32>,
}

impl From<f32> for Param {
    fn from(value: f32) -> Self {
        Self(Arc::new(AtomicF32::new(value)))
    }
}

impl ParamHandle {
    pub fn set(&mut self, value: f32) {
        self.value
            .store(value, std::sync::atomic::Ordering::Relaxed);
    }
}

#[wasm_bindgen]
pub struct Param(Arc<AtomicF32>);

impl From<&Param> for ParamHandle {
    fn from(param: &Param) -> Self {
        param.handle()
    }
}

impl Param {
    pub fn handle(&self) -> ParamHandle {
        ParamHandle {
            value: self.0.clone(),
        }
    }
}

impl Node<0, 1> for Param {
    const INPUT_NAMES: [&'static str; 0] = [];
    const OUTPUT_NAMES: [&'static str; 1] = ["value"];
    fn process(&mut self, _: [f32; 0], output: &mut [f32; 1]) {
        output[0] = self.0.load(std::sync::atomic::Ordering::Relaxed);
    }
    fn new(_: u32) -> Self {
        Self(Arc::new(AtomicF32::new(0.0)))
    }
}
