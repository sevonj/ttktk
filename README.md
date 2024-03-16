# TTKTK - TTK-91 ToolKit

![CI](https://github.com/sevonj/ttktk/actions/workflows/main.yml/badge.svg)

This Rust package provides both a library, and a standalone executable for managing TTK-91 source (.k91) binary (.b91) files.  
This is part of [TiToMachine](https://github.com/sevonj/titomachine).

Modules:
- **titoasm** - Assembler.
- (disassembler: TiToMachine disassembler will be moved here)

![img.png](docs/box.png)

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

## Building
You need Rustlang.

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
