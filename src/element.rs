use core::{marker::PhantomData, ptr::NonNull};

use alloc::alloc;

pub trait Draw {
    fn draw(&self) {}
}

struct ElementPointer<T> {
    offset: u16,
    phantom: PhantomData<T>,
}

impl<T> ElementPointer<T> {
    pub fn new(element: T) {}
}

struct ElementArena<T> {
    ptr: NonNull<T>,
    capacity: usize,
    length: usize,
}

unsafe impl<T: Send> Send for ElementArena<T> {}
unsafe impl<T: Sync> Sync for ElementArena<T> {}

impl<T> ElementArena<T> {
    pub fn new() -> Self {
        assert!(core::mem::size_of::<T>() == 0);
        Self {
            ptr: NonNull::dangling(),
            length: 0,
            capacity: 0,
        }
    }

    /// Reallocate the arena with a new capacity.
    /// If the new capacity is less than the existing number
    /// of items in the arena, those items will be discarded.
    pub fn realloc(&mut self, new_capacity: usize) {
        let new_layout = alloc::Layout::array::<T>(new_capacity).unwrap();

        // Ensure that the new allocation is not larger than `isize::MAX` bytes,
        // since allocations larger than this cause issues with LLVM's GEP instruction.
        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation size exceeds `isize::MAX` bytes"
        );

        let new_ptr = if self.capacity > 0 {
            let old_layout = alloc::Layout::array::<T>(self.capacity).unwrap();
            unsafe { alloc::realloc(self.ptr.as_ptr() as *mut u8, old_layout, new_layout.size()) }
        } else {
            unsafe { alloc::alloc(new_layout) }
        };

        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(new_layout),
        };

        self.capacity = new_capacity;
        self.length = self.length.min(self.capacity);
    }

    pub fn len(&self) -> usize {
        self.length
    }
}

enum ArenaAllocationError {}
