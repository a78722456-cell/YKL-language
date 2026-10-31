use crossbeam_channel::{Receiver as CbReceiver, Sender as CbSender, unbounded};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread;

// Arena Allocator implementation
pub struct ArenaAllocator {
    chunks: RefCell<Vec<Vec<u8>>>,
    chunk_size: usize,
}

impl ArenaAllocator {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunks: RefCell::new(vec![Vec::with_capacity(chunk_size)]),
            chunk_size,
        }
    }

    pub fn alloc<T>(&self, val: T) -> &mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        let mut chunks = self.chunks.borrow_mut();
        let chunk = chunks.last_mut().unwrap();

        let cur_len = chunk.len();
        let offset = (cur_len + align - 1) & !(align - 1);

        if offset + size > chunk.capacity() {
            let new_cap = self.chunk_size.max(size);
            chunks.push(Vec::with_capacity(new_cap));
            let new_chunk = chunks.last_mut().unwrap();
            unsafe {
                new_chunk.set_len(size);
                let ptr = new_chunk.as_mut_ptr() as *mut T;
                std::ptr::write(ptr, val);
                &mut *ptr
            }
        } else {
            unsafe {
                chunk.set_len(offset + size);
                let ptr = chunk.as_mut_ptr().add(offset) as *mut T;
                std::ptr::write(ptr, val);
                &mut *ptr
            }
        }
    }

    pub fn reset(&self) {
        let mut chunks = self.chunks.borrow_mut();
        for c in chunks.iter_mut() {
            c.clear();
        }
    }
}

unsafe impl Send for ArenaAllocator {}
unsafe impl Sync for ArenaAllocator {}

// Bump Allocator
pub struct BumpAllocator {
    buffer: Vec<u8>,
    offset: Mutex<usize>,
}

impl BumpAllocator {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0; capacity],
            offset: Mutex::new(0),
        }
    }

    pub fn alloc_bytes(&self, size: usize) -> Option<*mut u8> {
        let mut offset = self.offset.lock().unwrap();
        if *offset + size <= self.buffer.len() {
            let ptr = unsafe { self.buffer.as_ptr().add(*offset) as *mut u8 };
            *offset += size;
            Some(ptr)
        } else {
            None
        }
    }

    pub fn reset(&self) {
        let mut offset = self.offset.lock().unwrap();
        *offset = 0;
    }
}

// Actor / Channel model
#[derive(Clone)]
pub struct Sender<T>(CbSender<T>);

#[derive(Clone)]
pub struct Receiver<T>(CbReceiver<T>);

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = unbounded();
    (Sender(tx), Receiver(rx))
}

impl<T> Sender<T> {
    pub fn send(&self, msg: T) {
        let _ = self.0.send(msg);
    }
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Option<T> {
        self.0.recv().ok()
    }
    pub fn try_recv(&self) -> Option<T> {
        self.0.try_recv().ok()
    }
}

// Work-stealing Green Thread / Task Scheduler
pub type Task = Box<dyn FnOnce() + Send + 'static>;

#[derive(Clone)]
pub struct Scheduler {
    sender: CbSender<Task>,
}

impl Scheduler {
    pub fn new(num_threads: usize) -> Self {
        let (tx, rx) = unbounded::<Task>();
        let rx = Arc::new(rx);

        for _ in 0..num_threads {
            let rx_clone = Arc::clone(&rx);
            thread::spawn(move || {
                while let Ok(task) = rx_clone.recv() {
                    task();
                }
            });
        }

        Self { sender: tx }
    }

    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let _ = self.sender.send(Box::new(f));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_allocator() {
        let arena = ArenaAllocator::new(1024);
        let val1 = arena.alloc(42i32);
        let val2 = arena.alloc(100i64);
        assert_eq!(*val1, 42);
        assert_eq!(*val2, 100);
        arena.reset();
    }

    #[test]
    fn test_scheduler_and_channels() {
        let sched = Scheduler::new(2);
        let (tx, rx) = channel::<i32>();

        for i in 0..5 {
            let tx_clone = tx.clone();
            sched.spawn(move || {
                tx_clone.send(i * 10);
            });
        }

        let mut sum = 0;
        for _ in 0..5 {
            sum += rx.recv().unwrap();
        }
        assert_eq!(sum, 0 + 10 + 20 + 30 + 40);
    }
}
