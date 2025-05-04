# Tutorial

## Installation

This was tested under Solana v2.1.9. Change it if needed:
```
agave-install init 2.1.0
```

I tested this with Rust toolchain v1.84.0

Install the tool:
```
git clone https://github.com/Solana-Debugger/solana-debugger-cli
```
```
cd solana-debugger-cli
```
```
cargo build
```

## Delta counter program

* Initialize
```
$ solana-debugger init delta-counter delta-counter_input/create_counter
```

* Show entrypoint arguments
```
$ solana-debugger entrypoint.rs:18
```
* Prints `program_id`, `accounts`, `instruction_data`

* Show a variable inside a function
```
$ solana-debugger processor/process_create_counter.rs:42 counter_info
```
* `counter_info` is the PDA we will write to
* So, you can verify with this output that `is_signer` is `false` and `is_writable` is `true`

* Show other variables
```
$ solana-debugger processor/process_create_counter.rs:60 rent space
```
TODO: test rent_lamports

* Show a custom struct
```
$ solana-debugger processor/process_create_counter.rs:78 counter
```
* Should be zero since that's the counter's initial value

* Switch to another input
* This time, we will try to increase an existing counter
```
$ solana-debugger init delta-counter delta-counter_input/increase_counter_from_100_by_155
```

* Show all variables at the beginning of `increase_counter`
```
$ solana-debugger processor/process_increase_counter.rs:28
```
* Try to find `delta` among them; it should be `155` (we want to increase the counter by `155`)
```
$ solana-debugger processor/process_increase_counter.rs:55 counter
```
* Show the deserialization of the counter PDA
* `count` should be `100`

* Check the values after the addition has happened
```
$ solana-debugger processor/process_increase_counter.rs:63 counter
```
* `count` should be `255`

## SPL governance program

* Initialize
```
$ solana-debugger init solana-program-library/governance/program governance_input/create_realm
```

* Make sure it's working (compiling will take a while)
```
$ solana-debugger processor/process_create_realm.rs:48
```
* This should print all accounts
* Also: `name: "Realm #0"`

* Print some custom types
```
$ solana-debugger processor/process_create_realm.rs:116 realm_config_data
$ solana-debugger processor/process_create_realm.rs:149 realm_data
```

* Let's look at a line that will be hit twice
* Consider the function `create_and_serialize_account_signed` from `tools/spl_token.rs`
* It's used to create new tokens (1 call = 1 new token)
* To create a new token, it executes two CPIs: one to make an account using the System Program and one to initialize it using the Token Program
* Note that `process_create_realm` calls `create_and_serialize_account_signed` twice (i.e. it creates two tokens)
* Run this:
```
$ solana-debugger tools/spl_token.rs:80 create_account_instruction initialize_account_instruction
```
* With this command, we can inspect the ixs for both of these tokens
* There should be two line hits, each showing both of the ixs
