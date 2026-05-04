use crate::ring_buffer::RingBuffer;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    NoteOn,
    NoteOff,
}

pub type EventBuffer = RingBuffer<Event, 16>;
