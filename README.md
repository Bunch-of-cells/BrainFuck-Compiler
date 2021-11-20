# BrainFuck-Compiler

This is a BrainFuck Compiler written purely in Rust. It is a source-to-source compiler. It converts BrainFuck code to C code, and then finally machine code.

This compiler is able to compile the mandelbrot set in just 0.25 seconds. Take a look at mandelbrot.bf for the program

# Setup
You need to install Rust to make the executable.

# Bugs
If you run the executable with no arguments, the stdin interpreter starts running. But when you start writing anything it does not show up on the console.
Also, this compiler is not very optimized.

# Usage
You can invoke the program like this:
```
./brainfuck filename
```
If you want a list of all the flags you can run `./brainfuck -h`
