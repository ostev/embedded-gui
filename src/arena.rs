use core::{marker::PhantomData, ops, ptr::NonNull};

use alloc::alloc;

type Index = u16;

pub struct Box<T> {
    offset: u16,
    phantom_type: PhantomData<T>,
}

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
    pub fn set_capacity(&mut self, new_capacity: Index) {
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
        self.set_capacity(self.length + 20);
    }

    pub fn push(&mut self, element: T) -> Box<T> {
        if self.length == self.capacity {
            self.grow();
        }

        unsafe {
            core::ptr::write(self.ptr.as_ptr().add(self.length.into()), element);
        }

        let arena_ref = Box {
            offset: self.length,
            phantom_type: PhantomData,
        };

        // We'll run out of memory before overflowing
        self.length += 1;

        arena_ref
    }

    pub fn checked_push(&mut self, element: T) -> Option<Box<T>> {
        self.length.checked_add(1).map(|_| self.push(element))
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, ptr: Box<T>) -> &T {
        unsafe { &*self.ptr.as_ptr().add(ptr.offset.into()) }
    }

    pub fn get(&self, ptr: Box<T>) -> Option<&T> {
        if ptr.offset < self.length {
            Some(unsafe { self.get_unchecked(ptr) })
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn get_mut_unchecked(&mut self, ptr: Box<T>) -> &mut T {
        unsafe { &mut *self.ptr.as_ptr().add(ptr.offset.into()) }
    }

    pub fn get_mut(&mut self, ptr: Box<T>) -> Option<&mut T> {
        if ptr.offset < self.length {
            Some(unsafe { self.get_mut_unchecked(ptr) })
        } else {
            None
        }
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

impl<T> ops::Index<Box<T>> for Arena<T> {
    type Output = T;

    fn index(&self, index: Box<T>) -> &Self::Output {
        self.get(index).expect("Index out of bounds")
    }
}

impl<T> ops::IndexMut<Box<T>> for Arena<T> {
    fn index_mut(&mut self, index: Box<T>) -> &mut Self::Output {
        self.get_mut(index).expect("Index out of bounds")
    }
}
