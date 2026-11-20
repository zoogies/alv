# Optimizations

tracking easy optimization targets rn:

- anything that uses Token
- clone and copies
- we could just borrow all strings from slices of the lifetime input program instead of throwing around copies in Token and other stuff

"When multiple local variables go out of scope at once, you get a series of OP_POP instructions that get interpreted one at a time. A simple optimization you could add to your Lox implementation is a specialized OP_POPN instruction that takes an operand for the number of slots to pop and pops them all at once."

- give or and and dedicated opcodes/instructions

personal ideas:
- loop unrolling?
- can always go look at llvm for ideas for passes

```
Most of the remaining overhead is in fetching each instruction. Every read_byte calls self.frame() twice, and each call does a last_mut().expect(..). Then chunk() does another frames.last(), a heap index, a get_fn match, and a bounds-checked index into code. That's several lookups per bytecode byte. The book avoids this by caching the current frame in a local inside run(): CallFrame* frame = &vm.frames[vm.frameCount - 1];. A Rust version of that caching is a nice speedup to come back to after the deadline. It's not worth doing now.
```

on the heap the global strings table for interning stores strings full values as their keys, so every _unique_ string gets stored twice
- possibly use Rc or something for the string itself, the Rc is the handle and the deref for string value
