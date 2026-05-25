# Language Spec

## Goals

- Create a simple, embeddable scripting langauage centered around game development
- Make the ECS paradigm a first class language feature
- Support for running standalone, for other generic purposes

## Design Decisions

### Typing

alv is statically typed, with type inference using `let` and `const`.

### Syntax

alv expresses comments like so:

```alv
// this is a single line comment.

// there are no multi line comments,
// so you must chain single line
// comments together.
```

The syntax of alv is C-like, in that it requires semicolons and block braces, while adopting modern declaration and function header syntax.

```alv
let health = 100; // inferred type

fn damage(e: Entity, amount: int) -> void {
    ...
}
```

### Execution Model

There is no execution of top-level statements in module code. In Python and Lua, any code at the top level will be automatically executed, like so:

```python
a = 1
print(a)  # prints 1
```

In alv, you can only evaluate non-declarative statements inside function bodies, like `main`:

```alv
fn main() -> void {
    int a = 1;
    print(a);  // prints 1
}
```

### Core Constructs

Like in many languages, there are functions:

```alv
fn add(x: int, y: int) -> int {
    return x + y;
}
```

In alv, ECS is also a native language feature.

For example, a system can be defined like so:

```alv
sys DamageBurning(dt: float) phase Update {
    for e in query(Health, Burning) {
        e.Health.hp -= e.Burning.dps;
    }
}
```

Systems are different from functions, in that they are scheduled.

## TODO

Idk, this needs a good amount of work. Unsure if writing up the spec should happen first, or if it should be entirely driven by feedback from the implementation.
