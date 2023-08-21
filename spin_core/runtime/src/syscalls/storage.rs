use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use risc0_zkvm::sha::rust_crypto::{Digest, Sha256};
use risc0_zkvm::{serde::to_vec, Syscall};
use spin_primitives::syscalls::{GetStorageResponse, SetStorageRequest};
use tracing::{debug, span, Level};

use std::sync::{Arc, RwLock};

use crate::context::ExecutionContext;

fn bytes_to_hex_string(slice: &[u8]) -> String {
    slice.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub struct GetStorageCallHandler {
    context: Arc<RwLock<ExecutionContext>>,
}

impl GetStorageCallHandler {
    pub fn new(context: Arc<RwLock<ExecutionContext>>) -> Self {
        Self { context }
    }
}

impl Syscall for GetStorageCallHandler {
    fn syscall(
        &mut self,
        _syscall: &str,
        syscall_ctx: &mut dyn risc0_zkvm::SyscallContext,
        to_guest: &mut [u32],
    ) -> Result<(u32, u32)> {
        let span = span!(Level::DEBUG, "get_storage call handler");
        let _enter = span.enter();

        let context = self.context.write().unwrap();

        let buf_ptr = syscall_ctx.load_register(risc0_zkvm_platform::syscall::reg_abi::REG_A3);
        let buf_len = syscall_ctx.load_register(risc0_zkvm_platform::syscall::reg_abi::REG_A4);
        let from_guest = syscall_ctx.load_region(buf_ptr, buf_len);
        let key = String::from_utf8(from_guest).unwrap();

        let db_key = format!(
            "committed_storage.{}.{}",
            context.call().account.to_string(),
            key
        );

        let storage = context
            .db
            .get(db_key)
            .expect("Failed to get storage from db")
            .map(|v| v.to_vec());

        // let hash = {
        //     let algorithm = &mut Sha256::default();
        //     algorithm.update(&storage);
        //     algorithm.finalize_reset().as_slice().to_vec()
        // }

        let response = GetStorageResponse { storage };

        let response_bytes = BorshSerialize::try_to_vec(&response).unwrap();

        debug!(contract=?context.call().account, key=?key, "Loading storage");

        let output = to_vec(&response_bytes).unwrap();
        to_guest[0..output.len()].copy_from_slice(&output);

        Ok((0, 0))
    }
}

pub struct SetStorageCallHandler {
    context: Arc<RwLock<ExecutionContext>>,
}

impl SetStorageCallHandler {
    pub fn new(context: Arc<RwLock<ExecutionContext>>) -> Self {
        Self { context }
    }
}

impl risc0_zkvm::Syscall for SetStorageCallHandler {
    fn syscall(
        &mut self,
        _syscall: &str,
        ctx: &mut dyn risc0_zkvm::SyscallContext,
        _to_guest: &mut [u32],
    ) -> Result<(u32, u32)> {
        let span = span!(Level::DEBUG, "set_storage call handler");
        let _enter = span.enter();

        let context = self.context.write().unwrap();

        let buf_ptr = ctx.load_register(risc0_zkvm_platform::syscall::reg_abi::REG_A3);
        let buf_len = ctx.load_register(risc0_zkvm_platform::syscall::reg_abi::REG_A4);
        let from_guest = ctx.load_region(buf_ptr, buf_len);

        let request: SetStorageRequest =
            BorshDeserialize::deserialize(&mut from_guest.as_slice()).unwrap();

        let algorithm = &mut Sha256::default();
        algorithm.update(request.storage.clone());
        let hash2 = algorithm.finalize_reset();
        assert_eq!(request.hash, hash2.as_slice());

        debug!(contract=?context.call().account, key=?request.key, new_hash = bytes_to_hex_string(hash2.as_slice()), "Updating storage");

        let db_key = format!(
            "committed_storage.{}.{}",
            context.call().account.to_string(),
            request.key
        );

        context
            .db
            .insert(db_key, request.storage)
            .expect("Failed to insert storage to db");

        Ok((0, 0))
    }
}
