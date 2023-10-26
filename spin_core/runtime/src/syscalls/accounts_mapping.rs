use anyhow::Result;
use risc0_zkvm::{serde::to_vec, Syscall};
use tracing::{debug, span, Level};

use std::sync::{Arc, RwLock};

use spin_primitives::AccountId;

use crate::context::ExecutionContext;

pub struct AccountsMappingHandler {
    context: Arc<RwLock<ExecutionContext>>,
}

impl AccountsMappingHandler {
    pub fn new(context: Arc<RwLock<ExecutionContext>>) -> Self {
        Self { context }
    }
}

impl Syscall for AccountsMappingHandler {
    fn syscall(
        &mut self,
        _syscall: &str,
        syscall_ctx: &mut dyn risc0_zkvm::SyscallContext,
        to_guest: &mut [u32],
    ) -> Result<(u32, u32)> {
        let span = span!(Level::DEBUG, "accounts_mapping handler");
        let _enter = span.enter();

        let ctx = self.context.write().unwrap();
        debug!(from_contract=?ctx.contract());

        let buf_ptr = syscall_ctx.load_register(risc0_zkvm_platform::syscall::reg_abi::REG_A3);
        let buf_len = syscall_ctx.load_register(risc0_zkvm_platform::syscall::reg_abi::REG_A4);
        let from_guest = syscall_ctx.load_region(buf_ptr, buf_len).unwrap();

        let account_id = AccountId::new(String::from_utf8(from_guest).unwrap());
        debug!(of_account_id=?account_id);

        let evm_address = ExecutionContext::get_account_evm_address(account_id);
        debug!(evm_address=?evm_address);

        let output: Vec<u32> = to_vec(&evm_address.to_fixed_bytes()).unwrap();

        to_guest[0..output.len()].copy_from_slice(&output);
        Ok((0, 0))
    }
}
