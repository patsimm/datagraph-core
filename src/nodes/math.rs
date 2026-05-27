use crate::graph::Node;

pub struct Add;
impl Node<2, 1> for Add {
    const INPUT_NAMES: [&'static str; 2] = ["input1", "input2"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 2], output: &mut [f32; 1]) {
        output[0] = input[0] + input[1];
    }
    fn new(_: u32) -> Self {
        Self
    }
}

pub struct Multiply;
impl Node<2, 1> for Multiply {
    const INPUT_NAMES: [&'static str; 2] = ["input1", "input2"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 2], output: &mut [f32; 1]) {
        output[0] = input[0] * input[1];
    }
    fn new(_: u32) -> Self {
        Self
    }
}

pub struct Max;
impl Node<2, 1> for Max {
    const INPUT_NAMES: [&'static str; 2] = ["input1", "input2"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 2], output: &mut [f32; 1]) {
        output[0] = input[0].max(input[1]);
    }
    fn new(_: u32) -> Self {
        Self
    }
}

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

pub struct Passthrough;
impl Node<1, 1> for Passthrough {
    const INPUT_NAMES: [&'static str; 1] = ["input"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, input: [f32; 1], output: &mut [f32; 1]) {
        output[0] = input[0];
    }
    fn new(_: u32) -> Self {
        Self
    }
}
