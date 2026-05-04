use std::{
    cell::UnsafeCell,
    sync::{Arc, atomic::AtomicUsize},
};

#[derive(Debug, Clone, Copy)]
pub enum Event {
    NoteOn,
    NoteOff,
}

pub struct EventBuffer<const N: usize> {
    events: [UnsafeCell<Option<Event>>; N],
    read_index: AtomicUsize,
    write_index: AtomicUsize,
}

unsafe impl<const N: usize> Send for EventBuffer<N> {}
unsafe impl<const N: usize> Sync for EventBuffer<N> {}

impl<const N: usize> EventBuffer<N> {
    pub fn new() -> Self {
        Self {
            events: std::array::from_fn(|_| UnsafeCell::new(None)),
            read_index: AtomicUsize::new(0),
            write_index: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, event: Event) -> bool {
        let write_index = self.write_index.load(std::sync::atomic::Ordering::Relaxed);
        let next_write_index = (write_index + 1) % self.events.len();

        if next_write_index == self.read_index.load(std::sync::atomic::Ordering::Acquire) {
            return false;
        }

        unsafe { *self.events[write_index].get() = Some(event) };

        self.write_index
            .store(next_write_index, std::sync::atomic::Ordering::Release);
        true
    }

    pub fn pop(&self) -> Option<Event> {
        let read_index = self.read_index.load(std::sync::atomic::Ordering::Relaxed);
        if read_index == self.write_index.load(std::sync::atomic::Ordering::Acquire) {
            return None;
        }
        let opt_event = unsafe { *self.events[read_index].get() };
        if let Some(event) = opt_event {
            self.read_index.store(
                (read_index + 1) % self.events.len(),
                std::sync::atomic::Ordering::Release,
            );
            Some(event)
        } else {
            None
        }
    }
}
