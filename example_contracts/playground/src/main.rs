#![no_main]

struct Contract;

#[spin_sdk_macros::contract]
impl Contract {
    pub fn hello() {
        let output = format!("Hello, {}!", env::signer().to_string(),);
        env::commit(output);
    }
}
