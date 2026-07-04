//! Core protocol primitives for the ABYSS devnet.
//!
//! This crate intentionally starts with a small transparent devnet model. The
//! shielded note system, zk circuits, and production cryptography will replace
//! the placeholder hashing and transparent transaction model in later phases.

pub mod address;
pub mod block;
pub mod chain;
pub mod coin;
pub mod genesis;
pub mod hashing;
pub mod mempool;
pub mod storage;
pub mod transaction;

pub use address::Address;
pub use block::{Block, BlockHeader};
pub use chain::{ApplyError, Chain, ChainConfig};
pub use coin::{Coin, COIN, MAX_SUPPLY};
pub use genesis::GenesisConfig;
pub use mempool::{Mempool, MempoolError};
pub use storage::{init_devnet_chain, load_chain, save_chain, StorageError};
pub use transaction::{Transaction, TransactionId};
