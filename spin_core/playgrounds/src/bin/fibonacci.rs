use borsh::{BorshDeserialize, BorshSerialize};
use playgrounds::{init_temp_node, install_tracing};
use spin_primitives::{
    AccountId, Block, Digest, ExecutionOutcome, SignedTransaction, SYSTEM_META_CONTRACT_ACCOUNT_ID,
};
use tracing::info;

// TODO: move to spin_primitives
#[derive(BorshDeserialize, BorshSerialize, Debug)]
struct Account {
    pub account_id: AccountId,
    pub public_key: Vec<u8>,
    pub code_hash: Digest,
    pub balance: u128,
    pub nonce: u64,
}

fn main() {
    install_tracing();

    let mut node = init_temp_node();

    let latest_block = node.latest_block();
    let fibonacci = AccountId::new(String::from("fibonacci.spin"));

    let tx = create_account(&latest_block, fibonacci.clone());
    let tx_hash = tx.tx.hash;
    node.add_tx(tx);
    let block = node.produce_block();
    let outcome = block.execution_outcomes.get(&tx_hash).unwrap();
    info!(?outcome, "Outcome");

    let tx = account_info(&latest_block, fibonacci.clone());
    let tx_hash = tx.tx.hash;
    node.add_tx(tx);
    let block = node.produce_block();
    let outcome = block.execution_outcomes.get(&tx_hash).unwrap();
    let account: Account = BorshDeserialize::deserialize(&mut outcome.output.as_slice()).unwrap();
    info!(?account, "Account");

    let tx = deploy_contract(
        &latest_block,
        fibonacci.clone(),
        include_bytes!("../../../../example_contracts/target/riscv-guest/riscv32im-risc0-zkvm-elf/release/fibonacci_contract").to_vec(),
    );
    let tx_hash = tx.tx.hash;
    node.add_tx(tx);
    let block = node.produce_block();
    let outcome = block.execution_outcomes.get(&tx_hash).unwrap();
    info!(?outcome, "Outcome");

    let tx = account_info(&latest_block, fibonacci.clone());
    let tx_hash = tx.tx.hash;
    node.add_tx(tx);
    let block = node.produce_block();
    let outcome = block.execution_outcomes.get(&tx_hash).unwrap();
    let account: Account = BorshDeserialize::deserialize(&mut outcome.output.as_slice()).unwrap();
    info!(?account, "Account");

    // call fibonacci
    let n = 10u32;
    let tx = spin_primitives::TransactionBuilder::new(
        fibonacci.clone(),
        "fibonacci".to_string(),
        BorshSerialize::try_to_vec(&n).unwrap(),
        100_000_000,
        AccountId::new("".to_string()),
        &latest_block,
    )
    .build();

    let tx = SignedTransaction {
        tx,
        signature: Default::default(),
    };
    let tx_hash = tx.tx.hash;
    node.add_tx(tx);
    let block = node.produce_block();
    let outcome = block.execution_outcomes.get(&tx_hash).unwrap();
    let outcome: ExecutionOutcome =
        BorshDeserialize::deserialize(&mut outcome.output.as_slice()).unwrap();
    let output: u64 = outcome.try_deserialize_output().unwrap();
    info!(?output, n, "fibonacci");
}

fn account_info(latest_block: &Block, account_id: AccountId) -> SignedTransaction {
    let tx = spin_primitives::TransactionBuilder::new(
        AccountId::new(String::from(SYSTEM_META_CONTRACT_ACCOUNT_ID)),
        "account_info".to_string(),
        BorshSerialize::try_to_vec(&account_id.to_string()).unwrap(),
        100_000_000,
        AccountId::new("".to_string()),
        &latest_block,
    )
    .build();

    SignedTransaction {
        tx,
        signature: Default::default(),
    }
}

fn create_account(latest_block: &Block, account_id: AccountId) -> SignedTransaction {
    #[derive(BorshDeserialize, BorshSerialize)]
    struct AccountCreationRequest {
        pub account_id: AccountId,
        pub public_key: Vec<u8>,
    }

    let args = AccountCreationRequest {
        account_id,
        public_key: Vec::new(),
    };

    let tx = spin_primitives::TransactionBuilder::new(
        AccountId::new(String::from(SYSTEM_META_CONTRACT_ACCOUNT_ID)),
        "create_account".to_string(),
        BorshSerialize::try_to_vec(&args).unwrap(),
        100_000_000,
        AccountId::new("".to_string()),
        &latest_block,
    )
    .build();

    SignedTransaction {
        tx,
        signature: Default::default(),
    }
}

fn deploy_contract(
    latest_block: &Block,
    account_id: AccountId,
    code: Vec<u8>,
) -> SignedTransaction {
    #[derive(BorshDeserialize, BorshSerialize)]
    struct ContractDeploymentRequest {
        pub code: Vec<u8>,
    }

    let args = ContractDeploymentRequest { code };

    let tx = spin_primitives::TransactionBuilder::new(
        AccountId::new(String::from(SYSTEM_META_CONTRACT_ACCOUNT_ID)),
        "deploy_contract".to_string(),
        BorshSerialize::try_to_vec(&args).unwrap(),
        100_000_000,
        account_id,
        &latest_block,
    )
    .build();

    SignedTransaction {
        tx,
        signature: Default::default(),
    }
}

// use tracing::info;

// use spin_primitives::{AccountId, ExecutionOutcome};
// use spin_runtime::context::ExecutionContext;
// use spin_runtime::executor;

// use playgrounds::install_tracing;

// use std::sync::{Arc, RwLock};

// fn main() {
//     install_tracing();

//     let abi_path = String::from("./etc/evm_contracts/erc20.abi");
//     let bytecode_path = String::from("./etc/evm_contracts/erc20_bytecode");

//     let alice = AccountId::new("alice.spin".to_string());
//     let alice_evm_address = ExecutionContext::get_account_evm_address(alice.clone());

//     let abi = ethabi::Contract::load(std::fs::read(abi_path).unwrap().as_slice()).unwrap();

//     // init_evm_accounts();
//     // info!("EVM accounts initialized");

//     let token_address = deploy_evm_contract(&abi, bytecode_path, &alice);
//     info!(?token_address, "token deployed");

//     let token_owner = call_evm_contract(&abi, token_address, String::from("owner"), &[], &alice);

//     assert!(token_owner[0].clone().into_address().unwrap().0 == alice_evm_address.to_fixed_bytes());
//     info!("Token owner is alice");

//     let alice_balance = call_evm_contract(
//         &abi,
//         token_address,
//         String::from("balanceOf"),
//         &[ethabi::Token::Address(alice_evm_address)],
//         &alice,
//     );
//     info!(?alice_balance, "Alice balance");

//     call_evm_contract(
//         &abi,
//         token_address,
//         String::from("mint"),
//         &[
//             ethabi::Token::Address(alice_evm_address),
//             ethabi::Token::Uint(100.into()),
//         ],
//         &alice,
//     );
//     info!("Alice minted 100 tokens");

//     let alice_balance = call_evm_contract(
//         &abi,
//         token_address,
//         String::from("balanceOf"),
//         &[ethabi::Token::Address(alice_evm_address)],
//         &alice,
//     );
//     info!(?alice_balance, "Alice balance");
//     assert!(alice_balance[0].clone().into_uint().unwrap() == 100.into());
// }

// #[allow(dead_code)]
// fn init_evm_accounts() {
//     let ctx = Arc::new(RwLock::new(ExecutionContext::new(
//         spin_primitives::ContractCallWithContext::new(
//             AccountId::new("evm".to_string()),
//             "init".into(),
//             (),
//             100_000_000,
//             AccountId::new(String::from("alice.spin")),
//             AccountId::new(String::from("alice.spin")),
//         ),
//     )));

//     executor::execute(ctx.clone()).unwrap();
// }

// /// Deploy EVM contract and return its address
// fn deploy_evm_contract(
//     abi: &ethabi::Contract,
//     hex_bytecode_path: String,
//     owner_account_id: &AccountId,
// ) -> eth_primitive_types::H160 {
//     let bytecode_file = std::fs::read(hex_bytecode_path).expect("Can't read bytecode");
//     let bytecode = hex::decode(bytecode_file).expect("Can't decode bytecode");

//     let constructor = abi.constructor().unwrap();
//     let constructor_input = constructor.encode_input(bytecode, &[]).unwrap();

//     let ctx = Arc::new(RwLock::new(ExecutionContext::new(
//         spin_primitives::ContractCallWithContext::new(
//             AccountId::new("evm".to_string()),
//             "deploy_contract".into(),
//             constructor_input,
//             100_000_000,
//             owner_account_id.clone(),
//             owner_account_id.clone(),
//         ),
//     )));

//     let s = executor::execute(ctx.clone()).unwrap();
//     let committment: ExecutionOutcome =
//         borsh::BorshDeserialize::deserialize(&mut s.journal.as_slice()).unwrap();

//     let result: ([u8; 20], Vec<u8>) = committment.try_deserialize_output().unwrap();
//     let address = eth_primitive_types::H160::from_slice(&result.0);
//     info!(address = ?address, "Contract deployed");
//     address
// }

// fn call_evm_contract(
//     abi: &ethabi::Contract,
//     contract_address: eth_primitive_types::H160,
//     function: String,
//     args: &[ethabi::Token],
//     account_id: &AccountId,
// ) -> Vec<ethabi::Token> {
//     let function = abi.function(&function).unwrap();
//     let input = function.encode_input(args).unwrap();

//     let ctx = Arc::new(RwLock::new(ExecutionContext::new(
//         spin_primitives::ContractCallWithContext::new(
//             AccountId::new("evm".to_string()),
//             "call_contract".into(),
//             (contract_address.to_fixed_bytes(), input),
//             100_000_000,
//             account_id.clone(),
//             account_id.clone(),
//         ),
//     )));

//     let s = executor::execute(ctx.clone()).unwrap();
//     let committment: ExecutionOutcome =
//         borsh::BorshDeserialize::deserialize(&mut s.journal.as_slice()).unwrap();

//     let output: Vec<u8> = committment.try_deserialize_output().unwrap();
//     function
//         .decode_output(output.as_slice())
//         .expect("Can't decode output")
// }
