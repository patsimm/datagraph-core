pub trait Source {
    fn output(&mut self, sample_num: usize) -> f32;
}

pub trait Effect {
    fn process(&mut self, input: f32, sample_num: usize) -> f32;
}
