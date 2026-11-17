# Optimizations

tracking easy optimization targets rn:

- anything that uses Token
- clone and copies
- we could just borrow all strings from slices of the lifetime input program instead of throwing around copies in Token and other stuff

"When multiple local variables go out of scope at once, you get a series of OP_POP instructions that get interpreted one at a time. A simple optimization you could add to your Lox implementation is a specialized OP_POPN instruction that takes an operand for the number of slots to pop and pops them all at once."

- give or and and dedicated opcodes/instructions
