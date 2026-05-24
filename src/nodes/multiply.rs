use crate::graph::Node;

pub struct Multiply;

impl Node<2, 1> for Multiply {
    const INPUT_NAMES: [&'static str; 2] = ["input1", "input2"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 2]) -> [f32; 1] {
        [input[0] * input[1]]
    }
}
