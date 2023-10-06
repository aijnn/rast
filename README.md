# Rust AST

Rast is a simple library for writing rules which search Rust abstract syntax trees (AST).

Static application security testing (SAST) tools commonly provide their own pattern syntax which can
be used to describe code patterns. When exploring codebases I like to write rules on fly and I often
find myself in situations where the provided pattern syntax just can't describe what I want to find.
I avoid these limitations by writing rules "as code". This library provides helper functions and the
necessary boilerplate to scan a directory, parse Rust files and run a rule against them. It is meant
help me quickly write rules when searching for specific code patterns. Any serious SAST rules should
use proper tooling instead. This library doesn't currently do much but should grow as I use it more.

See `examples/` for intended use:
```
# Find all function definitions
cargo run --example fn <source_code_directory>
```
