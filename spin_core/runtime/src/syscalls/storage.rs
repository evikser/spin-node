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
        let from_guest = syscall_ctx.load_region(buf_ptr, buf_len).unwrap();
        let key = String::from_utf8(from_guest).unwrap();

        // TODO: don't use files as contract state storage xD
        let state: Vec<u8> = std::fs::read(format!(
            "./state/storage/{}.{}",
            key,
            context.contract().to_string()
        ))
        .unwrap_or_else(|e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                debug!(
                    "No state found for key {:?} in {:?}, creating new",
                    key,
                    context.contract()
                );
                Vec::new()
            }
            _ => todo!(),
        });

        // tracing::warn!("state: {:?}", state);

        let algorithm = &mut Sha256::default();
        algorithm.update(&state);
        let hash = algorithm.finalize_reset();

        let response = GetStorageResponse {
            hash: hash.into(),
            state,
        };

        let response_bytes = BorshSerialize::try_to_vec(&response).unwrap();

        debug!(contract=?context.contract(), key=?key, hash = bytes_to_hex_string(hash.as_slice()), "Loading storage");

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
        let from_guest = ctx.load_region(buf_ptr, buf_len).unwrap();

        let request: SetStorageRequest =
            BorshDeserialize::deserialize(&mut from_guest.as_slice()).unwrap();

        let algorithm = &mut Sha256::default();
        algorithm.update(request.state.clone());
        let hash2 = algorithm.finalize_reset();
        assert_eq!(request.hash, hash2.as_slice());

        debug!(contract=?context.contract(), key=?request.key, new_hash = bytes_to_hex_string(hash2.as_slice()), "Updating storage");

        // TODO: don't use files as contract state storage xD
        std::fs::write(
            format!(
                "./state/storage/{}.{}",
                request.key,
                context.contract().to_string()
            ),
            request.state,
        )
        .unwrap();

        Ok((0, 0))
    }
}
