use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use crate::graph::PortKey;

const MAX_SUBSCRIPTIONS: usize = 64;
pub const BUFFER_SIZE: usize = 2048;
const STRIDE: usize = 1 + BUFFER_SIZE; // generation counter + samples

struct Buffer {
    data: Box<[AtomicU32]>,
}

// SAFETY: all accesses go through AtomicU32
unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

impl Buffer {
    fn new() -> Self {
        Self {
            data: (0..MAX_SUBSCRIPTIONS * STRIDE)
                .map(|_| AtomicU32::new(0))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }
}

pub struct NodeDataWriter {
    buffer: Arc<Buffer>,
    subscriptions: Vec<(PortKey, usize)>,
    accumulators: Vec<Box<[f32]>>,
    accumulator_pos: Vec<usize>,
    version: u64,
}

pub struct NodeDataReader {
    buffer: Arc<Buffer>,
}

pub fn node_data_channel() -> (NodeDataWriter, NodeDataReader) {
    let buf = Arc::new(Buffer::new());
    (
        NodeDataWriter {
            buffer: Arc::clone(&buf),
            subscriptions: Vec::new(),
            accumulators: (0..MAX_SUBSCRIPTIONS)
                .map(|_| vec![0.0_f32; BUFFER_SIZE].into_boxed_slice())
                .collect(),
            accumulator_pos: vec![0; MAX_SUBSCRIPTIONS],
            version: 0,
        },
        NodeDataReader { buffer: buf },
    )
}

impl NodeDataWriter {
    pub fn subscribe(&mut self, port: PortKey, index: usize) {
        self.subscriptions.retain(|(_, i)| *i != index);
        self.subscriptions.push((port, index));
        self.accumulator_pos[index] = 0;
        self.version = self.version.wrapping_add(1);
    }

    pub fn unsubscribe(&mut self, index: usize) {
        self.subscriptions.retain(|(_, i)| *i != index);
        self.accumulator_pos[index] = 0;
        self.version = self.version.wrapping_add(1);
    }

    pub fn subscriptions(&self) -> &[(PortKey, usize)] {
        &self.subscriptions
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn write_batches<'a>(&mut self, batches: impl Iterator<Item = &'a [f32]>) {
        for ((_, slot), samples) in self.subscriptions.iter().zip(batches) {
            let slot = *slot;
            let pos = self.accumulator_pos[slot];
            let to_write = samples.len().min(BUFFER_SIZE - pos);
            self.accumulators[slot][pos..pos + to_write].copy_from_slice(&samples[..to_write]);
            self.accumulator_pos[slot] += to_write;

            if self.accumulator_pos[slot] >= BUFFER_SIZE {
                let base = slot * STRIDE;
                for (i, &v) in self.accumulators[slot].iter().enumerate() {
                    self.buffer.data[base + 1 + i].store(v.to_bits(), Ordering::Relaxed);
                }
                // Release ordering ensures sample writes are visible before the counter increments
                self.buffer.data[base].fetch_add(1, Ordering::Release);
                self.accumulator_pos[slot] = 0;
            }
        }
    }
}

impl NodeDataReader {
    pub fn buffer_ptr(&self) -> usize {
        self.buffer.data.as_ptr() as usize
    }

    pub fn buffer_stride(&self) -> usize {
        STRIDE
    }
}
