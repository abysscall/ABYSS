//! Tokenomics and sale planning primitives for ABYSS.
//!
//! This module is a deterministic planning model, not a legal wrapper and not
//! an on-chain sale contract. Any real token sale must be reviewed by qualified
//! counsel in the target jurisdictions before accepting funds.

use abyss_core::{Coin, COIN};

pub const BASIS_POINTS: u16 = 10_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenomicsPlan {
    pub symbol: &'static str,
    pub max_supply: Coin,
    pub allocations: Vec<Allocation>,
    pub sale_rounds: Vec<SaleRound>,
}

impl TokenomicsPlan {
    /// ABYSS default plan.
    ///
    /// Max supply is fixed at 55,000,000 AC, split into two top-level pools:
    ///   - Team Reserve  : 30,000,000 AC (54.5%) — vested, see `TeamVesting`
    ///   - Public Sale   : 25,000,000 AC (45.5%) — distributed across sale_rounds
    ///
    /// The `allocations` vector keeps the original category breakdown
    /// (validator rewards, ecosystem grants, treasury, etc.) but the basis
    /// points have been recalibrated so the totals reconcile exactly with
    /// the public 25M / 30M split published on the website and in
    /// PRESALE_STRATEGY.md.
    pub fn abyss_default() -> Self {
        Self {
            symbol: "AC",
            max_supply: Coin::MAX,
            allocations: vec![
                // ── Team Reserve side — sums to exactly 30,000,000 AC ──
                Allocation::new("Validator rewards and network security", 1_500, 8_250_000),
                Allocation::new("Ecosystem grants, apps, audits, bug bounties", 1_000, 5_500_000),
                Allocation::new("Foundation treasury with long vesting", 1_000, 5_500_000),
                Allocation::new("Core contributors with long vesting", 1_000, 5_500_000),
                Allocation::new("DEX liquidity reserve", 1_000, 5_250_000),
                // ── Public Sale side — sums to exactly 25,000,000 AC ──
                Allocation::new("Public sale and liquidity formation", 4_500, 25_000_000),
            ],
            sale_rounds: vec![
                // 1. Sale to Investors — institutional round, min ticket $500,000
                SaleRound::new("Sale to Investors", 2_000_000, 100, 500_000, 0),
                // 2. Pre-Sale — early community allocation
                SaleRound::new("Pre-Sale", 3_000_000, 200, 0, 0),
                // 3. Sale Stage 1 — public open round
                SaleRound::new("Sale Stage 1", 5_000_000, 300, 0, 0),
                // 4. Sale Stage 2 — growth phase allocation
                SaleRound::new("Sale Stage 2", 5_000_000, 400, 0, 0),
                // 5. Sale Stage 3 — pre-final public round
                SaleRound::new("Sale Stage 3", 10_000_000, 500, 0, 0),
                // NOTE: "Investor Buyback" (2,000,000 AC @ $3.00) and
                // "Final Sale · DEX" (variable supply @ $5.00) are NOT
                // ordinary sale rounds — they are handled by
                // `BuybackOffer` and `DexFinalSale` respectively, because
                // their token accounting differs from a fixed-cap sale:
                //   - Buyback returns tokens to the public-sale pool
                //     instead of minting new ones.
                //   - The DEX final sale has no fixed token_cap; it is
                //     executed as live test orders on the ABYSS DEX.
            ],
        }
    }

    pub fn validate(&self) -> Result<(), TokenomicsError> {
        let allocation_bps = self
            .allocations
            .iter()
            .try_fold(0_u16, |acc, item| acc.checked_add(item.basis_points))
            .ok_or(TokenomicsError::BasisPointOverflow)?;

        if allocation_bps != BASIS_POINTS {
            return Err(TokenomicsError::AllocationBasisPointsMustEqual10000 {
                actual: allocation_bps,
            });
        }

        let public_sale = self
            .allocation_amount("Public sale and liquidity formation")
            .ok_or(TokenomicsError::MissingPublicSaleAllocation)?;

        let round_tokens = self
            .sale_rounds
            .iter()
            .try_fold(Coin::ZERO, |acc, round| acc.checked_add(round.token_cap))
            .ok_or(TokenomicsError::SupplyOverflow)?;

        if round_tokens > public_sale {
            return Err(TokenomicsError::SaleRoundsExceedPublicAllocation {
                rounds: round_tokens,
                allocation: public_sale,
            });
        }

        Ok(())
    }

    pub fn allocation_amount(&self, name: &str) -> Option<Coin> {
        self.allocations
            .iter()
            .find(|allocation| allocation.name == name)
            .map(|allocation| allocation.amount())
    }

    /// Sum of raise caps for fixed-supply Standard rounds only.
    /// Excludes the Buyback offer (not a raise) and the DEX final sale
    /// (variable supply, no fixed cap).
    pub fn total_sale_cap_usd_cents(&self) -> Option<u64> {
        self.sale_rounds.iter().try_fold(0_u64, |acc, round| {
            acc.checked_add(round.raise_cap_usd_cents())
        })
    }

    pub fn sale_round(&self, round_id: &str) -> Option<&SaleRound> {
        self.sale_rounds
            .iter()
            .find(|round| round.id == round_id || round.name.eq_ignore_ascii_case(round_id))
    }

    /// Team Reserve total: everything in `allocations` except the public
    /// sale bucket. Should always equal 30,000,000 AC for the ABYSS default
    /// plan — enforced by a unit test below.
    pub fn team_reserve_amount(&self) -> Coin {
        self.allocations
            .iter()
            .filter(|a| a.name != "Public sale and liquidity formation")
            .fold(Coin::ZERO, |acc, a| {
                acc.checked_add(a.amount())
                    .expect("team reserve fits within max supply")
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvestorProfile {
    pub investor_id: String,
    pub jurisdiction: String,
    pub kyc_status: KycStatus,
    pub accredited_or_professional: bool,
    pub max_contribution_usd_cents: u64,
}

impl InvestorProfile {
    pub fn eligible_for(&self, round: &SaleRound) -> Result<(), TokenomicsError> {
        if self.kyc_status != KycStatus::Approved {
            return Err(TokenomicsError::InvestorNotApproved);
        }

        if self.jurisdiction.trim().is_empty() {
            return Err(TokenomicsError::MissingJurisdiction);
        }

        if round.minimum_ticket_usd >= 100_000 && !self.accredited_or_professional {
            return Err(TokenomicsError::ProfessionalInvestorRequired);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KycStatus {
    NotStarted,
    Pending,
    Approved,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Allocation {
    pub name: &'static str,
    pub basis_points: u16,
    /// Exact AC amount for this allocation. Using an explicit amount
    /// (rather than deriving it purely from basis_points / 10_000) avoids
    /// integer-rounding drift when MAX_SUPPLY does not divide evenly by
    /// 10_000 — which is the case for ABYSS's 55,000,000 AC cap.
    /// `basis_points` is kept for display/reporting purposes and must
    /// still sum to 10_000 across all allocations (checked in `validate`).
    exact_amount_ac: u64,
}

impl Allocation {
    pub const fn new(name: &'static str, basis_points: u16, exact_amount_ac: u64) -> Self {
        Self {
            name,
            basis_points,
            exact_amount_ac,
        }
    }

    pub fn amount(&self) -> Coin {
        Coin::from_ac(self.exact_amount_ac).expect("allocation is derived from capped max supply")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContributionIntent {
    pub investor_id: String,
    pub round_id: &'static str,
    pub contribution_usd_cents: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContributionReceipt {
    pub investor_id: String,
    pub jurisdiction: String,
    pub round_id: &'static str,
    pub round_name: &'static str,
    pub contribution_usd_cents: u64,
    pub token_amount: Coin,
    pub vesting: VestingSchedule,
}

impl ContributionReceipt {
    pub fn create(
        investor: &InvestorProfile,
        round: &SaleRound,
        contribution_usd_cents: u64,
    ) -> Result<Self, TokenomicsError> {
        investor.eligible_for(round)?;

        if contribution_usd_cents > investor.max_contribution_usd_cents {
            return Err(TokenomicsError::ContributionExceedsInvestorLimit);
        }

        Ok(Self {
            investor_id: investor.investor_id.clone(),
            jurisdiction: investor.jurisdiction.clone(),
            round_id: round.id,
            round_name: round.name,
            contribution_usd_cents,
            token_amount: round.tokens_for_usd_cents(contribution_usd_cents)?,
            vesting: VestingSchedule::locked(round.lockup_months),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VestingSchedule {
    pub cliff_months: u16,
    pub duration_months: u16,
}

impl VestingSchedule {
    pub const fn locked(months: u16) -> Self {
        Self {
            cliff_months: months,
            duration_months: months,
        }
    }

    pub const fn linear(cliff_months: u16, duration_months: u16) -> Self {
        Self {
            cliff_months,
            duration_months,
        }
    }

    pub fn unlocked_amount(&self, total: Coin, elapsed_months: u16) -> Coin {
        if elapsed_months < self.cliff_months {
            return Coin::ZERO;
        }

        if self.duration_months == 0 || elapsed_months >= self.duration_months {
            return total;
        }

        let unlocked = total.micro_ac() * elapsed_months as u64 / self.duration_months as u64;
        Coin::from_micro_ac(unlocked).expect("vesting cannot exceed total")
    }
}

/// Team Reserve vesting — two tranches, both linear with no cliff,
/// matching the schedule published on the ABYSS website.
///
///   Tranche A: 10,000,000 AC released linearly over 12 months
///              (~833,333 AC/month, no cliff).
///   Tranche B: 20,000,000 AC released linearly over 48 months,
///              capped at 5,000,000 AC unlocked per 12-month period
///              (no cliff).
///
/// Tranche A + Tranche B = 30,000,000 AC = `TokenomicsPlan::team_reserve_amount()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TeamVesting {
    pub tranche_a_total: Coin,
    pub tranche_a_months: u16,
    pub tranche_b_total: Coin,
    pub tranche_b_months: u16,
    pub tranche_b_annual_cap: Coin,
}

impl TeamVesting {
    pub fn abyss_default() -> Self {
        Self {
            tranche_a_total: Coin::from_ac(10_000_000).expect("fits max supply"),
            tranche_a_months: 12,
            tranche_b_total: Coin::from_ac(20_000_000).expect("fits max supply"),
            tranche_b_months: 48,
            tranche_b_annual_cap: Coin::from_ac(5_000_000).expect("fits max supply"),
        }
    }

    pub fn total(&self) -> Coin {
        self.tranche_a_total
            .checked_add(self.tranche_b_total)
            .expect("tranche A + B fits max supply")
    }

    /// AC unlocked from Tranche A after `elapsed_months`. Linear, no cliff.
    pub fn tranche_a_unlocked(&self, elapsed_months: u16) -> Coin {
        VestingSchedule::linear(0, self.tranche_a_months)
            .unlocked_amount(self.tranche_a_total, elapsed_months)
    }

    /// AC unlocked from Tranche B after `elapsed_months`. Linear, no cliff,
    /// naturally capped at 5,000,000 AC/year because the schedule spans
    /// 48 months for 20,000,000 AC (exactly 5,000,000 AC every 12 months).
    pub fn tranche_b_unlocked(&self, elapsed_months: u16) -> Coin {
        VestingSchedule::linear(0, self.tranche_b_months)
            .unlocked_amount(self.tranche_b_total, elapsed_months)
    }

    /// Combined AC unlocked across both tranches after `elapsed_months`.
    pub fn total_unlocked(&self, elapsed_months: u16) -> Coin {
        self.tranche_a_unlocked(elapsed_months)
            .checked_add(self.tranche_b_unlocked(elapsed_months))
            .expect("combined unlock cannot exceed total")
    }

    /// AC unlocked in year `year` (1-indexed) only, not cumulative.
    /// Year 1 includes the full Tranche A (10M) plus year-1 Tranche B (5M) = 15M.
    /// Years 2–5 are Tranche B only at 5M/year.
    pub fn unlocked_in_year(&self, year: u16) -> Coin {
        if year == 0 {
            return Coin::ZERO;
        }
        let months_end = year.saturating_mul(12).min(self.tranche_b_months.max(self.tranche_a_months));
        let months_start = months_end.saturating_sub(12);
        let end_total = self.total_unlocked(months_end);
        let start_total = self.total_unlocked(months_start);
        end_total
            .checked_sub(start_total)
            .expect("later unlock is never smaller than earlier unlock")
    }
}

/// Stage I "Sale to Investors" buyback offer.
///
/// After Sale Stage 1 closes, original Sale-to-Investors participants may
/// exit by selling back up to 2,000,000 AC at $3.00/AC (3x their $1.00
/// entry price). Bought-back tokens return to the public sale pool and
/// become available starting with Sale Stage 2 — they are NOT newly minted
/// and do NOT increase max_supply.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuybackOffer {
    pub id: &'static str,
    pub name: &'static str,
    pub token_cap: Coin,
    pub price_usd_cents: u64,
}

impl BuybackOffer {
    pub fn abyss_default() -> Self {
        Self {
            id: "investor-buyback",
            name: "Investor Buyback",
            token_cap: Coin::from_ac(2_000_000).expect("fits max supply"),
            price_usd_cents: 300,
        }
    }

    pub fn payout_usd_cents(&self, tokens: Coin) -> Option<u64> {
        let ac_units = tokens.micro_ac() / COIN;
        ac_units.checked_mul(self.price_usd_cents)
    }

    pub fn max_payout_usd_cents(&self) -> u64 {
        self.payout_usd_cents(self.token_cap)
            .expect("token_cap payout fits u64")
    }
}

/// Final Sale — executed via test orders on the ABYSS DEX.
///
/// Unlike the fixed-cap Standard rounds, this stage has variable supply:
/// it is open to anyone who still wants to participate, at a fixed price
/// of $5.00/AC, filled through live order matching on the ABYSS DEX rather
/// than a CLI/contract-enforced cap. This struct only encodes the fixed
/// price; actual fills are tracked by the DEX, not by this planning model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DexFinalSale {
    pub id: &'static str,
    pub name: &'static str,
    pub price_usd_cents: u64,
}

impl DexFinalSale {
    pub fn abyss_default() -> Self {
        Self {
            id: "final-sale-dex",
            name: "Final Sale (ABYSS DEX)",
            price_usd_cents: 500,
        }
    }

    pub fn tokens_for_usd_cents(&self, contribution_usd_cents: u64) -> Result<Coin, TokenomicsError> {
        let micro_ac = contribution_usd_cents
            .checked_mul(COIN)
            .ok_or(TokenomicsError::SupplyOverflow)?
            .checked_div(self.price_usd_cents)
            .ok_or(TokenomicsError::InvalidRoundPrice)?;

        Coin::from_micro_ac(micro_ac).ok_or(TokenomicsError::SupplyOverflow)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaleRound {
    pub id: &'static str,
    pub name: &'static str,
    pub token_cap: Coin,
    pub price_usd_cents: u64,
    pub minimum_ticket_usd: u64,
    pub lockup_months: u16,
}

impl SaleRound {
    pub fn new(
        name: &'static str,
        token_cap_ac: u64,
        price_usd_cents: u64,
        minimum_ticket_usd: u64,
        lockup_months: u16,
    ) -> Self {
        let id = match name {
            "Sale to Investors" => "sale-to-investors",
            "Pre-Sale" => "pre-sale",
            "Sale Stage 1" => "public-stage-1",
            "Sale Stage 2" => "public-stage-2",
            "Sale Stage 3" => "public-stage-3",
            _ => "custom",
        };

        Self {
            id,
            name,
            token_cap: Coin::from_ac(token_cap_ac).expect("round cap must fit max supply"),
            price_usd_cents,
            minimum_ticket_usd,
            lockup_months,
        }
    }

    pub fn raise_cap_usd_cents(&self) -> u64 {
        (self.token_cap.micro_ac() / COIN) * self.price_usd_cents
    }

    pub fn tokens_for_usd_cents(&self, contribution_usd_cents: u64) -> Result<Coin, TokenomicsError> {
        if contribution_usd_cents < self.minimum_ticket_usd * 100 {
            return Err(TokenomicsError::ContributionBelowMinimum);
        }

        let micro_ac = contribution_usd_cents
            .checked_mul(COIN)
            .ok_or(TokenomicsError::SupplyOverflow)?
            .checked_div(self.price_usd_cents)
            .ok_or(TokenomicsError::InvalidRoundPrice)?;

        let tokens = Coin::from_micro_ac(micro_ac).ok_or(TokenomicsError::SupplyOverflow)?;
        if tokens > self.token_cap {
            return Err(TokenomicsError::ContributionExceedsRoundCap);
        }

        Ok(tokens)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenomicsError {
    AllocationBasisPointsMustEqual10000 { actual: u16 },
    BasisPointOverflow,
    ContributionBelowMinimum,
    ContributionExceedsInvestorLimit,
    ContributionExceedsRoundCap,
    InvalidRoundPrice,
    InvalidUsdAmount,
    InvestorNotApproved,
    MissingJurisdiction,
    MissingPublicSaleAllocation,
    ProfessionalInvestorRequired,
    SaleRoundsExceedPublicAllocation { rounds: Coin, allocation: Coin },
    SupplyOverflow,
}

pub fn usd_cents_to_string(cents: u64) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

pub fn parse_usd_to_cents(value: &str) -> Result<u64, TokenomicsError> {
    let value = value.trim().trim_start_matches('$').replace(',', "");
    if value.is_empty() {
        return Err(TokenomicsError::InvalidUsdAmount);
    }

    let mut parts = value.split('.');
    let dollars = parts
        .next()
        .ok_or(TokenomicsError::InvalidUsdAmount)?
        .parse::<u64>()
        .map_err(|_| TokenomicsError::InvalidUsdAmount)?;
    let cents = match parts.next() {
        Some(part) if part.len() <= 2 => {
            let padded = format!("{part:0<2}");
            padded
                .parse::<u64>()
                .map_err(|_| TokenomicsError::InvalidUsdAmount)?
        }
        Some(_) => return Err(TokenomicsError::InvalidUsdAmount),
        None => 0,
    };

    if parts.next().is_some() {
        return Err(TokenomicsError::InvalidUsdAmount);
    }

    dollars
        .checked_mul(100)
        .and_then(|amount| amount.checked_add(cents))
        .ok_or(TokenomicsError::InvalidUsdAmount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tokenomics_is_valid() {
        let plan = TokenomicsPlan::abyss_default();

        assert_eq!(plan.validate(), Ok(()));
        assert_eq!(plan.max_supply, Coin::from_ac(55_000_000).unwrap());
    }

    #[test]
    fn team_reserve_and_public_sale_reconcile_to_55m() {
        let plan = TokenomicsPlan::abyss_default();

        let team_reserve = plan.team_reserve_amount();
        let public_sale = plan
            .allocation_amount("Public sale and liquidity formation")
            .unwrap();

        assert_eq!(team_reserve, Coin::from_ac(30_000_000).unwrap());
        assert_eq!(public_sale, Coin::from_ac(25_000_000).unwrap());
        assert_eq!(
            team_reserve.checked_add(public_sale).unwrap(),
            Coin::from_ac(55_000_000).unwrap()
        );
    }

    #[test]
    fn sale_rounds_fit_public_allocation() {
        let plan = TokenomicsPlan::abyss_default();
        let public_sale = plan
            .allocation_amount("Public sale and liquidity formation")
            .unwrap();
        let round_total = plan
            .sale_rounds
            .iter()
            .try_fold(Coin::ZERO, |acc, round| acc.checked_add(round.token_cap))
            .unwrap();

        // 2M + 3M + 5M + 5M + 10M = 25M (Standard rounds only;
        // Buyback and DEX final sale are tracked separately).
        assert_eq!(round_total, Coin::from_ac(25_000_000).unwrap());
        assert_eq!(public_sale, Coin::from_ac(25_000_000).unwrap());
    }

    #[test]
    fn total_sale_cap_matches_expected_raise() {
        let plan = TokenomicsPlan::abyss_default();

        // 2M*$1 + 3M*$2 + 5M*$3 + 5M*$4 + 10M*$5
        // = $2M + $6M + $15M + $20M + $50M = $93M
        let expected_cents = 93_000_000_00_u64;
        assert_eq!(plan.total_sale_cap_usd_cents(), Some(expected_cents));
    }

    #[test]
    fn calculates_tokens_for_contribution() {
        let round = SaleRound::new("Pre-Sale", 3_000_000, 200, 0, 0);

        assert_eq!(
            round.tokens_for_usd_cents(1_000_00),
            Ok(Coin::from_ac(500).unwrap())
        );
    }

    #[test]
    fn sale_to_investors_enforces_500k_minimum() {
        let round = SaleRound::new("Sale to Investors", 2_000_000, 100, 500_000, 0);

        assert_eq!(
            round.tokens_for_usd_cents(499_999_00),
            Err(TokenomicsError::ContributionBelowMinimum)
        );
        assert_eq!(
            round.tokens_for_usd_cents(500_000_00),
            Ok(Coin::from_ac(500_000).unwrap())
        );
    }

    #[test]
    fn creates_contribution_receipt_for_approved_investor() {
        let round = SaleRound::new("Sale Stage 1", 5_000_000, 300, 0, 0);
        let investor = InvestorProfile {
            investor_id: "investor-001".to_string(),
            jurisdiction: "EU".to_string(),
            kyc_status: KycStatus::Approved,
            accredited_or_professional: false,
            max_contribution_usd_cents: 10_000_00,
        };

        let receipt = ContributionReceipt::create(&investor, &round, 900_00).unwrap();

        assert_eq!(receipt.token_amount, Coin::from_ac(300).unwrap());
    }

    #[test]
    fn sale_to_investors_requires_professional_investor() {
        let round = SaleRound::new("Sale to Investors", 2_000_000, 100, 500_000, 0);
        let investor = InvestorProfile {
            investor_id: "retail-001".to_string(),
            jurisdiction: "EU".to_string(),
            kyc_status: KycStatus::Approved,
            accredited_or_professional: false,
            max_contribution_usd_cents: 1_000_000_00,
        };

        assert_eq!(
            ContributionReceipt::create(&investor, &round, 500_000_00),
            Err(TokenomicsError::ProfessionalInvestorRequired)
        );
    }

    #[test]
    fn finds_round_by_id() {
        let plan = TokenomicsPlan::abyss_default();

        assert_eq!(
            plan.sale_round("public-stage-1").map(|round| round.name),
            Some("Sale Stage 1")
        );
        assert_eq!(
            plan.sale_round("sale-to-investors").map(|round| round.name),
            Some("Sale to Investors")
        );
    }

    #[test]
    fn parses_usd_amounts() {
        assert_eq!(parse_usd_to_cents("$1,250.50"), Ok(125_050));
        assert_eq!(parse_usd_to_cents("900"), Ok(90_000));
        assert_eq!(
            parse_usd_to_cents("1.234"),
            Err(TokenomicsError::InvalidUsdAmount)
        );
    }

    // ── Team vesting tests ──

    #[test]
    fn team_vesting_tranches_sum_to_30m() {
        let vesting = TeamVesting::abyss_default();

        assert_eq!(vesting.tranche_a_total, Coin::from_ac(10_000_000).unwrap());
        assert_eq!(vesting.tranche_b_total, Coin::from_ac(20_000_000).unwrap());
        assert_eq!(vesting.total(), Coin::from_ac(30_000_000).unwrap());
    }

    #[test]
    fn tranche_a_unlocks_linearly_over_12_months() {
        let vesting = TeamVesting::abyss_default();

        assert_eq!(vesting.tranche_a_unlocked(0), Coin::ZERO);
        assert_eq!(
            vesting.tranche_a_unlocked(6),
            Coin::from_ac(5_000_000).unwrap()
        );
        assert_eq!(
            vesting.tranche_a_unlocked(12),
            Coin::from_ac(10_000_000).unwrap()
        );
        // fully unlocked stays fully unlocked afterwards
        assert_eq!(
            vesting.tranche_a_unlocked(24),
            Coin::from_ac(10_000_000).unwrap()
        );
    }

    #[test]
    fn tranche_b_unlocks_5m_per_year_over_4_years() {
        let vesting = TeamVesting::abyss_default();

        assert_eq!(vesting.tranche_b_unlocked(0), Coin::ZERO);
        assert_eq!(
            vesting.tranche_b_unlocked(12),
            Coin::from_ac(5_000_000).unwrap()
        );
        assert_eq!(
            vesting.tranche_b_unlocked(24),
            Coin::from_ac(10_000_000).unwrap()
        );
        assert_eq!(
            vesting.tranche_b_unlocked(36),
            Coin::from_ac(15_000_000).unwrap()
        );
        assert_eq!(
            vesting.tranche_b_unlocked(48),
            Coin::from_ac(20_000_000).unwrap()
        );
    }

    #[test]
    fn year_by_year_unlock_matches_published_schedule() {
        let vesting = TeamVesting::abyss_default();

        // Year 1: Tranche A fully unlocked (10M) + Tranche B year-1 (5M) = 15M
        assert_eq!(
            vesting.unlocked_in_year(1),
            Coin::from_ac(15_000_000).unwrap()
        );
        // Years 2-4: Tranche B only, 5M/year
        assert_eq!(
            vesting.unlocked_in_year(2),
            Coin::from_ac(5_000_000).unwrap()
        );
        assert_eq!(
            vesting.unlocked_in_year(3),
            Coin::from_ac(5_000_000).unwrap()
        );
        assert_eq!(
            vesting.unlocked_in_year(4),
            Coin::from_ac(5_000_000).unwrap()
        );
    }

    #[test]
    fn total_unlocked_across_both_tranches_never_exceeds_30m() {
        let vesting = TeamVesting::abyss_default();

        assert_eq!(
            vesting.total_unlocked(48),
            Coin::from_ac(30_000_000).unwrap()
        );
        assert_eq!(
            vesting.total_unlocked(100),
            Coin::from_ac(30_000_000).unwrap()
        );
    }

    // ── Buyback tests ──

    #[test]
    fn buyback_offers_2m_tokens_at_3_dollars() {
        let buyback = BuybackOffer::abyss_default();

        assert_eq!(buyback.token_cap, Coin::from_ac(2_000_000).unwrap());
        assert_eq!(buyback.price_usd_cents, 300);
        assert_eq!(buyback.max_payout_usd_cents(), 6_000_000_00);
    }

    #[test]
    fn buyback_payout_scales_with_token_amount() {
        let buyback = BuybackOffer::abyss_default();

        assert_eq!(
            buyback.payout_usd_cents(Coin::from_ac(1_000_000).unwrap()),
            Some(3_000_000_00)
        );
    }

    // ── DEX final sale tests ──

    #[test]
    fn dex_final_sale_prices_tokens_at_5_dollars() {
        let dex_sale = DexFinalSale::abyss_default();

        assert_eq!(
            dex_sale.tokens_for_usd_cents(500_00),
            Ok(Coin::from_ac(100).unwrap())
        );
        assert_eq!(dex_sale.price_usd_cents, 500);
    }
}
