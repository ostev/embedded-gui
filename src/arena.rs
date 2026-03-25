use core::{
    marker::{PhantomContravariantLifetime, PhantomCovariantLifetime, PhantomData},
    ops::{self, Deref},
    ptr::NonNull,
};

use alloc::alloc;

type Index = u16;

// #[derive(Clone, Copy, Debug)]
// pub struct Box<'a, T> {
//     offset: u16,
//     phantom: PhantomData<&'a T>,
// }

// impl<'a, T> Into<UntypedPointer<'a>> for Box<'a, T> {
//     #[inline(always)]
//     fn into(self) -> UntypedPointer<'a> {
//         UntypedPointer {
//             offset: self.offset,
//             phantom: PhantomCovariantLifetime::new(),
//         }
//     }
// }

#[derive(Copy, Clone)]
pub struct UntypedPointer<'a> {
    offset: Index,
    phantom: PhantomCovariantLifetime<'a>,
}

#[derive(Clone, Copy)]
pub struct RangePointer<'a> {
    start: Index,
    length: Index,
    phantom: PhantomCovariantLifetime<'a>,
}

#[derive(Clone)]
pub struct Arena<'a, T> {
    ptr: NonNull<T>,
    capacity: Index,
    length: Index,
    lifetime: PhantomContravariantLifetime<'a>,
}

impl<'a, T> Arena<'a, T> {
    const STARTING_CAPACITY: Index = if core::mem::size_of::<T>() == 0 {
        Index::MAX
    } else {
        0
    };

    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            length: 0,
            capacity: Self::STARTING_CAPACITY,
            lifetime: PhantomContravariantLifetime::new(),
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

    fn grow(&mut self, increase: Index) {
        assert!(
            core::mem::size_of::<T>() != 0,
            "The arena is at maximum capacity."
        );
        self.set_capacity(self.length + increase);
    }

    pub fn alloc(&mut self, element: T) -> UntypedPointer<'a> {
        if self.length == self.capacity {
            self.grow(20);
        }

        unsafe {
            core::ptr::write(self.ptr.as_ptr().add(self.length.into()), element);
        }

        let arena_ref = UntypedPointer {
            offset: self.length,
            phantom: PhantomCovariantLifetime::new(),
        };

        // We'll run out of memory before overflowing
        self.length += 1;

        arena_ref
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, ptr: UntypedPointer<'a>) -> &'a T {
        unsafe { &*self.ptr.as_ptr().add(ptr.offset.into()) }
    }

    pub fn get(&self, ptr: UntypedPointer<'a>) -> Option<&'a T> {
        if ptr.offset < self.length {
            Some(unsafe { self.get_unchecked(ptr) })
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn get_mut_unchecked(&mut self, boxed: UntypedPointer<'a>) -> &'a mut T {
        unsafe { &mut *self.ptr.as_ptr().add(boxed.offset.into()) }
    }

    pub fn get_mut(&mut self, ptr: UntypedPointer<'a>) -> Option<&'a mut T> {
        if ptr.offset < self.length {
            Some(unsafe { self.get_mut_unchecked(ptr) })
        } else {
            None
        }
    }
}

impl<'a, T> Drop for Arena<'a, T> {
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

impl<'a, T> core::ops::Deref for Arena<'a, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.length.into()) }
    }
}

impl<'a, T> core::ops::DerefMut for Arena<'a, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.length.into()) }
    }
}

impl<'a, T: 'a> ops::Index<UntypedPointer<'a>> for Arena<'a, T> {
    type Output = T;

    fn index(&self, index: UntypedPointer<'a>) -> &Self::Output {
        self.get(index).expect("Index out of bounds")
    }
}

impl<'a, T: 'a> ops::IndexMut<UntypedPointer<'a>> for Arena<'a, T> {
    fn index_mut(&mut self, index: UntypedPointer<'a>) -> &'a mut Self::Output {
        self.get_mut(index).expect("Index out of bounds")
    }
}
