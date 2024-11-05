# 1699 Clean Artifacts Reproduction

## Trying to reproduce the issue

[The issue](https://github.com/stellar/stellar-cli/issues/1699) is about having to clear/remove parts of a "dirty" `target` directory when re-building an modified smart contract.

## Baseline

I ran the following to start this repo:

```bash
stellar contract init 1699-clean-artifacts
```

I updated the `soroban-sdk` dependency in [Cargo.toml](./Cargo.toml) to use
`21.7.6`. Compiled with a very basic `stellar contract build`. Current state of
the compiled binary:

```bash
$ sha256sum target/wasm32-unknown-unknown/release/hello_world.wasm
7e26f8c9212bdb525094321757d7a93d541ffae5f494985b07c8d2660262b119  target/wasm32-unknown-unknown/release/hello_world.wasm

$ sha256sum target/wasm32-unknown-unknown/release/hello_world.d
7cd375dc75c218603c18c17aa4a8377db3431eb5ca61680c725782b1007d8449  target/wasm32-unknown-unknown/release/hello_world.d
```

## Next attempt

### Add a new function

I modified the [contract](./contracts/hello_world/src/lib.rs) to add a `goodbye`
function:

```rust
pub fn goodbye(env: Env, to: String) -> Vec<String> {
    vec![&env, String::from_str(&env, "Goodbye"), to]
}
```

### Did the new function change the compiled file?

Recompile with a (still) very basic `stellar contract build`. New state of the compiled binary:

```bash
$ sha256sum target/wasm32-unknown-unknown/release/hello_world.wasm
a961f79d9f6db6c9f93e5c9d51443133c4d764757304b803dadb8dc28951f386  target/wasm32-unknown-unknown/release/hello_world.wasm

$ sha256sum target/wasm32-unknown-unknown/release/hello_world.d
7cd375dc75c218603c18c17aa4a8377db3431eb5ca61680c725782b1007d8449  target/wasm32-unknown-unknown/release/hello_world.d
```

Still the same Wasm file, though the `hello_world.d` file is unchanged, if
that's important? :shrug:

## What if we add a `types.rs` file into the mix?

### Add a new file, using new SDK structs and macros

I created a [`types.rs` file](./contracts/hello_world/src/types.rs), to see if that
has any impact.

```rust
use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct State {
    pub count: u32,
    pub last_incr: u32,
}
```

Add two functions to our contract to make use of the custom type:

```rust
pub fn increment(env: Env, incr: u32) -> u32 {
    let mut state = Self::get_state(env.clone());

    state.count += incr;
    state.last_incr = incr;

    env.storage().persistent().set(&symbol_short!("STATE"), &state);

    state.count
}

pub fn get_state(env: Env) -> types::State {
    env.storage().persistent()
        .get(&symbol_short!("STATE"))
        .unwrap_or_else(|| types::State::default())
}
```

### Did any of that change the compiled binary?

Another `stellar contract build` and the compiled file is:

```bash
$ sha256sum target/wasm32-unknown-unknown/release/hello_world.wasm
da01403f01c4816c542a7587e233c30a5c445ab4a74b32c727789dd76d147f17  target/wasm32-unknown-unknown/release/hello_world.wasm

$ sha256sum target/wasm32-unknown-unknown/release/hello_world.d
0dc4e452ca908fb1df0eed04fb0dc7c55b0c9ab9c050af23cba03ab827627808  target/wasm32-unknown-unknown/release/hello_world.d
```

Different. Just as it should be.

## What if we add some `fn`s outside the contract impl?

### Let's abstract some stuff

We'll make a `store_state()` and `retrieve_state()` function:

```rust
fn store_state(env: &Env, state: &types::State) {
    env.storage().persistent().set(&symbol_short!("STATE"), state);
}

fn retrieve_state(env: &Env) -> types::State {
    env.storage().persistent().get(&symbol_short!("STATE")).unwrap_or_default()
}
```

and utilize them inside `increment()` and `get_state()`, respectively:

```rust
pub fn increment(env: Env, incr: u32) -> u32 {
    let mut state = Self::get_state(env.clone());

    state.count += incr;
    state.last_incr = incr;

    store_state(&env, &state);

    state.count
}

pub fn get_state(env: Env) -> types::State {
    retrieve_state(&env)
}
```

### And the results

Yet another `stellar contract build`, and yet another binary compilation:

```bash
$ sha256sum target/wasm32-unknown-unknown/release/hello_world.wasm
da01403f01c4816c542a7587e233c30a5c445ab4a74b32c727789dd76d147f17  target/wasm32-unknown-unknown/release/hello_world.wasm

$ sha256sum target/wasm32-unknown-unknown/release/hello_world.d
0dc4e452ca908fb1df0eed04fb0dc7c55b0c9ab9c050af23cba03ab827627808  target/wasm32-unknown-unknown/release/hello_world.d
```

Whoa, it happened!!??? Ok, ok, ok... but what if I created a new contract, with
the same contents. Would that hash be different?

I've made a duplicate contract in the `new_hello_world` directory. (I'm perhaps
realizing now that just by renaming it, that might change the compiled binary
file... maybe unhelpful...) Let's compile!

```bash
stellar contract build --package=new-hello-world
```

And check the file hashes:

```bash
$ sha256sum target/wasm32-unknown-unknown/release/new_hello_world.wasm
da01403f01c4816c542a7587e233c30a5c445ab4a74b32c727789dd76d147f17  target/wasm32-unknown-unknown/release/new_hello_world.wasm

$ sha256sum target/wasm32-unknown-unknown/release/new_hello_world.d
ec7ef8c9448274c0d4e0eb3986c4a3c8c6b382bd83172305fa1352867f7465c4  target/wasm32-unknown-unknown/release/new_hello_world.d
```

Well, that compiled file is identical (i guess renaming it had no material
affect).

## What if we adjust the custom type

### More timestampy

Let's add a timestamp in the `types.rs` file (moving back to the `hello-world`
contract, btw)

```rust
#[contracttype]
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct State {
    pub count: u32,
    pub last_incr: u32,
    pub timestamp: u64,
}
```

Make use of the timestamp in the `increment()` function:

```rust
pub fn increment(env: Env, incr: u32) -> u32 {
    let mut state = Self::get_state(env.clone());

    state.count += incr;
    state.last_incr = incr;
    state.timestamp = env.ledger().timestamp();

    store_state(&env, &state);

    state.count
}
```

### More results-y

And rinse and repeat `stellar contract build`, and rinse and repeat hashing files:

```bash
$ sha256sum target/wasm32-unknown-unknown/release/hello_world.wasm
6c34ac38322da79ad60c8689a2112c1b73c772cd47fd62e19a878e034d67735f  target/wasm32-unknown-unknown/release/hello_world.wasm

$ sha256sum target/wasm32-unknown-unknown/release/hello_world.d
0dc4e452ca908fb1df0eed04fb0dc7c55b0c9ab9c050af23cba03ab827627808  target/wasm32-unknown-unknown/release/hello_world.d
```

Yeah, that seems about right... I don't know what else to try??!

## What if we change _how_ the non-impl functions work?

### Subtle, easy-to-not-notice change, perhaps?

Let's do a different default if the persistent entry doesn't yet exist?

```rust
fn retrieve_state(env: &Env) -> types::State {
    env.storage().persistent().get(&symbol_short!("STATE")).unwrap_or_else(|| types::State {
        count: 86u32,
        last_incr: 75u32,
        timestamp: 309u64,
    })
}
```

### Hopefully not-noticed compilation differences?

This should work, right?

```bash
$ sha256sum target/wasm32-unknown-unknown/release/hello_world.wasm
728a30ae1f7f7c0e68a2bb6ecbd08a7e5e237a4ccf8f32f6f73395e7ff881881  target/wasm32-unknown-unknown/release/hello_world.wasm

$ sha256sum target/wasm32-unknown-unknown/release/hello_world.d
0dc4e452ca908fb1df0eed04fb0dc7c55b0c9ab9c050af23cba03ab827627808  target/wasm32-unknown-unknown/release/hello_world.d
```
