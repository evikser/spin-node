use borsh::{BorshDeserialize, BorshSerialize};
use once_cell::sync::Lazy;
use risc0_zkvm::sha::rust_crypto::{Digest, Sha256};
use std::{collections::HashMap, sync::Mutex};

use spin_primitives::{
    syscalls::{
        CrossContractCallRequest, GetStorageResponse, SetStorageRequest, CROSS_CONTRACT_CALL,
        GET_ACCOUNT_MAPPING, GET_STORAGE_CALL, SET_STORAGE_CALL,
    },
    AccountId, ContractEntrypointContext, Digest as HashDigest, ExecutionOutcome, StorageKey,
};

pub fn setup_env(call: &ContractEntrypointContext) {
    let mut env = ENV.lock().unwrap();
    *env = Some(Env::new_from_call(call));
}

#[derive(Debug, BorshSerialize)]
struct Env {
    signer: AccountId,
    caller: AccountId,
    contract: AccountId,
    attached_gas: u64,

    call_hash: HashDigest,
    initial_storage_hashes: HashMap<StorageKey, HashDigest>,
    storage_cache: HashMap<StorageKey, (Vec<u8>, bool)>,
    cross_calls_hashes: Vec<(HashDigest, HashDigest)>,
}

impl Env {
    fn new_from_call(call: &ContractEntrypointContext) -> Self {
        let call_hash = {
            let call_bytes = BorshSerialize::try_to_vec(&call).expect("Expected to serialize");
            let algorithm = &mut risc0_zkvm::sha::rust_crypto::Sha256::default();
            algorithm.update(&call_bytes);
            algorithm.finalize_reset().as_slice().try_into().unwrap()
        };

        Self {
            signer: call.signer.clone(),
            caller: call.sender.clone(),
            contract: call.account.clone(),
            attached_gas: call.attached_gas,
            call_hash: call_hash,
            initial_storage_hashes: Default::default(),
            storage_cache: Default::default(),
            cross_calls_hashes: Default::default(),
        }
    }
}

static ENV: Lazy<Mutex<Option<Env>>> = Lazy::new(|| Mutex::new(None));

impl Env {
    /// Returns the current call signer
    pub fn signer(&self) -> AccountId {
        self.signer.clone()
    }

    /// Returns the current call sender
    pub fn caller(&self) -> AccountId {
        self.caller.clone()
    }

    /// Returns the current call contract
    pub fn contract(&self) -> AccountId {
        self.contract.clone()
    }

    /// Makes a cross-contract call
    pub fn cross_contract_call(&mut self, req: CrossContractCallRequest) -> ExecutionOutcome {
        let req_hash = {
            let call_bytes = BorshSerialize::try_to_vec(&req).expect("Expected to serialize");
            let algorithm = &mut risc0_zkvm::sha::rust_crypto::Sha256::default();
            algorithm.update(&call_bytes);
            algorithm.finalize_reset().as_slice().try_into().unwrap()
        };

        let mut response = [0u32; 1024]; // TODO: make this dynamic

        risc0_zkvm::guest::env::syscall(
            CROSS_CONTRACT_CALL,
            req.try_to_vec().unwrap().as_slice(),
            &mut response,
        );

        let response: Vec<u8> =
            risc0_zkvm::serde::from_slice(&response).expect("Expected to deserialize");

        let outcome =
            ExecutionOutcome::try_from_bytes(response).expect("ExecutionOutcome is corrupted");

        // assert_eq!(req_hash, outcome.call_hash); // TODO: fix

        let output_hash = {
            let algorithm = &mut risc0_zkvm::sha::rust_crypto::Sha256::default();
            algorithm.update(&outcome.output);
            algorithm.finalize_reset().as_slice().try_into().unwrap()
        };

        self.cross_calls_hashes.push((req_hash, output_hash));

        outcome
    }

    /// Returns the storage value for the given key, return None if storage is not exist
    pub fn get_storage<T: BorshDeserialize>(&mut self, key: StorageKey) -> Option<T> {
        if let Some(storage_bytes) = self.storage_cache.get(&key) {
            return Some(
                BorshDeserialize::try_from_slice(storage_bytes.0.as_slice())
                    .expect("Expected to deserialize"),
            );
        }

        let mut response = [0u32; 65536]; // TODO: make this dynamic

        risc0_zkvm::guest::env::syscall(GET_STORAGE_CALL, &key.clone().into_bytes(), &mut response);

        let response: Vec<u8> =
            risc0_zkvm::serde::from_slice(&response).expect("Expected to deserialize");

        let response: GetStorageResponse =
            BorshDeserialize::try_from_slice(&mut response.as_slice())
                .expect("GetStorageResponse is corrupted");

        let Some(storage) = response.storage else {
        return None;
    };

        let hash = {
            let algorithm = &mut Sha256::default();
            algorithm.update(storage.clone());
            algorithm.finalize_reset().as_slice().try_into().unwrap()
        };

        self.storage_cache
            .insert(key.clone(), (storage.clone(), false));
        self.initial_storage_hashes.insert(key, hash);

        Some(
            borsh::BorshDeserialize::deserialize(&mut storage.as_slice())
                .expect("Expected to deserialize"),
        )
    }

    pub fn set_storage<T: borsh::BorshSerialize>(&mut self, key: String, data: T) {
        let storage_bytes =
            borsh::BorshSerialize::try_to_vec(&data).expect("Expected to serialize");

        self.storage_cache
            .insert(key.clone(), (storage_bytes.clone(), true));
    }

    fn send_storage_update(key: String, storage: Vec<u8>) -> HashDigest {
        let hash = {
            let algorithm = &mut Sha256::default();
            algorithm.update(&storage);
            algorithm.finalize_reset().as_slice().try_into().unwrap()
        };

        let request = SetStorageRequest {
            key: key.clone(),
            hash,
            storage,
        };

        let to_host = borsh::BorshSerialize::try_to_vec(&request).expect("Expected to serialize");

        risc0_zkvm::guest::env::syscall(SET_STORAGE_CALL, &to_host, &mut []);

        hash
    }

    pub fn commit<T: borsh::BorshSerialize>(self, output: T) {
        let Env {
            signer: _,
            caller: _,
            contract: _,
            attached_gas: _,
            call_hash,
            initial_storage_hashes,
            storage_cache,
            cross_calls_hashes,
        } = self;

        let output = borsh::BorshSerialize::try_to_vec(&output).expect("Expected to serialize");

        let storage_writes = storage_cache
            .into_iter()
            .filter_map(|(key, (storage, was_changed))| {
                if was_changed {
                    let hash = Env::send_storage_update(key.clone(), storage.clone());
                    Some((key.clone(), hash))
                } else {
                    None
                }
            })
            .collect();

        let outcome = ExecutionOutcome {
            output,
            call_hash: call_hash,
            storage_reads: initial_storage_hashes,
            storage_writes,
            cross_calls_hashes: cross_calls_hashes,
        };

        risc0_zkvm::guest::env::commit_slice(
            &borsh::BorshSerialize::try_to_vec(&outcome).expect("Expected to serialize"),
        )
    }

    /// Get EVM address by AccountId
    pub fn get_evm_address(&self, account_id: AccountId) -> eth_primitive_types::H160 {
        let mut response = [0u32; 20];

        risc0_zkvm::guest::env::syscall(
            GET_ACCOUNT_MAPPING,
            account_id.to_string().as_bytes(),
            &mut response,
        );

        let response: [u8; 20] =
            risc0_zkvm::serde::from_slice(&response).expect("Expected to deserialize");

        eth_primitive_types::H160::from_slice(&response)
    }
}

/// Returns the current call signer
pub fn signer() -> AccountId {
    ENV.lock().unwrap().as_ref().unwrap().signer()
}

/// Returns the current call sender
pub fn caller() -> AccountId {
    ENV.lock().unwrap().as_ref().unwrap().caller()
}

/// Returns the current call contract
pub fn contract() -> AccountId {
    ENV.lock().unwrap().as_ref().unwrap().contract()
}

/// Makes a cross-contract call
pub fn cross_contract_call<T: borsh::BorshSerialize>(
    account: AccountId,
    method: String,
    attached_gas: u64,
    args: T,
) -> ExecutionOutcome {
    let req = CrossContractCallRequest::new(account, method, args, attached_gas);
    ENV.lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .cross_contract_call(req)
}

/// Makes a cross-contract call with raw input
pub fn cross_contract_call_raw(
    account: AccountId,
    method: String,
    attached_gas: u64,
    args: Vec<u8>,
) -> ExecutionOutcome {
    let req = CrossContractCallRequest::new_raw(account, method, args, attached_gas);
    ENV.lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .cross_contract_call(req)
}

/// Returns the storage value for the given key, return None if storage is not exist
pub fn get_storage<T: BorshDeserialize>(key: StorageKey) -> Option<T> {
    ENV.lock().unwrap().as_mut().unwrap().get_storage(key)
}

pub fn set_storage<T: borsh::BorshSerialize>(key: String, data: T) {
    ENV.lock().unwrap().as_mut().unwrap().set_storage(key, data)
}

pub fn commit<T: borsh::BorshSerialize>(output: T) {
    ENV.lock().unwrap().take().unwrap().commit(output)
}

/// Get EVM address by AccountId
pub fn get_evm_address(account_id: AccountId) -> eth_primitive_types::H160 {
    ENV.lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .get_evm_address(account_id)
}
