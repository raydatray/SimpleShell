# SimpleShell
SimpleShell is an implementation of a shell originally done for McGill's COMP310/ECSE427 in W2024.

The original C version implements basic shell commands such as echo, set and:
- The ability to run scripts consisting of shell commands concurrently
- A basic scheduling system for when scripts are run concurrently (Round-Robin)
- Demand paging with an LRU cache replacement policy
- A file system with various utilities

Extended features in the Rust version include:
- The ability to select which type of scheduling you wish to use
- The ability to select which type of cache replacement you wish to use
- The ability to "reconfigure" the memory sizes of the shell without recompilation
- The ability to redefine page sizes and other shell parameters
- And arguably better written/more understandable code with proper error handling thru Rust's Result type

The original assignment submissions are in [a1](a1)/[a2](a2)/[a3](a3) and were done in collaboration with [Gordon](https://github.com/SoloUnity) 
- **A1:** tasked with implementing the following [commands](a1/interpreter.c); set, echo, my_mkdir, my_touch, my_touch, my_cd, my_cat 
- **A2:** tasked with implementing a [paging system](a2), extending it with demand paging, and finally an LRU replacement policy 
- **A3:** tasked with implementing various [file system utilities](a3/fs/fsutil2.c) based on an write-behind and inode with indirect pointer architecture 

The Rust rewrite is in [SimpleShell](SimpleShell) is an exercise for myself and to become more familiar with the language's differences compared to C <br>
I aim to: 
- Make use of the FP aspects of Rust
- Actually make use of references by borrowing values and not spamming .clone() everywhere
- Properly handle errors using Rust's result type
- Write "idiomatic" Rust code

## Some Notes

