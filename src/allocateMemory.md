Using NonNull throws a spanner in an important feature of Vec(and indeed al of the std collections): creating an empty Vec doesn't actually allocate at all. This is not the same as allocating a ZERO-SIZED memory block, which is not allowed by the global allocator(it results in undefined behaviour). So if we can/t allocate, but also can't put a null pointer in ptr, what do we do in Vec::new?. Well, we just put some other garbage in there.

This is perfectly fine bcs we already have cap == 0 as our sentinel for no allocation. We don't even need to handle it specially in almost any code bcs we usually need to check if cap > len or len > 0 anyway.
The recommended Rust value to put here is mem::align_of::<T>(). NonNull provides a convenience for this.
NonNull::dangling(). There are quite a few places where we'll want to use dangling bcs there's no reall allocation to talk about but null wld make the computer do bad things. 

PUSH and POP:

To do the write we have to be careful not to evaluate the memory we want to write to.At worst, its truly uninitialized memory from the allocator. At best, its the bits of some old value we popped off. Either way, we can't just index to the memory and dereference it, bcs that will evaluate the memory as a valid instance of T. Worse, foo[idx] = x will try to call drop on the old value of foo[idx].

The correct way to do this is with ptr::write which blindly overwrites the target address with the bits of the value we provide. NO evaluation involved.
For push, if the old len(bfr push was called) is 0, then we want to write to the 0th index. So we should offset bt the old len