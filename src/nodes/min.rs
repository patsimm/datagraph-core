use crate::graph::Node;

pub struct Min;

impl Node<2, 1> for Min {
    const INPUT_NAMES: [&'static str; 2] = ["input1", "input2"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 2], output: &mut [f32; 1]) {
        output[0] = input[0].min(input[1]);
    }
    fn new(_: u32) -> Self {
        Self
    }
}
