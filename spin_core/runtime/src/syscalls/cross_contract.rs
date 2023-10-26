use anyhow::Result;
use risc0_zkvm::{serde::to_vec, Syscall};
use tracing::debug;

use std::sync::{Arc, RwLock};

use spin_primitives::ContractCall;

use crate::{context::ExecutionContext, executor};

pub struct CrossContractCallHandler {
    context: Arc<RwLock<ExecutionContext>>,
}

impl CrossContractCallHandler {
    pub fn new(context: Arc<RwLock<ExecutionContext>>) -> Self {
        Self { context }
    }
}

impl Syscall for CrossContractCallHandler {
    fn syscall(
        &mut self,
        _syscall: &str,
        syscall_ctx: &mut dyn risc0_zkvm::SyscallContext,
        to_guest: &mut [u32],
    ) -> Result<(u32, u32)> {
        let mut origin_ctx = self.context.write().unwrap();
        debug!(from_contract=?origin_ctx.contract(), "handling syscall for cross contract call");
        origin_ctx.set_gas_usage(syscall_ctx.get_cycle().try_into().unwrap());

        let buf_ptr = syscall_ctx.load_register(risc0_zkvm_platform::syscall::reg_abi::REG_A3);
        let buf_len = syscall_ctx.load_register(risc0_zkvm_platform::syscall::reg_abi::REG_A4);
        let from_guest = syscall_ctx.load_region(buf_ptr, buf_len).unwrap();

        let call = ContractCall::try_from_bytes(from_guest).expect("Invalid contract call");

        let ccc_ctx = origin_ctx.cross_contract_call(call).unwrap();

        let ccc_session = executor::execute(ccc_ctx.clone()).unwrap(); // TODO: handle error
        let ccc_journal = ccc_session.journal.clone();
        {
            let mut ccc_ctx = ccc_ctx.write().unwrap();
            ccc_ctx.set_execution_session(ccc_session);
        }

        let output: Vec<u32> = to_vec(&ccc_journal).unwrap();

        to_guest[0..output.len()].copy_from_slice(&output);
        Ok((0, 0))
    }
}
