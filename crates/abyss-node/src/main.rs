use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use abyss_core::{hashing, Chain, ChainConfig, Coin, GenesisConfig, Mempool, Transaction};
use abyss_social::{
    AgentActivityWindow, AgentSocialAction, AgentSocialPolicy, DevFeed, Visibility,
};
use abyss_tokenomics::{
    parse_usd_to_cents, usd_cents_to_string, ContributionReceipt, DexFinalSale, InvestorProfile,
    InvestorSecondaryWindow, KycStatus, SecondaryListing, TeamVesting, TokenomicsPlan,
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
            Some("quote")            => quote_presale(args.collect()),
            Some("secondary-window") => quote_secondary_window(args.collect()),
            Some("dex-quote")        => quote_dex_final_sale(args.collect()),
            Some(command) => Err(format!("unknown presale command '{command}'")),
            None => Err("missing presale command".to_string()),
        },
        Some("tokenomics") => print_tokenomics(args.collect()),
        Some("vesting")    => print_vesting(args.collect()),
        Some("social") => match args.next().as_deref() {
            Some("demo") => run_social_demo(),
            Some("post") => social_post(args.collect()),
            Some(command) => Err(format!("unknown social command '{command}'")),
            None => Err("missing social command. Try: social demo | social post --author=<addr> --body=<text>".to_string()),
        },
        Some("help") | Some("--help") | Some("-h") | None => { print_help(); Ok(()) }
        Some(command) => Err(format!("unknown command '{command}'")),
    }
}

// ── presale secondary-window ──────────────────────────────────────────────────
//
// Investor Secondary Window: Stage I investors may list their tokens for P2P
// sale to new participants at $3.00/AC. ABYSS facilitates but does NOT
// purchase tokens itself — no buyback obligation.
//
// Usage:
//   abyss-node presale secondary-window --tokens=<ac> [--json]
//   abyss-node presale secondary-window --info [--json]

fn quote_secondary_window(args: Vec<String>) -> Result<(), String> {
    let mut tokens_ac: Option<u64> = None;
    let mut info_only = false;
    let mut json = false;

    for arg in &args {
        if arg == "--json" {
            json = true;
            continue;
        }
        if arg == "--info" {
            info_only = true;
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

    let window = InvestorSecondaryWindow::abyss_default();

    // -- info mode: just print window parameters --
    if info_only || tokens_ac.is_none() {
        if json {
            println!(
                "{{\n  \"id\": \"{}\",\n  \"name\": \"{}\",\n  \"price_usd_cents\": {},\n  \"registration_days\": {},\n  \"min_listing_bps\": {},\n  \"min_listing_ac\": {},\n  \"eligible_sellers\": \"Stage I investors only\",\n  \"eligible_buyers\": \"any participant\",\n  \"abyss_obligation\": \"none — facilitated P2P market, not a buyback\"\n}}",
                window.id, window.name, window.price_usd_cents,
                window.registration_days, window.min_listing_bps, window.min_listing_ac()
            );
        } else {
            println!("ABYSS Investor Secondary Window");
            println!("id                 : {}", window.id);
            println!(
                "price              : {}",
                usd_cents_to_string(window.price_usd_cents)
            );
            println!(
                "registration phase : {} days (intent listing)",
                window.registration_days
            );
            println!("sales phase        : until all listed tokens are sold");
            println!(
                "min listing        : {}% of Stage I allocation = {} AC minimum",
                window.min_listing_bps / 100,
                window.min_listing_ac()
            );
            println!("eligible sellers   : Stage I investors only");
            println!("eligible buyers    : any participant");
            println!("abyss obligation   : NONE — facilitated P2P market, not a buyback");
            println!();
            println!("note: submit --tokens=<amount> to calculate a specific listing quote");
        }
        return Ok(());
    }

    // -- listing quote mode --
    let tokens_ac = tokens_ac.unwrap();
    let listing = SecondaryListing::new("demo-investor", tokens_ac, true)
        .map_err(|e| format!("invalid token amount: {e:?}"))?;
    window
        .validate_listing(&listing)
        .map_err(|e| format!("listing rejected: {e:?}"))?;

    let payout = window
        .seller_payout_usd_cents(listing.tokens_to_list)
        .ok_or("payout calculation overflow")?;

    if json {
        println!(
            "{{\n  \"window_id\": \"{}\",\n  \"tokens_listed_ac\": \"{}\",\n  \"price_usd_cents\": {},\n  \"seller_payout_usd_cents\": {},\n  \"registration_days\": {},\n  \"min_listing_ac\": {},\n  \"eligible_sellers\": \"Stage I investors only\",\n  \"abyss_obligation\": \"none\",\n  \"status\": \"quote only; submit intent via website during registration phase\"\n}}",
            window.id, listing.tokens_to_list, window.price_usd_cents,
            payout, window.registration_days, window.min_listing_ac()
        );
        return Ok(());
    }

    println!("ABYSS Investor Secondary Window — listing quote");
    println!("window             : {} ({})", window.name, window.id);
    println!("tokens_to_list     : {}", listing.tokens_to_list);
    println!(
        "price              : {}",
        usd_cents_to_string(window.price_usd_cents)
    );
    println!("seller_payout      : {}", usd_cents_to_string(payout));
    println!(
        "registration_phase : {} days to submit intent",
        window.registration_days
    );
    println!("sales_phase        : open until all listed tokens are sold");
    println!(
        "min_listing        : {} AC (50% of Stage I slot)",
        window.min_listing_ac()
    );
    println!("abyss_obligation   : NONE — P2P facilitated market");
    println!(
        "status             : quote only; submit intent via website during registration phase"
    );

    Ok(())
}

// ── presale quote ─────────────────────────────────────────────────────────────

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
        println!(
            "{{\n  \"investor_id\": \"{}\",\n  \"jurisdiction\": \"{}\",\n  \"round_id\": \"{}\",\n  \"round_name\": \"{}\",\n  \"contribution_usd_cents\": {},\n  \"price_usd_cents\": {},\n  \"token_amount_ac\": \"{}\",\n  \"lockup_months\": {},\n  \"status\": \"quote only; do not accept funds without legal review\"\n}}",
            json_escape(&receipt.investor_id), json_escape(&receipt.jurisdiction),
            receipt.round_id, json_escape(receipt.round_name),
            receipt.contribution_usd_cents, round.price_usd_cents,
            receipt.token_amount, receipt.vesting.cliff_months,
        );
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
                            .map_err(|e| format!("invalid --amount: {e:?}"))?,
                    )
                }
                "--max" => {
                    max = parse_usd_to_cents(value).map_err(|e| format!("invalid --max: {e:?}"))?
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

// ── presale dex-quote ─────────────────────────────────────────────────────────

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
                    parse_usd_to_cents(value).map_err(|e| format!("invalid --amount: {e:?}"))?,
                )
            }
            _ => return Err(format!("unknown argument '{key}'")),
        }
    }
    let amount = amount.ok_or("missing --amount=<usd>")?;
    let dex_sale = DexFinalSale::abyss_default();
    let tokens = dex_sale
        .tokens_for_usd_cents(amount)
        .map_err(|e| format!("dex quote rejected: {e:?}"))?;
    if json {
        println!("{{\n  \"sale_id\": \"{}\",\n  \"sale_name\": \"{}\",\n  \"contribution_usd_cents\": {},\n  \"price_usd_cents\": {},\n  \"token_amount_ac\": \"{}\",\n  \"venue\": \"ABYSS DEX (test orders)\",\n  \"status\": \"quote only; executed via live DEX order matching, not this CLI\"\n}}",
            dex_sale.id, dex_sale.name, amount, dex_sale.price_usd_cents, tokens);
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

// ── tokenomics ────────────────────────────────────────────────────────────────

fn print_tokenomics(args: Vec<String>) -> Result<(), String> {
    let json = args.iter().any(|a| a == "--json");
    let plan = TokenomicsPlan::abyss_default();
    plan.validate()
        .map_err(|err| format!("invalid tokenomics plan: {err:?}"))?;
    let window = InvestorSecondaryWindow::abyss_default();
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
        let rounds_json: Vec<String> = plan.sale_rounds.iter().map(|r| {
            format!("    {{ \"id\": \"{}\", \"name\": \"{}\", \"token_cap_ac\": \"{}\", \"price_usd_cents\": {}, \"minimum_ticket_usd\": {}, \"raise_cap_usd_cents\": {} }}",
                r.id, json_escape(r.name), r.token_cap, r.price_usd_cents,
                r.minimum_ticket_usd, r.raise_cap_usd_cents())
        }).collect();
        let total = plan.total_sale_cap_usd_cents().unwrap_or(0);
        println!(
            "{{\n  \"symbol\": \"{}\",\n  \"max_supply_ac\": \"{}\",\n  \"team_reserve_ac\": \"{}\",\n  \"public_sale_ac\": \"{}\",\n  \"allocations\": [\n{}\n  ],\n  \"sale_rounds\": [\n{}\n  ],\n  \"investor_secondary_window\": {{ \"id\": \"{}\", \"name\": \"{}\", \"price_usd_cents\": {}, \"registration_days\": {}, \"min_listing_ac\": {}, \"abyss_obligation\": \"none\" }},\n  \"final_sale_dex\": {{ \"id\": \"{}\", \"name\": \"{}\", \"price_usd_cents\": {}, \"note\": \"variable supply, executed via DEX test orders\" }},\n  \"maximum_sale_raise_usd_cents\": {}\n}}",
            plan.symbol, plan.max_supply, plan.team_reserve_amount(),
            plan.allocation_amount("Public sale and liquidity formation").unwrap_or(Coin::ZERO),
            allocations_json.join(",\n"), rounds_json.join(",\n"),
            window.id, window.name, window.price_usd_cents,
            window.registration_days, window.min_listing_ac(),
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
    for a in &plan.allocations {
        println!("  - {}: {} bps / {}", a.name, a.basis_points, a.amount());
    }
    println!();
    println!("sale_rounds:");
    for r in &plan.sale_rounds {
        println!(
            "  - {} [{}]: cap {}, price {}, min ${}, raise cap {}",
            r.name,
            r.id,
            r.token_cap,
            usd_cents_to_string(r.price_usd_cents),
            r.minimum_ticket_usd,
            usd_cents_to_string(r.raise_cap_usd_cents())
        );
    }
    println!();
    println!("special_stages:");
    println!(
        "  - {} [{}]: price {}, registration {} days, min listing {} AC",
        window.name,
        window.id,
        usd_cents_to_string(window.price_usd_cents),
        window.registration_days,
        window.min_listing_ac()
    );
    println!(
        "    sellers: Stage I investors only | buyers: any participant | ABYSS obligation: NONE"
    );
    println!(
        "  - {} [{}]: variable supply, price {} (executed via ABYSS DEX test orders)",
        dex_sale.name,
        dex_sale.id,
        usd_cents_to_string(dex_sale.price_usd_cents)
    );
    if let Some(total) = plan.total_sale_cap_usd_cents() {
        println!();
        println!("maximum_sale_raise: {} (standard rounds only; excludes secondary window and variable dex sale)",
            usd_cents_to_string(total));
    }
    Ok(())
}

// ── vesting ───────────────────────────────────────────────────────────────────

fn print_vesting(args: Vec<String>) -> Result<(), String> {
    let json = args.iter().any(|a| a == "--json");
    let vesting = TeamVesting::abyss_default();
    if json {
        let years: Vec<String> = (1..=5).map(|year| {
            format!("    {{ \"year\": {}, \"unlocked_this_year_ac\": \"{}\", \"cumulative_unlocked_ac\": \"{}\" }}",
                year, vesting.unlocked_in_year(year), vesting.total_unlocked(year * 12))
        }).collect();
        println!("{{\n  \"tranche_a_total_ac\": \"{}\",\n  \"tranche_a_months\": {},\n  \"tranche_b_total_ac\": \"{}\",\n  \"tranche_b_months\": {},\n  \"total_ac\": \"{}\",\n  \"by_year\": [\n{}\n  ]\n}}",
            vesting.tranche_a_total, vesting.tranche_a_months,
            vesting.tranche_b_total, vesting.tranche_b_months,
            vesting.total(), years.join(",\n"));
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

// ── social ────────────────────────────────────────────────────────────────────

fn run_social_demo() -> Result<(), String> {
    let now = now_ms();
    let mut feed = DevFeed::new();
    let alice = "abyss1dev_alice";
    let bob = "abyss1dev_bob";

    let post1 = feed
        .publish(
            alice,
            "Hello ABYSS. The future of private social is here.",
            Visibility::Attributed,
            now,
        )
        .map_err(|e| format!("post failed: {e:?}"))?;
    let post2 = feed
        .publish(
            bob,
            "This message is mine. The network cannot prove it.",
            Visibility::Shielded,
            now + 1_000,
        )
        .map_err(|e| format!("post failed: {e:?}"))?;
    let post3 = feed
        .reply(
            alice,
            "Building in public. Transacting in private.",
            Visibility::Attributed,
            now + 2_000,
            post1,
        )
        .map_err(|e| format!("reply failed: {e:?}"))?;

    let agent_policy = AgentSocialPolicy::curator_default();
    let mut agent_window = AgentActivityWindow::new(now);
    agent_window
        .record_post(&agent_policy, AgentSocialAction::Repost, now + 3_000)
        .map_err(|e| format!("agent action rejected: {e:?}"))?;
    for i in 0..9 {
        let _ = agent_window.record_post(&agent_policy, AgentSocialAction::Repost, now + 4_000 + i);
    }
    let rate_limit_result =
        agent_window.record_post(&agent_policy, AgentSocialAction::Repost, now + 5_000);

    println!("ABYSS social layer -- devnet demonstration");
    println!("-----------------------------------------");
    println!();
    println!("feed: {} posts", feed.len());
    println!();
    println!("post #{}: [attributed]", post1.0);
    if let Some(p) = feed.get(post1) {
        println!("  author : {}", p.author);
        println!("  body   : {}", p.body);
    }
    println!();
    println!("post #{}: [shielded]", post2.0);
    if let Some(p) = feed.get(post2) {
        println!("  author : <hidden -- only the author and view-key holders can see this>");
        println!("  body   : {}", p.body);
        println!(
            "  author_visible_to(alice): {}",
            p.author_visible_to(alice, &Default::default())
        );
        println!(
            "  author_visible_to(bob):   {}",
            p.author_visible_to(bob, &Default::default())
        );
    }
    println!();
    println!("post #{}: [reply to #{}]", post3.0, post1.0);
    if let Some(p) = feed.get(post3) {
        println!("  author : {}", p.author);
        println!("  body   : {}", p.body);
    }
    println!(
        "  replies to #{}: {} found",
        post1.0,
        feed.replies_to(post1).len()
    );
    println!();
    println!("ai agent (curator policy):");
    println!("  can_post   : {}", agent_policy.can_post);
    println!("  can_repost : {}", agent_policy.can_repost);
    println!(
        "  rate_limit : {} reposts / {} seconds",
        agent_policy.max_posts_per_window, agent_policy.rate_window_seconds
    );
    match rate_limit_result {
        Err(e) => println!("  rate limit enforced: {:?}", e),
        Ok(()) => println!("  (rate limit not yet hit)"),
    }
    println!();
    println!("note: this is an in-memory devnet feed.");
    println!("      production storage is content-addressed and replicated (Phase 6).");
    Ok(())
}

fn social_post(args: Vec<String>) -> Result<(), String> {
    let mut author = String::new();
    let mut body = String::new();
    let mut shielded = false;
    for arg in &args {
        if arg == "--shielded" {
            shielded = true;
            continue;
        }
        let (key, value) = arg
            .split_once('=')
            .ok_or_else(|| format!("invalid argument '{arg}', expected --key=value"))?;
        match key {
            "--author" => author = value.to_string(),
            "--body" => body = value.to_string(),
            _ => return Err(format!("unknown argument '{key}'")),
        }
    }
    if author.is_empty() {
        return Err("missing --author=<address>".to_string());
    }
    if body.is_empty() {
        return Err("missing --body=<text>".to_string());
    }
    let visibility = if shielded {
        Visibility::Shielded
    } else {
        Visibility::Attributed
    };
    let mut feed = DevFeed::new();
    let id = feed
        .publish(&author, &body, visibility, now_ms())
        .map_err(|e| format!("post rejected: {e:?}"))?;
    println!("ABYSS social post");
    println!("post_id    : {}", id.0);
    println!("author     : {author}");
    println!(
        "visibility : {}",
        if shielded { "shielded" } else { "attributed" }
    );
    println!("body       : {body}");
    println!("status     : published to devnet feed (in-memory)");
    Ok(())
}

// ── account ───────────────────────────────────────────────────────────────────

fn create_account(label: &str) {
    let account = WalletAccount::generate(label);
    println!("ABYSS dev account created");
    println!("label: {}", account.label());
    println!("address: {}", account.address());
    println!("public_key: {}", account.public_key());
    println!(
        "agent_permissions: {:?}",
        account.agent_policy().permissions()
    );
    println!("warning: dev account only; production key storage is not implemented yet");
}

// ── devnet ────────────────────────────────────────────────────────────────────

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
        .map_err(|err| format!("failed to insert tx1: {err:?}"))?;
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

    let tx2 = alice_agent
        .create_agent_payment(
            bob.clone(),
            Coin::from_ac(125).ok_or("invalid amount")?,
            Coin::from_micro_ac(2_500).ok_or("invalid fee")?,
            chain.next_nonce(&alice),
        )
        .map_err(|err| format!("agent policy rejected payment: {err:?}"))?;
    mempool
        .insert(tx2)
        .map_err(|err| format!("failed to insert tx2: {err:?}"))?;
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

// ── help ──────────────────────────────────────────────────────────────────────

fn print_help() {
    println!("ABYSS Node");
    println!();
    println!("Usage:");
    println!("  abyss-node account new [label]");
    println!("  abyss-node devnet");
    println!("  abyss-node presale quote --amount=<usd> [--round=<id>] [--kyc-approved] [--professional] [--json]");
    println!("  abyss-node presale secondary-window [--info] [--tokens=<ac>] [--json]");
    println!("  abyss-node presale dex-quote --amount=<usd> [--json]");
    println!("  abyss-node tokenomics [--json]");
    println!("  abyss-node vesting [--json]");
    println!("  abyss-node social demo");
    println!("  abyss-node social post --author=<address> --body=<text> [--shielded]");
    println!("  abyss-node help");
    println!();
    println!("Sale round ids: sale-to-investors, pre-sale, public-stage-1, public-stage-2, public-stage-3");
    println!(
        "Secondary window: Stage I investors only, min 250,000 AC, fixed $3.00, P2P facilitated"
    );
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default()
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
