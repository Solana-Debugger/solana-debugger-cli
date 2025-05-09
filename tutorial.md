# Tutorial

## Installation

This was tested under Solana v2.1.9. You may need to change your version:
```
$ agave-install init 2.1.9
```

Install the tool:
```
$ git clone https://github.com/Solana-Debugger/solana-debugger-cli
$ cd solana-debugger-cli
$ cargo build
$ ln -s `realpath target/debug/solana-debugger-cli` ~/bin/solana-debugger
```
This was tested using the stable Rust toolchain v1.84.0.

## Delta counter program

### Setup

Pull the example program:
```
$ git clone https://github.com/Solana-Debugger/delta-counter-program-example
```

Initialize the debugger with it:
```
$ cd delta-counter-program-example
$ solana-debugger init delta-counter debug_input/create_counter
```

Verify that it works by displaying the entrypoint arguments (compiling may take a while):
```
$ solana-debugger entrypoint.rs:18
```
This should print `program_id`, `accounts` and `instruction_data`.

### Create counter

Show a variable inside a function:
```
$ solana-debugger processor/process_create_counter.rs:42 counter_info
```
Note that `counter_info` is the PDA we will write to. Using the debugger's output, confirm that `is_signer` is `false` and `is_writable` is `true`, as you'd expect here.

Show multiple variables: 
```
$ solana-debugger processor/process_create_counter.rs:60 rent space rent_lamports
```

Show a custom struct:
```
$ solana-debugger processor/process_create_counter.rs:78 counter
```
`count` should be zero since that's the counter's initial value.

### Increase counter

Now we will switch to a different program input. This time, we will debug an instruction that increases an existing counter's value:
```
$ solana-debugger init delta-counter debug_input/increase_counter_from_100_by_155
```

Show all available variables at the beginning of the `increase_counter` function (compiling may take a while)
```
$ solana-debugger processor/process_increase_counter.rs:28
```
Try to find `delta` among them! It should be `155` since we want to increase the counter by 155

Show the counter struct before and after the increase:
```
$ solana-debugger processor/process_increase_counter.rs:55 counter
$ solana-debugger processor/process_increase_counter.rs:63 counter
```
Verify that `count` goes from `100` to `255`

## Governance program

### Setup

Pull the test program:
```
$ git clone https://github.com/Solana-Debugger/governance-program-example
```

Initialize the debugger with it:
```
$ cd governance-program-example
$ solana-debugger init solana-program-library/governance/program debug_input/create_realm
```

Verify that it works (compiling may take a while):
```
$ solana-debugger processor/process_create_realm.rs:48
```
This should print various variables holding `AccountInfo`s, such as `realm_info`.

Try to find `name: "Realm #0"` in this output!

### Custom types

Print some crate-specific structs:
```
$ solana-debugger processor/process_create_realm.rs:116 realm_config_data
$ solana-debugger processor/process_create_realm.rs:149 realm_data
```

### Multiple hits

Let's look at a line that will be executed multiple times.

We will look at the function `create_and_serialize_account_signed` from `tools/spl_token.rs`. It's used to create new tokens (1 call = 1 new token). To create a new token, it executes two CPIs: one to make an account using the System Program and one to initialize it using the Token Program.

Note that `process_create_realm` calls `create_and_serialize_account_signed` twice (i.e. it creates two tokens). What happens if we try to inspect it?

Run this:
```
$ solana-debugger tools/spl_token.rs:80 create_account_instruction initialize_account_instruction
```

This should show two line hits. Each of them should print two variables (i.e. show both instructions).

So, for each of the two tokens created, this command shows both instructions needed to do that.

Note that the `program_id`s are indeed `11111111111111111111111111111111` and `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`.
