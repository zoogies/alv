# ROADMAP

Just some general ideas as I work on how I will translate the final Lox interpreter to something more interesting/

1. implement all the optimizations he suggests in the book
2. implement strict typing with "auto/let" binding. might steal rust immutability and shadowing (think really hard about what the point of using it as a scripting langauge is and what level of annoying strict protections are even helpful to the user.)
    - in general, debugging embedded scripting languages can be quite annoying, so certainly anything that can be done in the LSP and compiler that reduces iterations is good, but it cant ergonomically hurt the process of writing code
3. comprehensive stdlib
4. alv ecs

## tinies

- follow the python style of putting stack traces in "reverse" order with most recent frame at the bottom
