#![no_main]

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
struct State {
    value: u64,
}

struct Contract;

#[spin_sdk_macros::contract]
impl Contract {
    pub fn init() {
        let state = State { value: 0 };
        env::set_state(String::from("root"), state);
    }

    pub fn get() {
        let state: State = env::get_state(String::from("root")).unwrap();
        env::commit(state);
    }

    pub fn add() {
        let mut state: State = env::get_state(String::from("root")).unwrap();
        state.value += 1;

        env::set_state(String::from("root"), state);
    }
}
