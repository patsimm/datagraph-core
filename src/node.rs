pub trait Source {
    fn output(&mut self) -> f32;
}

pub trait Effect {
    fn process(&mut self, input: f32) -> f32;
}
