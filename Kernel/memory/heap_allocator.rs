use core::{
    alloc::GlobalAlloc,
    ptr::null_mut,
    sync::atomic::{AtomicUsize, Ordering},
};

pub const HEAP_START: usize = 0x1000_0000_0000;
pub const HEAP_SIZE: usize = 100 * 1024;
#[global_allocator]
static ALLOCATOR: HeapAllocator = HeapAllocator::new(HEAP_START, HEAP_START + HEAP_SIZE);

pub struct HeapAllocator {
    heap_start: usize,
    heap_end: usize,
    next: AtomicUsize,
}

impl HeapAllocator {
    pub const fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            heap_start,
            heap_end,
            next: AtomicUsize::new(heap_start),
        }
    }
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        loop {
            // load current state of the `next` field
            let current_next = self.next.load(Ordering::Relaxed);
            let alloc_start = align_up(current_next, layout.align());
            let alloc_end = alloc_start.saturating_add(layout.size());

            if alloc_end <= self.heap_end {
                // update the `next` pointer if it still has the value `current_next`
                let next_now = self
                    .next
                    .compare_exchange(
                        current_next,
                        alloc_end,
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    )
                    .unwrap();
                if next_now == current_next {
                    // next address was successfully updated, allocation succeeded
                    return alloc_start as *mut u8;
                }
            } else {
                return null_mut();
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        // TODO: don't leak memory
    }
}

/// Align downwards. Returns the greatest x with alignment `align`
/// so that x <= addr. The alignment must be a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

/// Align upwards. Returns the smallest x with alignment `align`
/// so that x >= addr. The alignment must be a power of 2.
pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}
