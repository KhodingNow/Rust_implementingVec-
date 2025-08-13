// LayOut of a Vec - has 3 parts:
// - a pointer to the allocation -> the size of the allocation -> number of elemnets initialized

fn struct Vec<T> {
    ptr: *mut T,
    cap: usize,
    len: usize,
} // this would compile - but the compiler wld be very strict
// a &Vec<'a static str> couldn't be used where a &Vec<'a str> is expected bcs of the details on variance
// the std lib uses Unique<T> in place of *mut T when it has a raw pointer to an allocation that it owns
// Unique is unstable, so we'd like not to use it if possible.
// As a recap, Unique is a Wrapper around a raw pointer that declares that:
// - We are covariant over T
// - We are Send/Sync if T is Send/Sync
// - Our pointer is never null (so Option<Vec<T>> is null-pointer-optimized)

// We can implement all of this in Stable Rust - to do so, we need NoNull<t> instead of Unique<T> - another Wrapper around a raw pointer

use std::ptr::NoNull;

pub struct Vec<T> {
    ptr: NoNull<T>,
    cap: usize,
    len: usize,
}

unsafe impl<T: Send> Send for Vec<T> {}   
unsafe impl<T: Sync> Sync for Vec<T> {}

// Allocating Memory

use std::alloc::{self, layout};

impl<T> Vec<T> {
    fn grow(&mut self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else{
            // This can't overflow since self.cap <= isize::MAX,
            let new_cap = 2 * self.cap;

            // "Layout::array" checks that the number of bytes is <= usize::MAX,
            // but this is redundant since old_layout.size() <= isize::MAX,
            // so the 'unwrap' should never fail.
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        // Ensure that the new allocation doesn't exceed 'isize::MAX' bytes
        assert!(new_layout.size() <= isize::MAX as usize, "Allocatio too large");

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        // If allocation fails, new_ptr will be null, in which case we abort
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap; 
    }

    // PUSH and POP:

    pub fn push(&mut self, elem: T) {
        if self.len == self.cap { self.grow(); }

        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), elem);
        }

        // Can't fail, we'll OOM first.
        self.len += 1;
    }

    // IntoIter for Vec iteration

    pub struct IntoIter<T> {
        buf: NonNull<T>,
        cap: usize,
        start: *const T,
        end: *const T,
    }

    // We end up with this for initialization:

    impl<T> IntoIterator for Vec<T> {
        type item = T;
        type IntoIter = IntoIter<T> {
            // Make sure not to drop Vec since that wld free the buffer
            let vec = ManuallyDrop::new(self);

            // Can't destructure Vec since it's Drop
            let ptr = vec.ptr;
            let cap = vec.cap;
            let len = vec.len;

            IntoIter {
                buf: ptr,
                cap,
                start: ptr.as_ptr(),
                end: if cap == 0 {
                    // can't offset this pointer, its not allocated
                    ptr.as_ptr()
                } else {
                    unsafe { ptr.as_ptr().add(len) }
                },
            }
        }
    }

    // Here's iterating forward:

    impl<T> Iterator for IntoIter<T> {
        type Item = T;
        fn next(&mut self) -> Option<T> {
            if self.start == self.end {
                None
            } else {
                unsafe {
                    let result = ptr::read(self.start);
                    self.start = self.start.offset(1);
                    Some(result)
                }
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = (self.end as usize - self.start as usize)
                        / mem::size_of::<T>();
            (len, Some(len))
        }

    }

    // Here's iterating backWards.

    impl<T> DoubleEndedIterator for IntoIter<T> {
        fn next_back(&mut self) -> Option<T> {
            if self.start == self.end {
                None
            } else {
                unsafe {
                    self.end = self.offset(-1);
                    Some(ptr::read(self.end))
                }
            }
        }
    }
    // IntoIter takes ownership of its allocation, it needs to implement Drop to free it.
    // However, it also want to implement Drop to any elements it contains that were not yielded.

    impl<T> Drop for IntoIter<T> {
        fn drop(&mut self) {
            if self.cap != 0 {
                // drop any remaining elements

                for _ in &mut *self {}
                let layout = Layout::array::<T>(self.cap).unwrap();
                unsafe {
                    alloc::dealloc(self.buf.as_ptr() as *mut u8, layout);
                }
            }
        }
    }

} 
