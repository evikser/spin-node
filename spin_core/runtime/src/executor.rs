use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use risc0_zkvm::{serde::to_vec, Executor, ExecutorEnv};
use spin_primitives::{
    syscalls::{
        CROSS_CONTRACT_CALL, GET_ACCOUNT_MAPPING, GET_ENV_CALL, GET_STORAGE_CALL, SET_STORAGE_CALL,
    },
    AccountId,
};
use tracing::debug;

use crate::syscalls::{
    accounts_mapping::AccountsMappingHandler, cross_contract::CrossContractCallHandler,
    env::GetEnvCallHandler,
};
use crate::{
    context::ExecutionContext,
    syscalls::storage::{GetStorageCallHandler, SetStorageCallHandler},
};

const MAX_MEMORY: u32 = 0x10000000;
const PAGE_SIZE: u32 = 0x400;

fn load_contract(account: AccountId) -> Result<Vec<u8>> {
    std::fs::read(format!("./state/contracts/{}", &account.to_string()))
        .with_context(|| format!("Can't read contract {}", account.to_string()))
}

struct ContractLogger {
    context: Arc<RwLock<ExecutionContext>>,
}

impl ContractLogger {
    fn new(context: Arc<RwLock<ExecutionContext>>) -> Self {
        Self { context }
    }
}

impl std::io::Write for ContractLogger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let context = self.context.write().unwrap();

        // TODO: handle non-utf8 logs
        let msg = String::from_utf8(buf.to_vec()).unwrap();

        tracing::debug!(contract = ?context.contract(),msg, "📜 Contract log");

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        unimplemented!()
    }
}

pub fn execute(context: Arc<RwLock<ExecutionContext>>) -> Result<risc0_zkvm::Session> {
    let mut exec = {
        let ctx = context.read().unwrap();
        debug!(contract = ?ctx.contract(), "Executing contract");

        let env = ExecutorEnv::builder()
            .add_input(&to_vec(&ctx.call().into_bytes())?)
            .session_limit(Some(ctx.attached_gas().try_into().unwrap()))
            .syscall(GET_ENV_CALL, GetEnvCallHandler::new(context.clone()))
            .syscall(
                CROSS_CONTRACT_CALL,
                CrossContractCallHandler::new(context.clone()),
            )
            .syscall(
                GET_STORAGE_CALL,
                GetStorageCallHandler::new(context.clone()),
            )
            .syscall(
                SET_STORAGE_CALL,
                SetStorageCallHandler::new(context.clone()),
            )
            .syscall(
                GET_ACCOUNT_MAPPING,
                AccountsMappingHandler::new(context.clone()),
            )
            .stdout(ContractLogger::new(context.clone()))
            .build()?;

        let elf = if ctx.contract() == &AccountId::new(String::from("evm")) {
            meta_contracts::EVM_METACONTRACT_ELF.to_vec()
        } else {
            load_contract(ctx.contract().clone())
                .context(format!("Load contract {:?}", ctx.contract()))?
        };

        let program = risc0_zkvm::Program::load_elf(&elf, MAX_MEMORY)?;
        let image = risc0_zkvm::MemoryImage::new(&program, PAGE_SIZE)?;
        risc0_zkvm::Executor::new(env, image)
    };

    let session = exec.run()?;
    {
        let cycles = 2u64.pow(
            session
                .segments
                .iter()
                .map(|s| s.resolve().unwrap().po2)
                .sum::<usize>()
                .try_into()
                .unwrap(),
        );
        let mut ctx = context.write().unwrap();
        ctx.set_gas_usage(cycles);
    }

    // debug!("Start proving...");
    // let _receipt = session.prove();
    // debug!("Proved");

    Ok(session)
}
