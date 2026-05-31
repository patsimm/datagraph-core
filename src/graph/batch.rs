use crate::graph::port::PortKey;

#[derive(Default)]
pub struct BatchBuffer {
    batchsize: usize,
    buffer: Vec<f32>,
}

impl<'a> IntoIterator for &'a BatchBuffer {
    type Item = &'a [f32];
    type IntoIter = std::slice::Chunks<'a, f32>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.chunks(self.batchsize)
    }
}

impl BatchBuffer {
    pub(super) fn resize(&mut self, ports: &[PortKey], batchsize: usize) {
        self.batchsize = batchsize;
        self.buffer.resize(ports.len() * batchsize, 0.0);
    }

    pub(super) fn set_value(&mut self, port_index: usize, sample_index: usize, value: f32) {
        self.buffer[port_index * self.batchsize + sample_index] = value;
    }
}
