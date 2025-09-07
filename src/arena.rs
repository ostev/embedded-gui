use core::{marker::PhantomData, ptr::NonNull};

use alloc::alloc;

type Index = u16;

pub trait Draw {
    fn draw(&self) {}
}

pub struct Ref<T> {
    offset: u16,
    phantom: PhantomData<T>,
}

impl<T> Ref<T> {}

pub struct Arena<T> {
    ptr: NonNull<T>,
    capacity: Index,
    length: Index,
}

unsafe impl<T: Send> Send for Arena<T> {}
unsafe impl<T: Sync> Sync for Arena<T> {}

impl<T> Arena<T> {
    const STARTING_CAPACITY: Index = if core::mem::size_of::<T>() == 0 {
        Index::MAX
    } else {
        0
    };

    pub fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            length: 0,
            capacity: Self::STARTING_CAPACITY,
        }
    }

    /// Reallocate the arena with a new capacity.
    /// If the new capacity is less than the existing number
    /// of items in the arena, those items will be discarded.
    pub fn realloc(&mut self, new_capacity: Index) {
        if core::mem::size_of::<T>() != 0 {
            let new_layout = alloc::Layout::array::<T>(new_capacity.into()).unwrap();

            // Ensure that the new allocation is not larger than `isize::MAX` bytes,
            // since allocations larger than this cause issues with LLVM's GEP instruction.
            assert!(
                new_layout.size() <= isize::MAX as usize,
                "Allocation size exceeds `isize::MAX` bytes"
            );

            let new_ptr = if self.capacity != 0 {
                let old_layout = alloc::Layout::array::<T>(self.capacity.into()).unwrap();
                unsafe {
                    alloc::realloc(self.ptr.as_ptr() as *mut u8, old_layout, new_layout.size())
                }
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
    }

    fn grow(&mut self) {
        assert!(
            core::mem::size_of::<T>() != 0,
            "The arena is at maximum capacity."
        );
        self.realloc(self.length + 20);
    }

    pub fn push(&mut self, element: T) -> Ref<T> {
        if self.length == self.capacity {
            self.grow();
        }

        unsafe {
            core::ptr::write(self.ptr.as_ptr().add(self.length.into()), element);
        }

        let arena_ref = Ref {
            offset: self.length,
            phantom: PhantomData,
        };

        // We'll run out of memory before overflowing
        self.length += 1;

        arena_ref
    }

    pub fn checked_push(&mut self, element: T) -> Option<Ref<T>> {
        self.length.checked_add(1).map(|_| self.push(element))
    }
}

impl<T> Drop for Arena<T> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            let layout = alloc::Layout::array::<T>(self.capacity.into()).unwrap();

            for element in self.iter_mut() {
                unsafe {
                    core::ptr::drop_in_place(element);
                }
            }

            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

impl<T> core::ops::Deref for Arena<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.length.into()) }
    }
}

impl<T> core::ops::DerefMut for Arena<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.length.into()) }
    }
}
