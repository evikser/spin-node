#![no_main]

use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
use spin_sdk::{
    env,
    spin_primitives::{
        AccountId, ContractEntrypointContext, Digest, ExecutionOutcome, SignedTransaction,
        Transaction, TransactionBody, SYSTEM_META_CONTRACT_ACCOUNT_ID,
    },
};

struct Balances {
    pub balances: HashMap<AccountId, u128>,
}

struct Contract;

// TODO: move to spin_primitives
#[derive(BorshDeserialize, BorshSerialize)]
struct Account {
    pub account_id: AccountId,
    pub public_key: Vec<u8>,
    pub code_hash: Digest,
    pub balance: u128,
    pub nonce: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
struct AccountCreationRequest {
    pub account_id: AccountId,
    pub public_key: Vec<u8>,
}

#[derive(BorshDeserialize, BorshSerialize)]
struct ContractDeploymentRequest {
    pub code: Vec<u8>,
}

risc0_zkvm::entry!(entrypoint);

fn entrypoint() {
    let mut bytes: Vec<u8> = risc0_zkvm::guest::env::read();
    let signed_tx: SignedTransaction =
        BorshDeserialize::try_from_slice(&mut bytes).expect("Corrupted transaction");

    let SignedTransaction { tx, signature: _ } = signed_tx;

    let call = ContractEntrypointContext {
        account: tx.body.contract.clone(),
        method: tx.body.method.clone(),
        args: tx.body.args.clone(),
        attached_gas: tx.body.attached_gas,
        sender: tx.body.signer.clone(),
        signer: tx.body.signer.clone(),
    };

    spin_sdk::env::setup_env(&call);

    // TODO: check tx signature

    match tx.body.contract.to_string().as_str() {
        SYSTEM_META_CONTRACT_ACCOUNT_ID => process_system_transaction(tx.body),
        _ => process_contract_transaction(tx.body),
    }
}

fn process_system_transaction(tx: TransactionBody) {
    match tx.method.as_str() {
        "create_account" => create_account(tx.try_deserialize_args().unwrap()),
        "deploy_contract" => deploy_contract(tx.try_deserialize_args().unwrap()),
        "account_info" => account_info(tx.try_deserialize_args().unwrap()),
        _ => {}
    }
}

fn create_account(req: AccountCreationRequest) {
    let AccountCreationRequest {
        account_id,
        public_key,
    } = req;

    let None = env::get_storage::<Account>(format!("accounts.{}", account_id.to_string())) else {
            panic!("Account already exists");
        };

    let account = Account {
        account_id: account_id.clone(),
        public_key,
        code_hash: Digest::default(),
        balance: 0,
        nonce: 0,
    };

    env::set_storage(format!("accounts.{}", account_id.to_string()), account);
    env::commit(())
}

fn account_info(account_id: AccountId) {
    let Some(account) = env::get_storage::<Account>(format!("accounts.{}", account_id.to_string())) else {
            panic!("Account does not exist");
        };

    env::commit(account)
}

fn deploy_contract(req: ContractDeploymentRequest) {
    let Some(_account) = env::get_storage::<Account>(format!("accounts.{}", env::signer().to_string())) else {
            panic!("Account does not exist");
        };

    // TODO: update account.code_hash
    env::set_storage(format!("code.{}", env::signer().to_string()), req.code);
    env::commit(());
}

fn process_contract_transaction(tx: TransactionBody) {
    let outcome = env::cross_contract_call_raw(tx.contract, tx.method, tx.attached_gas, tx.args);

    env::commit(outcome);
}
