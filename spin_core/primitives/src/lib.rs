use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

pub mod syscalls;

#[derive(Serialize, Deserialize, Debug, BorshSerialize, BorshDeserialize)]
pub struct ContractEntrypointContext {
    pub account: AccountId,
    pub method: String,
    pub args: Vec<u8>,
    pub attached_gas: u64,
    pub sender: AccountId,
    pub signer: AccountId,
}

impl ContractEntrypointContext {
    pub fn new<T: BorshSerialize>(
        account: AccountId,
        method: String,
        args: T,
        attached_gas: u64,
        sender: AccountId,
        signer: AccountId,
    ) -> Self
    where
        T: BorshSerialize,
    {
        Self {
            account,
            method,
            args: BorshSerialize::try_to_vec(&args).expect("Expected to serialize"),
            attached_gas,
            sender,
            signer,
        }
    }

    pub fn try_from_bytes(bytes: Vec<u8>) -> std::io::Result<Self> {
        borsh::BorshDeserialize::deserialize(&mut bytes.as_slice())
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        borsh::BorshSerialize::try_to_vec(&self).expect("Expected to serialize")
    }

    pub fn function_call(&self) -> FunctionCall {
        FunctionCall {
            method: self.method.clone(),
            args: self.args.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, BorshSerialize, BorshDeserialize)]
pub struct FunctionCall {
    pub method: String,
    pub args: Vec<u8>,
}

impl FunctionCall {
    pub fn try_deserialize_args<T: BorshDeserialize>(&self) -> std::io::Result<T> {
        borsh::BorshDeserialize::deserialize(&mut self.args.as_slice())
    }
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    BorshSerialize,
    BorshDeserialize,
    Clone,
    PartialEq,
    Hash,
    PartialOrd,
    Eq,
)]
pub struct AccountId(String);

impl AccountId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn new_evm(address: eth_primitive_types::H160) -> Self {
        Self(format!("{:?}.evm", address))
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl From<eth_primitive_types::H160> for AccountId {
    fn from(address: eth_primitive_types::H160) -> Self {
        Self::new_evm(address)
    }
}

pub type Digest = [u8; 32];

pub type StorageKey = String;

/// Execution outcome of a contract call.
#[derive(Serialize, Deserialize, Debug, BorshSerialize, BorshDeserialize)]
pub struct ExecutionOutcome {
    pub call_hash: Digest,
    pub output: Vec<u8>,
    pub storage_reads: HashMap<StorageKey, Digest>,
    pub storage_writes: HashMap<StorageKey, Digest>,
    pub cross_calls_hashes: Vec<(Digest, Digest)>, // hashes of call and output of cross calls
}

impl ExecutionOutcome {
    pub fn try_from_bytes(bytes: Vec<u8>) -> std::io::Result<Self> {
        borsh::BorshDeserialize::deserialize(&mut bytes.as_slice())
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        borsh::BorshSerialize::try_to_vec(&self).expect("Expected to serialize")
    }

    pub fn try_deserialize_output<T: BorshDeserialize>(&self) -> std::io::Result<T> {
        borsh::BorshDeserialize::deserialize(&mut self.output.as_slice())
    }
}

pub const SYSTEM_META_CONTRACT_ACCOUNT_ID: &str = "spin";
pub const EVM_ETA_CONTRACT_ACCOUNT_ID: &str = "evm";

#[derive(Clone, Serialize, Deserialize, Debug, BorshSerialize, BorshDeserialize)]
pub struct Transaction {
    pub hash: Digest,
    pub body: TransactionBody,
}

impl Transaction {
    pub fn new(body: TransactionBody) -> Self {
        Self {
            hash: body.hash(),
            body,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, BorshSerialize, BorshDeserialize)]
pub struct TransactionBody {
    pub contract: AccountId,
    pub method: String,
    pub args: Vec<u8>,
    pub attached_gas: u64,
    pub signer: AccountId,
    pub origin_block_height: u64,
    pub origin_block_hash: Digest,
    pub deadline: u64,
    pub nonce: u64,
}

impl TransactionBody {
    pub fn hash(&self) -> Digest {
        use sha2::{Digest as _, Sha256};
        let hash = Sha256::digest(self.try_to_vec().unwrap());
        hash.try_into().unwrap()
    }

    pub fn try_deserialize_args<T: BorshDeserialize>(&self) -> std::io::Result<T> {
        borsh::BorshDeserialize::deserialize(&mut self.args.as_slice())
    }
}

#[derive(Serialize, Deserialize, Debug, BorshSerialize, BorshDeserialize)]
pub struct TransactionBuilder {
    pub contract: AccountId,
    pub method: String,
    pub args: Vec<u8>,
    pub attached_gas: u64,
    pub signer: AccountId,
    pub origin_block_height: u64,
    pub origin_block_hash: Digest,
    pub deadline: Option<u64>,
    pub nonce: u64,
}

impl TransactionBuilder {
    pub fn new(
        contract: AccountId,
        method: String,
        args: Vec<u8>,
        attached_gas: u64,
        signer: AccountId,
        origin_block: &Block,
    ) -> Self {
        Self {
            contract,
            method,
            args,
            attached_gas,
            signer,
            origin_block_height: origin_block.height,
            origin_block_hash: origin_block.hash.clone(),
            deadline: None,
            nonce: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    pub fn set_nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn set_deadline(mut self, deadline: u64) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn build(self) -> Transaction {
        let body = TransactionBody {
            contract: self.contract.clone(),
            method: self.method.clone(),
            args: self.args.clone(),
            attached_gas: self.attached_gas,
            signer: self.signer.clone(),
            origin_block_height: self.origin_block_height,
            origin_block_hash: self.origin_block_hash.clone(),
            deadline: self.deadline.unwrap_or(10), // TODO default deadline
            nonce: self.nonce,
        };

        Transaction::new(body)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, BorshSerialize, BorshDeserialize)]
pub struct SignedTransaction {
    pub tx: Transaction,
    pub signature: Vec<u8>, // TODO
}

#[derive(Serialize, Deserialize, Debug, BorshSerialize, BorshDeserialize)]
pub struct Block {
    pub height: u64,
    pub hash: Digest,
    pub parent_hash: Digest,
    pub timestamp: u64,
    pub txs: Vec<SignedTransaction>,
    pub execution_outcomes: HashMap<Digest, ExecutionOutcome>,
    pub sessions: HashMap<Digest, String>, // TODO: replace json to struct
}
