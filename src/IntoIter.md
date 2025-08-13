Moving into iterators - iter and iter_mut have already been written for us, thanks to the magic of Deref.However, there's two interesting iterators that Vec provides that slices can't: into_iter and drain.
IntoIter consumes the Vec by-value, and can consequently yield its elements by-value. In order to enable this, IntoIter needs to take control of Vec's allocation.

IntoIter needs to be douBleEnded as well, to enable reading from both ends. Reading from the back could just be implemented as calling pop, but reading from the front is harder. We could call remove(0) but that wld be insanely expensive. Instead we'll just use ptr::read to copy values out of either end of the Vec without mutating the buffer at all.

To achieve this, we are to use a very common C idiom for array iteration. We make two pointers, one that points to the start of the array, and one that points to one-element past the end.When we want an element from one end, we'll read  out the value pointed to at that end and move the pointer over by one. 
When the two pointers are equal, we know we are done.
