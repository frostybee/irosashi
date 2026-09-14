use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNT: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);

/// A system allocator that counts allocations, for the allocation audit. Install it
/// in a bench or test binary with `#[global_allocator]`.
pub struct CountingAlloc;

/// Allocation count and bytes requested so far.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocSnapshot {
    pub count: u64,
    pub bytes: u64,
}

impl AllocSnapshot {
    pub fn take() -> Self {
        Self {
            count: COUNT.load(Ordering::Relaxed),
            bytes: BYTES.load(Ordering::Relaxed),
        }
    }

    pub fn since(self, earlier: AllocSnapshot) -> AllocSnapshot {
        AllocSnapshot {
            count: self.count - earlier.count,
            bytes: self.bytes - earlier.bytes,
        }
    }
}

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        COUNT.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        COUNT.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// Runs `f` and returns its result with the allocations it made.
pub fn measure<T>(f: impl FnOnce() -> T) -> (T, AllocSnapshot) {
    let before = AllocSnapshot::take();
    let value = f();
    (value, AllocSnapshot::take().since(before))
}
