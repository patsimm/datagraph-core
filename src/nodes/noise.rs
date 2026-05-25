use crate::graph::Node;

pub struct Noise;

impl Node<0, 1> for Noise {
    const INPUT_NAMES: [&'static str; 0] = [];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, _input: [f32; 0]) -> [f32; 1] {
        [rand::random::<f32>() * 2.0 - 1.0]
    }
    fn new(_: u32) -> Self {
        Self
    }
}
