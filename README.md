# alv

alv is a C-Like, ECS-native interpreted game scripting language.

## disclaimer

This repo is totally WIP and not a real thing you should even try to build or use. I'm currently just building a `lox` interpreter in Rust

## running

example:

```bash
cargo r tests/10.4.lox 
```

### TODO

Change Stmt::Function body into it's on struct such that we don't have to destructure with `if let` at every site.
