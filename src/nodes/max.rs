use crate::graph::Node;

pub struct Max;

impl Node<2, 1> for Max {
    const INPUT_NAMES: [&'static str; 2] = ["input1", "input2"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 2]) -> [f32; 1] {
        [input[0].max(input[1])]
    }
    fn new(_: u32) -> Self {
        Self
    }
}
