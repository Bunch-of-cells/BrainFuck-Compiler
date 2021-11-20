# BrainFuck-Compiler

This is a BrainFuck Compiler written purely in Rust. It converts BrainFuck code to C code, and then finally machine language, thus generating an executable.

As of now, this compiler is able to draw a mandelbrot set in 0.25 seconds. take a look at mandelbrot.bf for the program

# Setup
You need to install Rust to make the executable. If you are on Linux you can find a pre-compiled executable in target/release/brainfuck.

# Bugs
If you run the executable with no arguments, the stdin interpreter sstarts running. But when you start writing anything it does not show up on the console.

# Usage
You can invoke the program like this:
```
./brainfuck filename
```
If you want a list of all the flags you can run `./brainfuck -h`
