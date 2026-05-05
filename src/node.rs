use crate::graph::Node;

pub trait Source {
    fn generate(&mut self, sample_num: usize) -> f32;
}

impl<T: Source> Node<0, 1> for T {
    fn process(&mut self, _: [f32; 0], sample_num: usize) -> [f32; 1] {
        [self.generate(sample_num)]
    }
}

pub trait Effect {
    fn process(&mut self, input: f32, sample_num: usize) -> f32;
}

impl<T: Effect> Node<1, 1> for T {
    fn process(&mut self, input: [f32; 1], sample_num: usize) -> [f32; 1] {
        [self.process(input[0], sample_num)]
    }
}
