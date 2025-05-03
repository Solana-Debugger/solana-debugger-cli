# Solana Debugger CLI

This is a CLI tool to debug Solana programs.

The following features are planned:
- inspect the value of variables at a location (similar to breakpoints + inspect)
- display the call stack
- evaluate arbitrary Rust expressions at specified locations
- backend mode: it should be useable as a component in a different system (e.g. for an IDE plugin)

[Showcase video](https://x.com/maximschmidt94/status/1914802590568562965)

**If you want to try this yourself, check out [the tutorial](tutorial.md)**

## Status

Current stage of development: pre-alpha

This project is under active development. Many features have not been implemented yet (call stack, expression evaluation etc.). Only variable inspection works right now.

## Input

To run the debugger, you must specify the entire input to the Solana program (accounts, signers, transaction).

We use a format that should be familiar to Solana devs. [Here are some examples](https://github.com/Solana-Debugger/delta-counter-program-example/tree/main/debug_input).

Since creating this input manually is hard, we provide a module to [generate inputs from tests](https://github.com/Solana-Debugger/save-input).

## Installation

```
$ git clone https://github.com/Solana-Debugger/solana-debugger-cli
$ cd solana-debugger-cli
$ cargo build
$ ln -s `realpath target/debug/solana-debugger-cli` ~/bin
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

`file_path`, is relative to the `src` folder

Example:
```
$ solana-debugger init token/program input/transfer_tokens
$ solana-debugger lib.rs:33 var1 var2
```
