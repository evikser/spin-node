use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use risc0_zkvm::{serde::to_vec, Executor, ExecutorEnv};
use spin_primitives::{
    syscalls::{CROSS_CONTRACT_CALL, GET_ACCOUNT_MAPPING, GET_STORAGE_CALL, SET_STORAGE_CALL},
    AccountId, Digest, SignedTransaction, Transaction, SYSTEM_META_CONTRACT_ACCOUNT_ID,
};
use tracing::debug;

use crate::syscalls::{
    accounts_mapping::AccountsMappingHandler, cross_contract::CrossContractCallHandler,
};
use crate::{
    context::ExecutionContext,
    syscalls::storage::{GetStorageCallHandler, SetStorageCallHandler},
};

const MAX_MEMORY: u32 = 0x10000000;
const PAGE_SIZE: u32 = 0x400;

fn load_contract(db: &sled::Db, account: AccountId) -> Result<Vec<u8>> {
    let db_key = format!(
        "committed_storage.{}.code.{}",
        SYSTEM_META_CONTRACT_ACCOUNT_ID,
        account.to_string(),
    );

    let code = db
        .get(db_key)
        .expect("Failed to get storage from db")
        .map(|v| v.to_vec())
        .expect("Account not found");

    let code = BorshDeserialize::deserialize(&mut code.as_slice()).unwrap();

    Ok(code)
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

        tracing::debug!(contract = ?context.call().account, msg, "📜 Contract log");

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        unimplemented!()
    }
}

pub fn bootstrap_tx(db: sled::Db, signed_tx: SignedTransaction) -> Result<risc0_zkvm::Session> {
    let context = Arc::new(RwLock::new(ExecutionContext::new(
        spin_primitives::ContractEntrypointContext {
            account: signed_tx.tx.body.contract.clone(),
            method: signed_tx.tx.body.method.clone(),
            args: signed_tx.tx.body.args.clone(),
            attached_gas: signed_tx.tx.body.attached_gas,
            sender: signed_tx.tx.body.signer.clone(),
            signer: signed_tx.tx.body.signer.clone(),
        },
        db,
    )));

    let mut exec = {
        let ctx = context.read().unwrap();
        debug!(contract = ?ctx.call().account, "Executing contract");

        let tx_bytes = signed_tx.try_to_vec()?;

        let env = ExecutorEnv::builder()
            .add_input(&to_vec(&tx_bytes)?)
            .session_limit(Some(ctx.call().attached_gas.try_into().unwrap()))
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

        let elf = meta_contracts::ROOT_METACONTRACT_ELF.to_vec();

        let program = risc0_zkvm::Program::load_elf(&elf, MAX_MEMORY)?;
        let image = risc0_zkvm::MemoryImage::new(&program, PAGE_SIZE)?;
        risc0_zkvm::LocalExecutor::new(env, image, program.entry)
    };

    let session = exec.run()?;
    {
        // let cycles = 2u64.pow(
        //     session
        //         .segments
        //         .iter()
        //         .map(|s| s.resolve().unwrap().po2)
        //         .sum::<usize>()
        //         .try_into()
        //         .unwrap(),
        // );
        // let mut ctx = context.write().unwrap();
        // ctx.set_gas_usage(cycles);
    }

    Ok(session)
}

pub fn execute(context: Arc<RwLock<ExecutionContext>>) -> Result<risc0_zkvm::Session> {
    let mut exec = {
        let ctx = context.read().unwrap();
        debug!(contract = ?ctx.call().account, "Executing contract");

        let env = ExecutorEnv::builder()
            .add_input(&to_vec(&ctx.call().into_bytes())?)
            .session_limit(Some(ctx.call().attached_gas.try_into().unwrap()))
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

        let elf = if ctx.call().account == AccountId::new(String::from("evm")) {
            meta_contracts::EVM_METACONTRACT_ELF.to_vec()
        } else {
            load_contract(&ctx.db, ctx.call().account.clone())
                .context(format!("Load contract {:?}", ctx.call().account))?
        };

        let program = risc0_zkvm::Program::load_elf(&elf, MAX_MEMORY)?;
        let image = risc0_zkvm::MemoryImage::new(&program, PAGE_SIZE)?;
        risc0_zkvm::LocalExecutor::new(env, image, program.entry)
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
