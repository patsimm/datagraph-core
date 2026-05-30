use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use crate::graph::{PortKey, PortValueAccess};

const MAX_SUBSCRIPTIONS: usize = 2048;

struct Buffer {
    slots: Box<[AtomicU32]>,
}

// SAFETY: all accesses go through AtomicU32
unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

impl Buffer {
    fn new() -> Self {
        Self {
            slots: (0..MAX_SUBSCRIPTIONS)
                .map(|_| AtomicU32::new(0))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    fn write(&self, index: usize, value: f32) {
        if index < MAX_SUBSCRIPTIONS {
            self.slots[index].store(value.to_bits(), Ordering::Relaxed);
        }
    }
}

pub struct LatestValueWriter {
    buffer: Arc<Buffer>,
    subscriptions: Vec<(PortKey, usize)>,
}

pub struct LatestValueReader {
    buffer: Arc<Buffer>,
}

pub fn latest_value_channel() -> (LatestValueWriter, LatestValueReader) {
    let buf = Arc::new(Buffer::new());
    (
        LatestValueWriter {
            buffer: Arc::clone(&buf),
            subscriptions: Vec::new(),
        },
        LatestValueReader { buffer: buf },
    )
}

impl LatestValueWriter {
    pub fn subscribe(&mut self, port: PortKey, index: usize) {
        self.subscriptions.retain(|(_, i)| *i != index);
        self.subscriptions.push((port, index));
    }

    pub fn unsubscribe(&mut self, index: usize) {
        self.subscriptions.retain(|(_, i)| *i != index);
    }

    pub fn write_from_graph(&self, graph: &impl PortValueAccess) {
        for (port, index) in &self.subscriptions {
            if let Some(&v) = graph.port_value(port) {
                self.buffer.write(*index, v);
            }
        }
    }
}

impl LatestValueReader {
    pub fn buffer_ptr(&self) -> usize {
        self.buffer.slots.as_ptr() as usize
    }
}
