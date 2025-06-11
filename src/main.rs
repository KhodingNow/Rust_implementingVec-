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
// As a recap, Unique is a Wrapper around a raw ppointer that declares that:
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
            let new new_layout = Layout::array::<T>(new_cap).unwrap();
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

        // If allocation fails, new_ptr will be nuull, in which case we abort
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
}