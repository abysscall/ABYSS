use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use abyss_core::{hashing, Chain, ChainConfig, Coin, GenesisConfig, Mempool, Transaction};
use abyss_wallet::WalletAccount;

fn main() {
    let exit_code = match run() {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("abyss-node: {err}");
            1
        }
    };

    std::process::exit(exit_code);
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("account") => match args.next().as_deref() {
            Some("new") => {
                let label = args.next().unwrap_or_else(|| "default".to_string());
                create_account(&label);
                Ok(())
            }
            Some(command) => Err(format!("unknown account command '{command}'")),
            None => Err("missing account command".to_string()),
        },
        Some("devnet") => run_devnet(),
        Some("help") | Some("--help") | Some("-h") | None => {
            print_help();
            Ok(())
        }
        Some(command) => Err(format!("unknown command '{command}'")),
    }
}

fn create_account(label: &str) {
    let account = WalletAccount::generate(label);
    println!("ABYSS dev account created");
    println!("label: {}", account.label());
    println!("address: {}", account.address());
    println!("public_key: {}", account.public_key());
    println!("agent_permissions: {:?}", account.agent_policy().permissions());
    println!("warning: dev account only; production key storage is not implemented yet");
}

fn run_devnet() -> Result<(), String> {
    let treasury_account = WalletAccount::from_dev_seed("treasury", "abyss:genesis:treasury");
    let alice_account = WalletAccount::from_dev_seed("alice", "abyss:dev:alice");
    let bob_account = WalletAccount::from_dev_seed("bob", "abyss:dev:bob");
    let treasury = treasury_account.address();
    let alice = alice_account.address();
    let bob = bob_account.address();

    let mut chain = Chain::from_genesis(
        ChainConfig::default(),
        GenesisConfig::single_treasury(treasury.clone()),
        now_ms(),
    )
    .map_err(|err| format!("failed to create genesis: {err:?}"))?;

    let tx1 = Transaction::new(
        treasury.clone(),
        alice.clone(),
        Coin::from_ac(1_000).ok_or("invalid amount")?,
        Coin::from_micro_ac(10_000).ok_or("invalid fee")?,
        chain.next_nonce(&treasury),
    );
    let mut mempool = Mempool::new();
    mempool
        .insert(tx1)
        .map_err(|err| format!("failed to insert tx1 into mempool: {err:?}"))?;

    chain
        .produce_block("abyss-validator-1", now_ms(), mempool.drain_for_block(128))
        .map_err(|err| format!("failed to produce block 1: {err:?}"))?;

    let mut alice_agent = alice_account.clone();
    alice_agent
        .agent_policy_mut()
        .grant(abyss_wallet::AgentPermission::ExecuteLimitedTrades);
    alice_agent
        .agent_policy_mut()
        .set_agent_trade_limit(Coin::from_ac(250).ok_or("invalid agent limit")?);

    let tx2 = alice_agent.create_agent_payment(
        bob.clone(),
        Coin::from_ac(125).ok_or("invalid amount")?,
        Coin::from_micro_ac(2_500).ok_or("invalid fee")?,
        chain.next_nonce(&alice),
    )
    .map_err(|err| format!("agent policy rejected payment: {err:?}"))?;
    mempool
        .insert(tx2)
        .map_err(|err| format!("failed to insert tx2 into mempool: {err:?}"))?;

    chain
        .produce_block("abyss-validator-1", now_ms(), mempool.drain_for_block(128))
        .map_err(|err| format!("failed to produce block 2: {err:?}"))?;

    println!("ABYSS devnet booted");
    println!("chain_id: {}", chain.config().chain_id);
    println!("height: {}", chain.height());
    println!("tip_hash: {}", hashing::hex(&chain.tip_hash()));
    println!("treasury_address: {treasury}");
    println!("alice_address: {alice}");
    println!("bob_address: {bob}");
    println!("treasury: {}", chain.balance_of(&treasury));
    println!("alice: {}", chain.balance_of(&alice));
    println!("bob: {}", chain.balance_of(&bob));

    Ok(())
}

fn print_help() {
    println!("ABYSS Node");
    println!();
    println!("Usage:");
    println!("  abyss-node account new [label]   create a development account");
    println!("  abyss-node devnet   run a local in-memory devnet simulation");
    println!("  abyss-node help     show this help");
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}
