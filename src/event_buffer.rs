use crate::ring_buffer::RingBuffer;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    NoteOn { frequency: f32 },
    NoteOff,
}

pub type EventBuffer = RingBuffer<Event, 16>;
