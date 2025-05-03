# Solana Debugger CLI

This is a CLI tool to debug Solana programs.

The following features are planned:
- inspect the value of variables at a location (similar to breakpoints + inspect)
- display the call stack
- evaluate arbitrary Rust expressions at specified locations
- backend mode: it should be useable as a component in a different system (e.g. for an IDE plugin)

Current stage of development: pre-alpha

This project is under active development. Many features have not been implemented yet (call stack, expression evaluation etc.). Only variable inspection works right now.

To run the debugger, you must specify the entire input to the Solana program (accounts, signers, transaction). We use a format that should be familiar to Solana devs: [here are some examples](https://github.com/Solana-Debugger/delta-counter-program-example/tree/main/debug_input). Since creating this input manually is hard, we provide a module to [generate inputs from tests](https://github.com/Solana-Debugger/save-input).

**[Preview]**(https://x.com/maximschmidt94/status/1914802590568562965)

## Tutorial ("I want to try this myself!")

TODO
