# Thread-Safe queue

This is an implementation of a thread-safe queue in Rust adapted from the one given in the "Multi-Threading in C++" course at TU Darmstadt.  
The initial version was unsound regarding the Stacked Borrows concept and was made sound with the help of the lovely people of [this](https://users.rust-lang.org/t/implementation-of-thread-safe-queue-miri-failure/47172) thread.