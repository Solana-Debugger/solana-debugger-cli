# Solana Debugger CLI

This is a CLI tool to debug Solana programs.

It will support the following features:
- Inspect the value of a variable at a breakpoint
- Display the call stack
- Evaluate arbitrary Rust expressions at specified locations
- It can be used as debugger backend of another system (e.g. an IDE plugin)

## Demo

![Solana Debugger CLI screenshot](inline-screenshot.png)

[Video](https://x.com/maximschmidt94/status/1914802590568562965)

[Screenshot](screenshot.png)

## Quick Test

If you just want to see that it works:

```
$ agave-install init 2.1.9
$ mkdir tmp
$ cd tmp
$ git clone https://github.com/Solana-Debugger/solana-debugger-cli
$ cd solana-debugger-cli 
$ cargo build
$ cd ..
$ git clone https://github.com/Solana-Debugger/delta-counter-program-example
$ cd delta-counter-program-example
# to force the installation of platform-tools (skip if not needed)
$ cd delta-counter; cargo-build-sbf; cd ..
$ ../solana-debugger-cli/target/debug/solana-debugger-cli init delta-counter debug_input/create_counter
$ ../solana-debugger-cli/target/debug/solana-debugger-cli entrypoint.rs:18
```

This should print `program_id`, `accounts` and `instruction_data`.

For details, see the [tutorial](tutorial.md).

## Tutorial

For a demonstration of all features, check out the [tutorial](tutorial.md).

It uses two example programs specifically made to be tested with Solana Debugger:

* [Delta counter program](https://github.com/Solana-Debugger/delta-counter-program-example)
* [Governance program](https://github.com/Solana-Debugger/governance-program-example)

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

Instead, we do this: We instrument the program in clever ways (i.e. add log statements), run it through the SVM, capture its output and present it to the user. Think of it as automated printf debugging.

This means: 100% reliable outputs, you can set breakpoints at any line, you have access to any variable that you'd have access to in the Rust program, compiler optimization never gets in the way, you can get other traces like compute unit consumption, it can deal with frameworks that use code generation (Anchor!), CPIs can be debugged as you would expect.

While this is an unconventional approach, it allows for robust and reliable source-level debugging.
