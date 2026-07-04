//! JSON persistence for the in-memory devnet chain.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::address::Address;
use crate::block::{Block, BlockHeader};
use crate::chain::{ApplyError, Chain, ChainConfig};
use crate::coin::Coin;
use crate::genesis::GenesisConfig;
use crate::hashing::Hash256;
use crate::transaction::Transaction;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChainSnapshot {
    pub config: ChainConfig,
    pub blocks: Vec<BlockSnapshot>,
    pub balances: BTreeMap<String, u64>,
    pub nonces: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockSnapshot {
    pub height: u64,
    pub previous_hash: String,
    pub state_root: String,
    pub transactions_root: String,
    pub timestamp_ms: u64,
    pub proposer: String,
    pub transactions: Vec<TransactionSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionSnapshot {
    pub from: String,
    pub to: String,
    pub amount_micro_ac: u64,
    pub fee_micro_ac: u64,
    pub nonce: u64,
    pub memo_commitment: String,
}

fn hash_to_hex(hash: Hash256) -> String {
    crate::hashing::hex(&hash)
}

fn hex_to_hash(value: &str) -> Result<Hash256, StorageError> {
    if value.len() != 64 {
        return Err(StorageError::Serde("hash hex must be 64 characters".to_string()));
    }
    let mut out = [0_u8; 32];
    for (index, chunk) in value.as_bytes().chunks(2).enumerate() {
        if index >= 32 {
            break;
        }
        let pair = std::str::from_utf8(chunk).map_err(|e| StorageError::Serde(e.to_string()))?;
        out[index] = u8::from_str_radix(pair, 16).map_err(|e| StorageError::Serde(e.to_string()))?;
    }
    Ok(out)
}

impl Chain {
    pub fn to_snapshot(&self) -> ChainSnapshot {
        let (balances, nonces) = self.snapshot_state();
        ChainSnapshot {
            config: self.config().clone(),
            blocks: self.blocks().iter().map(BlockSnapshot::from_block).collect(),
            balances,
            nonces,
        }
    }

    pub fn from_snapshot(snapshot: ChainSnapshot) -> Result<Self, StorageError> {
        let blocks = snapshot
            .blocks
            .into_iter()
            .map(BlockSnapshot::into_block)
            .collect::<Result<Vec<_>, _>>()?;

        Chain::from_persisted(snapshot.config, blocks, snapshot.balances, snapshot.nonces)
            .map_err(StorageError::Apply)
    }
}

impl BlockSnapshot {
    fn from_block(block: &Block) -> Self {
        Self {
            height: block.header.height,
            previous_hash: hash_to_hex(block.header.previous_hash),
            state_root: hash_to_hex(block.header.state_root),
            transactions_root: hash_to_hex(block.header.transactions_root),
            timestamp_ms: block.header.timestamp_ms,
            proposer: block.header.proposer.clone(),
            transactions: block
                .transactions
                .iter()
                .map(TransactionSnapshot::from_transaction)
                .collect(),
        }
    }

    fn into_block(self) -> Result<Block, StorageError> {
        Ok(Block {
            header: BlockHeader {
                height: self.height,
                previous_hash: hex_to_hash(&self.previous_hash)?,
                state_root: hex_to_hash(&self.state_root)?,
                transactions_root: hex_to_hash(&self.transactions_root)?,
                timestamp_ms: self.timestamp_ms,
                proposer: self.proposer,
            },
            transactions: self
                .transactions
                .into_iter()
                .map(TransactionSnapshot::into_transaction)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TransactionSnapshot {
    fn from_transaction(tx: &Transaction) -> Self {
        Self {
            from: tx.from.as_str().to_string(),
            to: tx.to.as_str().to_string(),
            amount_micro_ac: tx.amount.micro_ac(),
            fee_micro_ac: tx.fee.micro_ac(),
            nonce: tx.nonce,
            memo_commitment: hash_to_hex(tx.memo_commitment),
        }
    }

    fn into_transaction(self) -> Result<Transaction, StorageError> {
        Ok(Transaction {
            from: Address::new(self.from).map_err(StorageError::Address)?,
            to: Address::new(self.to).map_err(StorageError::Address)?,
            amount: Coin::from_micro_ac(self.amount_micro_ac).ok_or(StorageError::InvalidBalance)?,
            fee: Coin::from_micro_ac(self.fee_micro_ac).ok_or(StorageError::InvalidBalance)?,
            nonce: self.nonce,
            memo_commitment: hex_to_hash(&self.memo_commitment)?,
        })
    }
}

pub fn save_chain(chain: &Chain, path: impl AsRef<Path>) -> Result<(), StorageError> {
    let snapshot = chain.to_snapshot();
    let json = serde_json::to_string_pretty(&snapshot).map_err(|e| StorageError::Serde(e.to_string()))?;
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent).map_err(|e| StorageError::Io(e.to_string()))?;
    }
    fs::write(path, json).map_err(|e| StorageError::Io(e.to_string()))
}

pub fn load_chain(path: impl AsRef<Path>) -> Result<Chain, StorageError> {
    let json = fs::read_to_string(path.as_ref()).map_err(|e| StorageError::Io(e.to_string()))?;
    let snapshot: ChainSnapshot = serde_json::from_str(&json).map_err(|e| StorageError::Serde(e.to_string()))?;
    Chain::from_snapshot(snapshot)
}

pub fn init_devnet_chain(
    data_dir: impl AsRef<Path>,
    timestamp_ms: u64,
) -> Result<Chain, StorageError> {
    let treasury = Address::new("abyss1dev_treasury").map_err(StorageError::Address)?;
    let chain = Chain::from_genesis(
        ChainConfig::default(),
        GenesisConfig::single_treasury(treasury),
        timestamp_ms,
    )
    .map_err(StorageError::Apply)?;
    let path = data_dir.as_ref().join("chain.json");
    save_chain(&chain, path)?;
    Ok(chain)
}

#[derive(Clone, Debug)]
pub enum StorageError {
    Address(crate::address::AddressError),
    Apply(ApplyError),
    InvalidBalance,
    Io(String),
    Serde(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Address(err) => write!(f, "{err}"),
            Self::Apply(err) => write!(f, "{err:?}"),
            Self::InvalidBalance => write!(f, "invalid balance in snapshot"),
            Self::Io(err) => write!(f, "{err}"),
            Self::Serde(err) => write!(f, "{err}"),
        }
    }
}

impl From<crate::address::AddressError> for StorageError {
    fn from(value: crate::address::AddressError) -> Self {
        Self::Address(value)
    }
}

impl From<ApplyError> for StorageError {
    fn from(value: ApplyError) -> Self {
        Self::Apply(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashing::{hex, ZERO_HASH};
    use crate::transaction::Transaction;

    #[test]
    fn round_trips_chain_through_json_snapshot() {
        let treasury = Address::new("treasury").unwrap();
        let alice = Address::new("alice").unwrap();
        let mut chain = Chain::from_genesis(
            ChainConfig::default(),
            GenesisConfig::single_treasury(treasury.clone()),
            0,
        )
        .unwrap();
        let tx = Transaction::new(
            treasury,
            alice,
            Coin::from_ac(5).unwrap(),
            Coin::ZERO,
            0,
        );
        chain.produce_block("validator-1", 1_000, vec![tx]).unwrap();

        let restored = Chain::from_snapshot(chain.to_snapshot()).unwrap();
        assert_eq!(restored.height(), chain.height());
        assert_eq!(restored.tip_hash(), chain.tip_hash());
        assert_eq!(
            restored.balance_of(&Address::new("alice").unwrap()),
            Coin::from_ac(5).unwrap()
        );
    }

    #[test]
    fn genesis_block_hashes_use_zero_placeholders() {
        let chain = Chain::from_genesis(
            ChainConfig::default(),
            GenesisConfig::single_treasury(Address::new("treasury").unwrap()),
            42,
        )
        .unwrap();
        let block = &chain.blocks()[0];
        assert_eq!(block.header.previous_hash, ZERO_HASH);
        assert_eq!(block.header.state_root, ZERO_HASH);
        assert_eq!(block.header.transactions_root, ZERO_HASH);
        assert_eq!(hex(&block.header.previous_hash), hex(&ZERO_HASH));
    }
}
