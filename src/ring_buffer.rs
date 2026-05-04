use std::{cell::UnsafeCell, sync::atomic::AtomicUsize};

pub struct RingBuffer<T: Copy, const N: usize> {
    events: [UnsafeCell<Option<T>>; N],
    read_index: AtomicUsize,
    write_index: AtomicUsize,
}

unsafe impl<T: Copy, const N: usize> Send for RingBuffer<T, N> {}
unsafe impl<T: Copy, const N: usize> Sync for RingBuffer<T, N> {}

impl<T: Copy, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            events: std::array::from_fn(|_| UnsafeCell::new(None)),
            read_index: AtomicUsize::new(0),
            write_index: AtomicUsize::new(0),
        }
    }

    pub fn flooded(self, val: T) -> Self {
        while self.push(val).is_ok() {}
        self
    }

    pub fn push(&self, event: T) -> Result<(), ()> {
        let write_index = self.write_index.load(std::sync::atomic::Ordering::Relaxed);
        let next_write_index = (write_index + 1) % self.events.len();

        if next_write_index == self.read_index.load(std::sync::atomic::Ordering::Acquire) {
            return Err(()); // Buffer is full
        }

        unsafe { *self.events[write_index].get() = Some(event) };

        self.write_index
            .store(next_write_index, std::sync::atomic::Ordering::Release);
        Ok(())
    }

    pub fn pop(&self) -> Option<T> {
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
