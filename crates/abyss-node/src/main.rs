use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use abyss_core::{hashing, Chain, ChainConfig, Coin, GenesisConfig, Mempool, Transaction};
use abyss_tokenomics::{
    parse_usd_to_cents, usd_cents_to_string, BuybackOffer, ContributionReceipt, DexFinalSale,
    InvestorProfile, KycStatus, TeamVesting, TokenomicsPlan,
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
            Some("buyback") => quote_buyback(args.collect()),
            Some("dex-quote") => quote_dex_final_sale(args.collect()),
            Some(command) => Err(format!("unknown presale command '{command}'")),
            None => Err("missing presale command".to_string()),
        },
        Some("tokenomics") => print_tokenomics(args.collect()),
        Some("vesting") => print_vesting(args.collect()),
        Some("help") | Some("--help") | Some("-h") | None => {
            print_help();
            Ok(())
        }
        Some(command) => Err(format!("unknown command '{command}'")),
    }
}

// ── presale quote (Standard sale rounds: Sale to Investors, Pre-Sale, Stage 1-3) ──

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

    if request.json {
        let json = format!(
            "{{\n  \"investor_id\": \"{}\",\n  \"jurisdiction\": \"{}\",\n  \"round_id\": \"{}\",\n  \"round_name\": \"{}\",\n  \"contribution_usd_cents\": {},\n  \"price_usd_cents\": {},\n  \"token_amount_ac\": \"{}\",\n  \"lockup_months\": {},\n  \"status\": \"quote only; do not accept funds without legal review\"\n}}",
            json_escape(&receipt.investor_id),
            json_escape(&receipt.jurisdiction),
            receipt.round_id,
            json_escape(receipt.round_name),
            receipt.contribution_usd_cents,
            round.price_usd_cents,
            receipt.token_amount,
            receipt.vesting.cliff_months,
        );
        println!("{json}");
        return Ok(());
    }

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
    json: bool,
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
        let mut json = false;

        for arg in args {
            if arg == "--kyc-approved" {
                kyc_approved = true;
                continue;
            }
            if arg == "--professional" {
                professional = true;
                continue;
            }
            if arg == "--json" {
                json = true;
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
            json,
        })
    }
}

// ── presale buyback (Investor Buyback offer: sell back at $3.00) ──

fn quote_buyback(args: Vec<String>) -> Result<(), String> {
    let mut tokens_ac: Option<u64> = None;
    let mut json = false;

    for arg in args {
        if arg == "--json" {
            json = true;
            continue;
        }
        let (key, value) = arg
            .split_once('=')
            .ok_or_else(|| format!("invalid argument '{arg}', expected --key=value"))?;
        match key {
            "--tokens" => {
                tokens_ac = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| format!("invalid --tokens value '{value}'"))?,
                )
            }
            _ => return Err(format!("unknown argument '{key}'")),
        }
    }

    let tokens_ac = tokens_ac.ok_or("missing --tokens=<ac amount>")?;
    let buyback = BuybackOffer::abyss_default();
    let tokens =
        Coin::from_ac(tokens_ac).ok_or_else(|| format!("invalid token amount {tokens_ac}"))?;

    if tokens > buyback.token_cap {
        return Err(format!(
            "requested {tokens} exceeds buyback cap of {}",
            buyback.token_cap
        ));
    }

    let payout = buyback
        .payout_usd_cents(tokens)
        .ok_or("payout calculation overflow")?;

    if json {
        println!(
            "{{\n  \"offer_id\": \"{}\",\n  \"offer_name\": \"{}\",\n  \"tokens_ac\": \"{}\",\n  \"price_usd_cents\": {},\n  \"payout_usd_cents\": {},\n  \"status\": \"quote only; not an executed buyback\"\n}}",
            buyback.id, buyback.name, tokens, buyback.price_usd_cents, payout
        );
        return Ok(());
    }

    println!("ABYSS investor buyback quote");
    println!("offer: {} ({})", buyback.name, buyback.id);
    println!("tokens_offered: {tokens}");
    println!("buyback_price: {}", usd_cents_to_string(buyback.price_usd_cents));
    println!("payout: {}", usd_cents_to_string(payout));
    println!(
        "offer_cap: {} (max payout {})",
        buyback.token_cap,
        usd_cents_to_string(buyback.max_payout_usd_cents())
    );
    println!("status: quote only; not an executed buyback");

    Ok(())
}

// ── presale dex-quote (Final Sale via ABYSS DEX test orders, $5.00/AC) ──

fn quote_dex_final_sale(args: Vec<String>) -> Result<(), String> {
    let mut amount = None;
    let mut json = false;

    for arg in args {
        if arg == "--json" {
            json = true;
            continue;
        }
        let (key, value) = arg
            .split_once('=')
            .ok_or_else(|| format!("invalid argument '{arg}', expected --key=value"))?;
        match key {
            "--amount" => {
                amount = Some(
                    parse_usd_to_cents(value).map_err(|err| format!("invalid --amount: {err:?}"))?,
                )
            }
            _ => return Err(format!("unknown argument '{key}'")),
        }
    }

    let amount = amount.ok_or("missing --amount=<usd>")?;
    let dex_sale = DexFinalSale::abyss_default();
    let tokens = dex_sale
        .tokens_for_usd_cents(amount)
        .map_err(|err| format!("dex quote rejected: {err:?}"))?;

    if json {
        println!(
            "{{\n  \"sale_id\": \"{}\",\n  \"sale_name\": \"{}\",\n  \"contribution_usd_cents\": {},\n  \"price_usd_cents\": {},\n  \"token_amount_ac\": \"{}\",\n  \"venue\": \"ABYSS DEX (test orders)\",\n  \"status\": \"quote only; executed via live DEX order matching, not this CLI\"\n}}",
            dex_sale.id, dex_sale.name, amount, dex_sale.price_usd_cents, tokens
        );
        return Ok(());
    }

    println!("ABYSS final sale (DEX) quote");
    println!("sale: {} ({})", dex_sale.name, dex_sale.id);
    println!("contribution: {}", usd_cents_to_string(amount));
    println!("price: {}", usd_cents_to_string(dex_sale.price_usd_cents));
    println!("token_amount: {tokens}");
    println!("venue: ABYSS DEX (test orders)");
    println!("status: quote only; executed via live DEX order matching, not this CLI");

    Ok(())
}

// ── tokenomics (full plan: allocations + sale rounds + buyback + dex) ──

fn print_tokenomics(args: Vec<String>) -> Result<(), String> {
    let json = args.iter().any(|a| a == "--json");
    let plan = TokenomicsPlan::abyss_default();
    plan.validate()
        .map_err(|err| format!("invalid tokenomics plan: {err:?}"))?;
    let buyback = BuybackOffer::abyss_default();
    let dex_sale = DexFinalSale::abyss_default();

    if json {
        let allocations_json: Vec<String> = plan
            .allocations
            .iter()
            .map(|a| {
                format!(
                    "    {{ \"name\": \"{}\", \"basis_points\": {}, \"amount_ac\": \"{}\" }}",
                    json_escape(a.name),
                    a.basis_points,
                    a.amount()
                )
            })
            .collect();

        let rounds_json: Vec<String> = plan
            .sale_rounds
            .iter()
            .map(|r| {
                format!(
                    "    {{ \"id\": \"{}\", \"name\": \"{}\", \"token_cap_ac\": \"{}\", \"price_usd_cents\": {}, \"minimum_ticket_usd\": {}, \"raise_cap_usd_cents\": {} }}",
                    r.id, json_escape(r.name), r.token_cap, r.price_usd_cents, r.minimum_ticket_usd, r.raise_cap_usd_cents()
                )
            })
            .collect();

        let total = plan.total_sale_cap_usd_cents().unwrap_or(0);

        println!(
            "{{\n  \"symbol\": \"{}\",\n  \"max_supply_ac\": \"{}\",\n  \"team_reserve_ac\": \"{}\",\n  \"public_sale_ac\": \"{}\",\n  \"allocations\": [\n{}\n  ],\n  \"sale_rounds\": [\n{}\n  ],\n  \"buyback_offer\": {{ \"id\": \"{}\", \"name\": \"{}\", \"token_cap_ac\": \"{}\", \"price_usd_cents\": {}, \"max_payout_usd_cents\": {} }},\n  \"final_sale_dex\": {{ \"id\": \"{}\", \"name\": \"{}\", \"price_usd_cents\": {}, \"note\": \"variable supply, executed via DEX test orders\" }},\n  \"maximum_sale_raise_usd_cents\": {}\n}}",
            plan.symbol,
            plan.max_supply,
            plan.team_reserve_amount(),
            plan.allocation_amount("Public sale and liquidity formation").unwrap_or(Coin::ZERO),
            allocations_json.join(",\n"),
            rounds_json.join(",\n"),
            buyback.id, buyback.name, buyback.token_cap, buyback.price_usd_cents, buyback.max_payout_usd_cents(),
            dex_sale.id, dex_sale.name, dex_sale.price_usd_cents,
            total,
        );
        return Ok(());
    }

    println!("ABYSS tokenomics");
    println!("symbol: {}", plan.symbol);
    println!("max_supply: {}", plan.max_supply);
    println!("team_reserve: {}", plan.team_reserve_amount());
    println!(
        "public_sale: {}",
        plan.allocation_amount("Public sale and liquidity formation")
            .unwrap_or(Coin::ZERO)
    );
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
            "  - {} [{}]: cap {}, price {}, min ${}, raise cap {}",
            round.name,
            round.id,
            round.token_cap,
            usd_cents_to_string(round.price_usd_cents),
            round.minimum_ticket_usd,
            usd_cents_to_string(round.raise_cap_usd_cents())
        );
    }

    println!();
    println!("special_stages:");
    println!(
        "  - {} [{}]: cap {}, price {}, max payout {} (not a mint, returns to public pool)",
        buyback.name,
        buyback.id,
        buyback.token_cap,
        usd_cents_to_string(buyback.price_usd_cents),
        usd_cents_to_string(buyback.max_payout_usd_cents())
    );
    println!(
        "  - {} [{}]: variable supply, price {} (executed via ABYSS DEX test orders)",
        dex_sale.name,
        dex_sale.id,
        usd_cents_to_string(dex_sale.price_usd_cents)
    );

    if let Some(total) = plan.total_sale_cap_usd_cents() {
        println!();
        println!(
            "maximum_sale_raise: {} (standard rounds only; excludes buyback and variable dex sale)",
            usd_cents_to_string(total)
        );
    }

    Ok(())
}

// ── vesting (Team Reserve unlock schedule) ──

fn print_vesting(args: Vec<String>) -> Result<(), String> {
    let json = args.iter().any(|a| a == "--json");
    let vesting = TeamVesting::abyss_default();

    if json {
        let years: Vec<String> = (1..=5)
            .map(|year| {
                format!(
                    "    {{ \"year\": {}, \"unlocked_this_year_ac\": \"{}\", \"cumulative_unlocked_ac\": \"{}\" }}",
                    year,
                    vesting.unlocked_in_year(year),
                    vesting.total_unlocked(year * 12)
                )
            })
            .collect();

        println!(
            "{{\n  \"tranche_a_total_ac\": \"{}\",\n  \"tranche_a_months\": {},\n  \"tranche_b_total_ac\": \"{}\",\n  \"tranche_b_months\": {},\n  \"total_ac\": \"{}\",\n  \"by_year\": [\n{}\n  ]\n}}",
            vesting.tranche_a_total,
            vesting.tranche_a_months,
            vesting.tranche_b_total,
            vesting.tranche_b_months,
            vesting.total(),
            years.join(",\n"),
        );
        return Ok(());
    }

    println!("ABYSS team reserve vesting");
    println!(
        "tranche_a: {} over {} months (linear, no cliff)",
        vesting.tranche_a_total, vesting.tranche_a_months
    );
    println!(
        "tranche_b: {} over {} months (linear, no cliff, capped at {} / 12 months)",
        vesting.tranche_b_total, vesting.tranche_b_months, vesting.tranche_b_annual_cap
    );
    println!("total: {}", vesting.total());
    println!();
    println!("unlock_by_year:");
    for year in 1..=5u16 {
        println!(
            "  year {}: +{} this year, {} cumulative",
            year,
            vesting.unlocked_in_year(year),
            vesting.total_unlocked(year * 12)
        );
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
    println!("  abyss-node presale quote --amount=<usd> [--round=<id>] [--kyc-approved] [--professional] [--json]");
    println!("  abyss-node presale buyback --tokens=<ac amount> [--json]");
    println!("  abyss-node presale dex-quote --amount=<usd> [--json]");
    println!("  abyss-node tokenomics [--json]   print the current AC tokenomics plan");
    println!("  abyss-node vesting [--json]   print the team reserve vesting schedule");
    println!("  abyss-node help     show this help");
    println!();
    println!("Sale round ids: sale-to-investors, pre-sale, public-stage-1, public-stage-2, public-stage-3");
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

/// Minimal JSON string escaping for the hand-rolled JSON output above.
/// Sufficient for our known field values (names, ids); not a general
/// JSON encoder.
fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
