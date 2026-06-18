# ABYSS Implementation Roadmap

## Current Milestone: Devnet Skeleton

The first implementation target is a transparent local devnet. It exists only
to validate chain structure, supply rules, block production, balances, nonces,
and developer workflow.

Completed in the initial scaffold:

- Rust workspace layout;
- `abyss-core` crate;
- `abyss-node` CLI crate;
- native AC supply cap;
- address validation;
- basic transactions;
- blocks and block headers;
- in-memory chain state;
- genesis allocation;
- nonce and balance checks;
- local devnet simulation command;
- unit tests for core behavior.
- development key/account module;
- wallet account policy module;
- mempool;
- early validator quorum certificate model.
- tokenomics planning crate;
- presale strategy draft.

## Next Milestone: Durable Local Node

- add persistent storage;
- add mempool;
- add node config file;
- add peer identity keys;
- add structured logs;
- add JSON-RPC or gRPC API;
- add CLI wallet commands.
- add tokenomics JSON export;
- add investor whitelist and vesting schedule models.
- connect presale quote simulation to the website investor form.

## Privacy Milestone

- implement shielded note model;
- implement note commitments;
- implement nullifiers;
- implement Merkle note tree;
- select zk proving backend;
- create first transfer circuit;
- add view keys.

## Consensus Milestone

- multi-node local network;
- validator set;
- block proposal;
- vote messages;
- BFT finality prototype;
- slashing evidence model.
