# TTKTK - TTK-91 ToolKit

![CI](https://github.com/sevonj/ttktk/actions/workflows/main.yml/badge.svg)

This Rust package provides tools for meddling with TTK-91 code.
This is part of [TiToMachine](https://github.com/sevonj/titomachine) project.

Binaries:
- **titoasm** - Assemble .k91 files to .b91

Library:
- **libttktk::compiler** - Assembler backend for titoasm and titomachine
- **libttktk::disassembler** - Disassembler
- **libttktk::instructions** - Instruction struct and related enums.
- **libttktk::b91** - Parse .b91 contents.

## Additions and differences to Titokone
(see: [Titokone](https://www.cs.helsinki.fi/group/titokone/))
- Supports expressing values in bin, oct, and hex.
- Supports expressing values as unsigned.
- Symbols are case sensitive.
- Supports TiToMachine extended spec, but should be fully backwards compatible.

## Usage
![img.png](docs/example_command.png)
```shell
   titoasm --help
```
```shell
   titoasm file.k91
```
```shell
   titoasm file.k91 -o outputfile.b91
```

## Use libttktk in rust code
Cargo.toml:
```toml
    [dependencies]

    # ...
    
    ttktk = { git = "https://github.com/sevonj/ttktk.git", tag = "v0.3.0" }
```
```rust
    use libttktk::compiler::compile;

    // ...

    let result = compile(source);
```

## Building
You need Rust.

Shell examples:
```shell
    cargo build
```
```shell
    cargo test
```
```shell
    cargo run -- file.k91 -o outputfile.b91
```
