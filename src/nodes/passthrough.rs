use crate::graph::Node;

pub struct Passthrough;

impl Node<1, 1> for Passthrough {
    const INPUT_NAMES: [&'static str; 1] = ["input"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 1]) -> [f32; 1] {
        input
    }
    fn new(_: u32) -> Self {
        Self
    }
}
