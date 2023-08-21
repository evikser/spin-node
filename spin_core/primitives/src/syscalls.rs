use borsh::{BorshDeserialize, BorshSerialize};

use crate::{AccountId, Digest, StorageKey};

pub const CROSS_CONTRACT_CALL: risc0_zkvm_platform::syscall::SyscallName = unsafe {
    risc0_zkvm_platform::syscall::SyscallName::from_bytes_with_nul(
        concat!("spinvm", "::", "CROSS_CONTRACT_CALL", "\0").as_ptr(),
    )
};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct CrossContractCallRequest {
    pub contract: AccountId,
    pub method: String,
    pub args: Vec<u8>,
    pub attached_gas: u64,
}

impl CrossContractCallRequest {
    pub fn new<T: BorshSerialize>(
        contract: AccountId,
        method: String,
        args: T,
        attached_gas: u64,
    ) -> Self
    where
        T: BorshSerialize,
    {
        Self {
            contract,
            method,
            args: BorshSerialize::try_to_vec(&args).expect("Expected to serialize"),
            attached_gas,
        }
    }

    pub fn new_raw(contract: AccountId, method: String, args: Vec<u8>, attached_gas: u64) -> Self {
        Self {
            contract,
            method,
            args,
            attached_gas,
        }
    }
}

pub const GET_STORAGE_CALL: risc0_zkvm_platform::syscall::SyscallName = unsafe {
    risc0_zkvm_platform::syscall::SyscallName::from_bytes_with_nul(
        concat!("spinvm", "::", "GET_STORAGE", "\0").as_ptr(),
    )
};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct GetStorageRequest {
    pub key: StorageKey,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct GetStorageResponse {
    pub storage: Option<Vec<u8>>,
}

pub const SET_STORAGE_CALL: risc0_zkvm_platform::syscall::SyscallName = unsafe {
    risc0_zkvm_platform::syscall::SyscallName::from_bytes_with_nul(
        concat!("spinvm", "::", "SET_STORAGE", "\0").as_ptr(),
    )
};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SetStorageRequest {
    pub key: StorageKey,
    pub hash: Digest,
    pub storage: Vec<u8>,
}

pub const GET_ACCOUNT_MAPPING: risc0_zkvm_platform::syscall::SyscallName = unsafe {
    risc0_zkvm_platform::syscall::SyscallName::from_bytes_with_nul(
        concat!("spinvm", "::", "GET_ACCOUNT_MAPPING", "\0").as_ptr(),
    )
};
