use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use abyss_core::{hashing, Chain, ChainConfig, Coin, GenesisConfig, Mempool, Transaction};
use abyss_tokenomics::{
    parse_usd_to_cents, usd_cents_to_string, ContributionReceipt, InvestorProfile, KycStatus,
    TokenomicsPlan,
};
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
        Some("presale") => match args.next().as_deref() {
            Some("quote") => quote_presale(args.collect()),
            Some(command) => Err(format!("unknown presale command '{command}'")),
            None => Err("missing presale command".to_string()),
        },
        Some("tokenomics") => print_tokenomics(),
        Some("help") | Some("--help") | Some("-h") | None => {
            print_help();
            Ok(())
        }
        Some(command) => Err(format!("unknown command '{command}'")),
    }
}

fn quote_presale(args: Vec<String>) -> Result<(), String> {
    let request = PresaleQuoteRequest::parse(args)?;
    let plan = TokenomicsPlan::abyss_default();
    plan.validate()
        .map_err(|err| format!("invalid tokenomics plan: {err:?}"))?;
    let round = plan
        .sale_round(&request.round_id)
        .ok_or_else(|| format!("unknown sale round '{}'", request.round_id))?;
    let investor = InvestorProfile {
        investor_id: request.investor_id,
        jurisdiction: request.jurisdiction,
        kyc_status: if request.kyc_approved {
            KycStatus::Approved
        } else {
            KycStatus::Pending
        },
        accredited_or_professional: request.professional,
        max_contribution_usd_cents: request.max_contribution_usd_cents,
    };
    let receipt = ContributionReceipt::create(&investor, round, request.contribution_usd_cents)
        .map_err(|err| format!("presale quote rejected: {err:?}"))?;

    println!("ABYSS presale quote");
    println!("investor_id: {}", receipt.investor_id);
    println!("jurisdiction: {}", receipt.jurisdiction);
    println!("round: {} ({})", receipt.round_name, receipt.round_id);
    println!(
        "contribution: {}",
        usd_cents_to_string(receipt.contribution_usd_cents)
    );
    println!("token_amount: {}", receipt.token_amount);
    println!("price: {}", usd_cents_to_string(round.price_usd_cents));
    println!("lockup_months: {}", receipt.vesting.cliff_months);
    println!(
        "unlocked_at_0_months: {}",
        receipt.vesting.unlocked_amount(receipt.token_amount, 0)
    );
    println!(
        "unlocked_after_lockup: {}",
        receipt
            .vesting
            .unlocked_amount(receipt.token_amount, receipt.vesting.cliff_months)
    );
    println!("status: quote only; do not accept funds without legal review");

    Ok(())
}

#[derive(Clone, Debug)]
struct PresaleQuoteRequest {
    investor_id: String,
    jurisdiction: String,
    round_id: String,
    contribution_usd_cents: u64,
    max_contribution_usd_cents: u64,
    kyc_approved: bool,
    professional: bool,
}

impl PresaleQuoteRequest {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut investor_id = "demo-investor".to_string();
        let mut jurisdiction = "EU".to_string();
        let mut round_id = "public-stage-1".to_string();
        let mut amount = None;
        let mut max = 10_000_00;
        let mut kyc_approved = false;
        let mut professional = false;

        for arg in args {
            if arg == "--kyc-approved" {
                kyc_approved = true;
                continue;
            }
            if arg == "--professional" {
                professional = true;
                continue;
            }

            let (key, value) = arg
                .split_once('=')
                .ok_or_else(|| format!("invalid argument '{arg}', expected --key=value"))?;

            match key {
                "--investor" => investor_id = value.to_string(),
                "--jurisdiction" => jurisdiction = value.to_string(),
                "--round" => round_id = value.to_string(),
                "--amount" => {
                    amount = Some(
                        parse_usd_to_cents(value)
                            .map_err(|err| format!("invalid --amount: {err:?}"))?,
                    )
                }
                "--max" => {
                    max = parse_usd_to_cents(value)
                        .map_err(|err| format!("invalid --max: {err:?}"))?
                }
                _ => return Err(format!("unknown argument '{key}'")),
            }
        }

        Ok(Self {
            investor_id,
            jurisdiction,
            round_id,
            contribution_usd_cents: amount.ok_or("missing --amount=<usd>")?,
            max_contribution_usd_cents: max,
            kyc_approved,
            professional,
        })
    }
}

fn print_tokenomics() -> Result<(), String> {
    let plan = TokenomicsPlan::abyss_default();
    plan.validate()
        .map_err(|err| format!("invalid tokenomics plan: {err:?}"))?;

    println!("ABYSS tokenomics");
    println!("symbol: {}", plan.symbol);
    println!("max_supply: {}", plan.max_supply);
    println!();
    println!("allocations:");
    for allocation in &plan.allocations {
        println!(
            "  - {}: {} bps / {}",
            allocation.name,
            allocation.basis_points,
            allocation.amount()
        );
    }

    println!();
    println!("sale_rounds:");
    for round in &plan.sale_rounds {
        println!(
            "  - {} [{}]: cap {}, price {}, min ${}, lockup {} months, raise cap {}",
            round.name,
            round.id,
            round.token_cap,
            usd_cents_to_string(round.price_usd_cents),
            round.minimum_ticket_usd,
            round.lockup_months,
            usd_cents_to_string(round.raise_cap_usd_cents())
        );
    }

    if let Some(total) = plan.total_sale_cap_usd_cents() {
        println!();
        println!("maximum_sale_raise: {}", usd_cents_to_string(total));
    }

    Ok(())
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
    println!("  abyss-node presale quote --amount=<usd> [--round=<id>] [--kyc-approved]");
    println!("  abyss-node tokenomics   print the current AC tokenomics plan");
    println!("  abyss-node help     show this help");
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}
