# Solana Debugger CLI

This is a CLI tool to debug Solana programs.

It will support the following features:
- inspect the value of a variable at a breakpoint
- display the call stack
- evaluate arbitrary Rust expressions at specified locations
- it can be used as debugger backend of another system (e.g. an IDE plugin)

[Showcase](https://x.com/maximschmidt94/status/1914802590568562965)

**If you want to try this yourself, check out the [tutorial](tutorial.md)**

## Status

Current stage of development: pre-alpha

This project is under active development. Only variable inspection works right now.

## Program input

To run the debugger, you must specify the entire input to the Solana program (accounts, signers, transaction).

We use a format that is compatible with other Solana tools and should be familiar to Solana devs. [Here is an example](https://github.com/Solana-Debugger/delta-counter-program-example/tree/main/debug_input/increase_counter_from_0_by_100).

Since creating this input can be hard, we provide a method to [generate inputs from tests](https://github.com/Solana-Debugger/save-input).

## Installation

```
$ git clone https://github.com/Solana-Debugger/solana-debugger-cli
$ cd solana-debugger-cli
$ cargo build
$ ln -s `realpath target/debug/solana-debugger-cli` ~/bin/solana-debugger
```

## Usage

Before you can start debugging, you need to initialize a debugger session:
```
$ solana-debugger init path_to_program program_input
```

To inspect variables, use this:
```
$ solana-debugger file_path:line [variable, ...]
```

`file_path` is relative to the program's `src` folder

Example:
```
$ solana-debugger init token/program input/transfer_tokens
$ solana-debugger lib.rs:33 var1 var2
```

## Internals

What's so cool about this debugger?

Debuggers usually work by interrupting the execution of a program and allowing the user to inspect its memory via some source mapping like DWARF. This is not what we do here.

Instead, what we do is essentially automated printf debugging: We instrument the program in clever ways, run it through the SWM, capture its output and present it to the user.

This means: 100% reliable outputs, you can set breakpoints at any line, compiler optimization doesn't get in the way, you can get other traces like compute units, it can deal with libraries that use code generation (Anchor!), you have access to any variable that you'd have access to in the Rust program.

While this is an unconventional approach, it allows for more robust and reliable source-level debugging.
