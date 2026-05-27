use std::{cell::UnsafeCell, sync::atomic::AtomicUsize};

pub struct RingBuffer<T: Copy> {
    buffer: Box<[UnsafeCell<Option<T>>]>,
    read_index: AtomicUsize,
    write_index: AtomicUsize,
}

unsafe impl<T: Copy> Send for RingBuffer<T> {}
unsafe impl<T: Copy> Sync for RingBuffer<T> {}

impl<T: Copy> RingBuffer<T> {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: (0..size).map(|_| UnsafeCell::new(None)).collect(),
            read_index: AtomicUsize::new(0),
            write_index: AtomicUsize::new(0),
        }
    }

    pub fn flooded(self, val: T) -> Self {
        while self.push(val).is_ok() {}
        self
    }

    pub fn drain(&self) -> Vec<T> {
        let mut events = Vec::new();
        while let Some(event) = self.pop() {
            events.push(event);
        }
        events
    }

    pub fn push(&self, value: T) -> Result<(), ()> {
        let write_index = self.write_index.load(std::sync::atomic::Ordering::Relaxed);
        let next_write_index = (write_index + 1) % self.buffer.len();

        if next_write_index == self.read_index.load(std::sync::atomic::Ordering::Acquire) {
            return Err(()); // Buffer is full
        }

        unsafe { *self.buffer[write_index].get() = Some(value) };

        self.write_index
            .store(next_write_index, std::sync::atomic::Ordering::Release);
        Ok(())
    }

    pub fn pop(&self) -> Option<T> {
        let read_index = self.read_index.load(std::sync::atomic::Ordering::Relaxed);
        if read_index == self.write_index.load(std::sync::atomic::Ordering::Acquire) {
            return None;
        }
        let opt_event = unsafe { *self.buffer[read_index].get() };
        if let Some(event) = opt_event {
            self.read_index.store(
                (read_index + 1) % self.buffer.len(),
                std::sync::atomic::Ordering::Release,
            );
            Some(event)
        } else {
            None
        }
    }

    pub fn move_read_index(&self, offset: isize) {
        self.read_index
            .try_update(
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
                |current| {
                    let new_index =
                        (current as isize + offset).rem_euclid(self.buffer.len() as isize) as usize;
                    Some(new_index)
                },
            )
            .ok();
    }
}
