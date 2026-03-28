use core::{
    cell::Cell,
    marker::{PhantomCovariantLifetime, PhantomData},
    ptr::NonNull,
};

use alloc::alloc;

pub struct Box<'a, T> {
    ptr: NonNull<T>,
    phantom: PhantomCovariantLifetime<'a>,
}

/**
 A resizable arena allocator that can store elements of any type.

 Note that the arena does not drop its contents when it is deallocated, so don't store
 any heap-allocated types in here and expect them to free their memory when the arena
 goes out of scope. However, destructors will run if a boxed type returned by `Arena::alloc`
 is dropped, though its memory won't be deallocated until the arena is dropped.
*/
#[derive(Clone)]
pub struct Arena {
    ptr: NonNull<u8>,
    capacity: usize,
    length: usize,
}

impl Arena {
    const STARTING_CAPACITY: usize = 0;
    const SIZE_INCREMENT_BYTES: usize = 128;

    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            length: 0,
            capacity: 0,
        }
    }

    /// Reallocate the arena with a new capacity.
    /// If the new capacity is less than the existing number
    /// of items in the arena, those items will be discarded, leaving dangling pointers.
    unsafe fn set_capacity(&mut self, new_capacity: usize) {
        let new_layout = alloc::Layout::array::<u8>(new_capacity.into()).unwrap();

        // Ensure that the new allocation is not larger than `isize::MAX` bytes,
        // since allocations larger than this cause issues with LLVM's GEP instruction.
        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation size exceeds `isize::MAX` bytes"
        );

        let new_ptr = if self.capacity != 0 {
            let old_layout = alloc::Layout::array::<u8>(self.capacity.get().into()).unwrap();
            unsafe { alloc::realloc(self.ptr.as_ptr() as *mut u8, old_layout, new_layout.size()) }
        } else {
            unsafe { alloc::alloc(new_layout) }
        };

        self.ptr = match NonNull::new(new_ptr as *mut u8) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(new_layout),
        };

        self.capacity = new_capacity;
        self.length = self.length.min(self.capacity);
    }

    pub fn grow(&mut self, increase: usize) {
        assert!(
            core::mem::size_of::<u8>() != 0,
            "The arena is at maximum capacity."
        );
        unsafe { self.set_capacity(self.length + increase) };
    }

    pub fn alloc<'a, T>(&'a mut self, element: T) -> Box<'a, T> {
        let element_size = core::mem::size_of::<T>();

        if element_size == 0 {
            Box {
                ptr: NonNull::dangling(),
                phantom: PhantomCovariantLifetime::new(),
            }
        } else {
            let offset = self.ptr.align_offset(core::mem::align_of::<T>());
            let alloc_size = element_size + offset;

            if self.length + alloc_size >= self.capacity {
                self.grow(element_size + Self::SIZE_INCREMENT_BYTES);
            }

            let element_ptr = unsafe {
                let ptr = self.ptr.as_ptr().add(self.length + offset) as *mut T;

                core::ptr::write(ptr, element);

                NonNull::new_unchecked(ptr)
            };

            // We'll run out of memory before overflowing
            self.length += alloc_size;

            Box {
                ptr: element_ptr,
                phantom: PhantomCovariantLifetime::new(),
            }
        }
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        let layout = alloc::Layout::array::<u8>(self.capacity).unwrap();

        unsafe {
            self.ptr.drop_in_place();
            alloc::dealloc(self.ptr.as_ptr(), layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_one_element() {
        let mut arena = Arena::new();
        let x = arena.alloc(5);
        let y = arena.alloc(7);
    }
}
