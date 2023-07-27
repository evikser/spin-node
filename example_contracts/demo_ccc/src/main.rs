#![no_main]

struct Contract;

#[spin_sdk_macros::contract]
impl Contract {
    pub fn fibonacci_and_multiply(input: (u32, u64)) {
        let n = input.0;
        let multiplier = input.1;

        let result: u64 = env::cross_contract_call(
            AccountId::new("fibonacci.spin".to_string()),
            "entypoint".to_string(),
            10_000,
            n as u32,
        );

        let result = result * multiplier;

        env::commit(result);
    }

    pub fn transfer_token(input: (AccountId, AccountId, u128)) {
        let token_account = input.0;
        let recipient = input.1;
        let amount = input.2;

        let _: () = env::cross_contract_call(
            token_account,
            "transfer".to_string(),
            1_000_000,
            (recipient, amount),
        );
    }

    pub fn hello(name: String) {
        let result = format!("Hello, {}!", name);
        env::commit(result);

        risc0_zkvm::guest::env::read()
    }
}
