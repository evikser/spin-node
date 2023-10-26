// use risc0_recursion::{lift, join};
use risc0_zkvm::{prove::get_prover, sha::Digest, default_prover, ExecutorEnv};
use tracing::info;
use anyhow::Context;
use risc0_zkvm::serde::to_vec;

use spin_primitives::{AccountId, ExecutionCommittment};
use spin_runtime::context::ExecutionContext;
use spin_runtime::executor;

use playgrounds::install_tracing;

use std::{sync::{Arc, RwLock}, fs};

const MAX_MEMORY: u32 = 0x10000000;
const PAGE_SIZE: u32 = 0x400;


fn main() {
    install_tracing();

    info!("starting!");

    let input = ethabi::Token::Uint(100.into());

    let env = ExecutorEnv::builder()
        .add_input(&ethabi::encode(&[input]))
        .build().unwrap();

    let elf_bytes = load_contract("../fibonacci.elf");
    let program = risc0_zkvm::Program::load_elf(&elf_bytes, MAX_MEMORY).unwrap();

    let image = risc0_zkvm::MemoryImage::new(&program, PAGE_SIZE).unwrap();
    let mut exec = risc0_zkvm::Executor::new(env, image);

    info!("running...");

    let session = exec.run().expect("run failed");

    // let fib = AccountId::new("fibonacci.spin".to_string());
    // let alice = AccountId::new("alice.spin".to_string());

    // let input = 1u32;
    // // let input = 100_000u32;
    // let ctx = Arc::new(RwLock::new(ExecutionContext::new(
    //     AccountId::new(alice.to_string()),
    //     AccountId::new(alice.to_string()),
    //     AccountId::new(fib.to_string()),
    //     100_000_000,
    //     spin_primitives::FunctionCall::new("fibonacci".into(), input),
    // )));
    
    // info!("executing...");    
    // let session = executor::execute(ctx).unwrap();

    // info!("got {} segments...", session.segments.len());
    // let verifier_ctx = risc0_zkvm::VerifierContext::default();
    
    // no recursion
    let prover = default_prover();
    let receipt = session.prove().unwrap();
    let segment_receipts = receipt.inner.flat();
    let segment_receipt = segment_receipts[0].clone();
    let seal = seal_to_str(&segment_receipt.seal);
    let image_id = format!("0x{}", &session.resolve().unwrap()[0].pre_image.compute_id());
    let post_state_digest = format!("0x{}", &session.resolve().unwrap()[0].post_image_id);
    let journal_hash = journal_to_str(&receipt.journal);

    info!("{:?}", segment_receipt.seal.iter().take(50).collect::<Vec<_>>());

    // recursion
    // let segments = session.resolve().unwrap();
    // let prover = get_prover("$poseidon");
    // let mut rollup_wrapped = None;
    // for receipt in segments {
    //     info!("proving next segment...");
    //     let segment_receipt = prover.prove_segment(&verifier_ctx, &receipt).unwrap();
    //     info!("lifting next segment...");
    //     let rec_receipt = lift(&segment_receipt).unwrap();
    //     info!("joining next segment...");
    //     rollup_wrapped = if rollup_wrapped == None {
    //         Some(rec_receipt)
    //     } else {
    //         Some(join(&rollup_wrapped.unwrap(), &rec_receipt).unwrap())
    //     };
    // }
    // let rollup = rollup_wrapped.unwrap();

    // info!("verifying...");
    // let result = rollup.verify_with_context(&verifier_ctx);
    // info!("verified: {:#?}", result);

    // let seal = seal_to_str(&rollup.seal);
    // let image_id = digest_to_str(&rollup.meta.pre.merkle_root);
    // let post_state_digest = digest_to_str(&rollup.meta.post.merkle_root);
    // let journal_hash = digest_to_str(&rollup.meta.output);

    let path = "/Users/nikita/Develop/spin-node-evikser/simple_contract_state".to_string();
    fs::write(format!("{}/seal.txt", path), &seal).unwrap();
    fs::write(format!("{}/imageId.txt", path), &image_id).unwrap();
    fs::write(format!("{}/postStateDigest.txt", path), &post_state_digest).unwrap();
    fs::write(format!("{}/journalHash.txt", path), &journal_hash).unwrap();

    // info!("seal size: {} kb", rollup.seal.len() / 1024);
    // info!("imageId/pre: {:x?}", rollup.meta.pre.merkle_root);
    // info!("postStateDigest/post: {:x?}", rollup.meta.post.merkle_root);
    // info!("journalHash/output: {:x?}", rollup.meta.output);

    info!("finished!");
}

fn seal_to_str(seal: &Vec<u32>) -> String {
    format!("0x{:08x?}", seal).replace(", ", "").replace("[", "").replace("]", "").to_string()
}

fn journal_to_str(journal: &Vec<u8>) -> String {
    format!("0x{:02x?}", journal).replace(", ", "").replace("[", "").replace("]", "").to_string()
}

fn digest_to_str(digest: &Digest) -> String {
    format!("0x{:?}", digest).replace("Digest(", "").replace(")", "").to_string()
}

fn load_contract(name: &str) -> Vec<u8> {
    std::fs::read(name.clone()).with_context(|| format!("Can't read contract {}", name)).unwrap()
}

fn token_init(token: &AccountId, signer: &AccountId, ticker: String, initial_supply: u128) {
    info!(
        ?token,
        owner = ?signer,
        ticker,
        initial_supply,
        "Creating token"
    );
    let ctx = Arc::new(RwLock::new(ExecutionContext::new(
        signer.clone(),
        signer.clone(),
        token.clone(),
        100_000_000,
        spin_primitives::FunctionCall::new("init".into(), (ticker, initial_supply)),
    )));

    executor::execute(ctx).unwrap();
}

fn transfer(token: &AccountId, from: &AccountId, to: &AccountId, amount: u128) {
    info!(amount, ?to, ?from, "Transfering");
    let ctx = Arc::new(RwLock::new(ExecutionContext::new(
        AccountId::new(from.to_string()),
        AccountId::new(from.to_string()),
        AccountId::new(token.to_string()),
        100_000_000,
        spin_primitives::FunctionCall::new("transfer".into(), (to, amount)),
    )));

    executor::execute(ctx).unwrap();
}

fn token_balance_of(token: &AccountId, account: &AccountId) -> u64 {
    let ctx = Arc::new(RwLock::new(ExecutionContext::new(
        account.clone(),
        account.clone(),
        token.clone(),
        100_000_000,
        spin_primitives::FunctionCall::new("balance_of".into(), account),
    )));

    let s = executor::execute(ctx.clone()).unwrap();

    let committment: ExecutionCommittment =
        borsh::BorshDeserialize::deserialize(&mut s.journal.as_slice()).unwrap();

    let balance: u64 = committment.try_deserialize_output().unwrap();
    balance
}
