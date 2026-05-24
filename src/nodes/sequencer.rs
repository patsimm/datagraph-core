use crate::graph::Node;

#[derive(Default)]
pub struct Sequencer {
    index: usize,
    gate_on: bool,
}

impl Node<5, 1> for Sequencer {
    const INPUT_NAMES: [&'static str; 5] = ["gate", "1", "2", "3", "4"];
    const OUTPUT_NAMES: [&'static str; 1] = ["output"];
    fn process(&mut self, inputs: [f32; 5]) -> [f32; 1] {
        let curr_gate_on = inputs[0] > 0.5;
        if curr_gate_on && !self.gate_on {
            self.index = (self.index + 1) % 4;
        }
        self.gate_on = curr_gate_on;

        [inputs[self.index + 1]]
    }
    fn new(_: u32) -> Self {
        Default::default()
    }
}
