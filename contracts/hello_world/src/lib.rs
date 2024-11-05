#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, String, Vec};

fn store_state(env: &Env, state: &types::State) {
    env.storage().persistent().set(&symbol_short!("STATE"), state);
}

fn retrieve_state(env: &Env) -> types::State {
    env.storage().persistent().get(&symbol_short!("STATE")).unwrap_or_default()
}

#[contract]
pub struct HelloContract;

#[contractimpl]
impl HelloContract {
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "Hello"), to]
    }

    pub fn goodbye(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "Goodbye"), to]
    }

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
}

mod test;
mod types;
