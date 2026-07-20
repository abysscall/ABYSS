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
    /// Max supply: 55,000,000 AC
    ///   - Team Reserve  : 30,000,000 AC (54.5%) вЂ” vested, see `TeamVesting`
    ///   - Public Sale   : 25,000,000 AC (45.5%) вЂ” distributed across sale_rounds
    pub fn abyss_default() -> Self {
        Self {
            symbol: "AC",
            max_supply: Coin::MAX,
            allocations: vec![
                // в”Ђв”Ђ Team Reserve side вЂ” sums to exactly 30,000,000 AC в”Ђв”Ђ
                Allocation::new("Validator rewards and network security", 1_500, 8_250_000),
                Allocation::new(
                    "Ecosystem grants, apps, audits, bug bounties",
                    1_000,
                    5_500_000,
                ),
                Allocation::new("Foundation treasury with long vesting", 1_000, 5_500_000),
                Allocation::new("Core contributors with long vesting", 1_000, 5_500_000),
                Allocation::new("DEX liquidity reserve", 1_000, 5_250_000),
                // в”Ђв”Ђ Public Sale side вЂ” sums to exactly 25,000,000 AC в”Ђв”Ђ
                Allocation::new("Public sale and liquidity formation", 4_500, 25_000_000),
            ],
            sale_rounds: vec![
                // 1. Sale to Investors вЂ” institutional, min ticket $500,000, max 4 slots
                SaleRound::new("Sale to Investors", 2_000_000, 100, 500_000, 0),
                // 2. Pre-Sale вЂ” early community allocation
                SaleRound::new("Pre-Sale", 3_000_000, 200, 0, 0),
                // 3. Sale Stage 1 вЂ” public open round
                SaleRound::new("Sale Stage 1", 5_000_000, 300, 0, 0),
                // 4. Sale Stage 2 вЂ” growth phase
                SaleRound::new("Sale Stage 2", 5_000_000, 400, 0, 0),
                // 5. Sale Stage 3 вЂ” pre-final public round
                SaleRound::new("Sale Stage 3", 10_000_000, 500, 0, 0),
                // NOTE: "Investor Secondary Window" (P2P secondary market,
                // Stage I investors only) and "Final Sale В· DEX" (variable
                // supply @ $5.00 via DEX test orders) are handled by
                // `InvestorSecondaryWindow` and `DexFinalSale` respectively вЂ”
                // they are not fixed-cap sale rounds.
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
            .find(|a| a.name == name)
            .map(|a| a.amount())
    }

    pub fn total_sale_cap_usd_cents(&self) -> Option<u64> {
        self.sale_rounds.iter().try_fold(0_u64, |acc, round| {
            acc.checked_add(round.raise_cap_usd_cents())
        })
    }

    pub fn sale_round(&self, round_id: &str) -> Option<&SaleRound> {
        self.sale_rounds
            .iter()
            .find(|r| r.id == round_id || r.name.eq_ignore_ascii_case(round_id))
    }

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

// в”Ђв”Ђ Investor Secondary Window в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
//
// Replaces the former "Buyback" mechanism.
//
// After Sale Stage 1 closes, a two-phase window opens for Stage I investors
// who wish to exit:
//
//   Phase A вЂ” Registration (14 days):
//     Stage I investors submit an intent listing stating how many tokens they
//     wish to sell. Minimum listing: 50% of their allocation (в‰Ґ 250,000 AC
//     per investor, since every Stage I slot is exactly 500,000 AC).
//
//   Phase B вЂ” Sales (until all listed tokens are sold):
//     Any new or existing participant may purchase listed tokens at the
//     fixed price of $3.00/AC. ABYSS facilitates the matching and records
//     the transfer; it does not itself purchase any tokens.
//
// Key legal distinction from a buyback:
//   ABYSS has NO obligation to purchase tokens. It acts as the operator
//   of a facilitated P2P secondary market, not as a counterparty.
//   Unsold tokens at the close of Phase B remain with their original holder.

/// Parameters governing the Investor Secondary Window.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvestorSecondaryWindow {
    pub id: &'static str,
    pub name: &'static str,
    /// Price at which listed tokens trade. Fixed at $3.00 (300 cents).
    pub price_usd_cents: u64,
    /// Duration of the registration phase in days.
    pub registration_days: u32,
    /// Minimum fraction of an investor's allocation they must list
    /// (expressed in basis points of their holding вЂ” 5000 = 50%).
    pub min_listing_bps: u16,
    /// Allocation per Stage I slot in AC. Used to compute the minimum
    /// listing in absolute token terms.
    pub stage1_slot_ac: u64,
}

impl InvestorSecondaryWindow {
    pub fn abyss_default() -> Self {
        Self {
            id: "investor-secondary-window",
            name: "Investor Secondary Window",
            price_usd_cents: 300,
            registration_days: 14,
            min_listing_bps: 5_000, // 50%
            stage1_slot_ac: 500_000,
        }
    }

    /// Minimum tokens a Stage I investor must list to participate.
    /// = stage1_slot_ac Г— min_listing_bps / 10_000
    pub fn min_listing_ac(&self) -> u64 {
        self.stage1_slot_ac * self.min_listing_bps as u64 / BASIS_POINTS as u64
    }

    /// USD payout to the seller for `tokens` AC at the fixed price.
    pub fn seller_payout_usd_cents(&self, tokens: Coin) -> Option<u64> {
        let ac_units = tokens.micro_ac() / COIN;
        ac_units.checked_mul(self.price_usd_cents)
    }

    /// Validate a listing request from a Stage I investor.
    pub fn validate_listing(&self, listing: &SecondaryListing) -> Result<(), TokenomicsError> {
        if !listing.is_stage1_investor {
            return Err(TokenomicsError::SecondaryWindowNotEligible);
        }
        let min = Coin::from_ac(self.min_listing_ac()).ok_or(TokenomicsError::SupplyOverflow)?;
        if listing.tokens_to_list < min {
            return Err(TokenomicsError::SecondaryListingBelowMinimum {
                min_ac: self.min_listing_ac(),
                actual_ac: listing.tokens_to_list.micro_ac() / COIN,
            });
        }
        Ok(())
    }
}

/// An investor's intent to list tokens during the Secondary Window.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecondaryListing {
    pub investor_id: String,
    pub tokens_to_list: Coin,
    /// Must be true вЂ” only Stage I investors may list.
    pub is_stage1_investor: bool,
}

impl SecondaryListing {
    pub fn new(
        investor_id: impl Into<String>,
        tokens_ac: u64,
        is_stage1_investor: bool,
    ) -> Result<Self, TokenomicsError> {
        Ok(Self {
            investor_id: investor_id.into(),
            tokens_to_list: Coin::from_ac(tokens_ac).ok_or(TokenomicsError::SupplyOverflow)?,
            is_stage1_investor,
        })
    }
}

// в”Ђв”Ђ DexFinalSale в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ

/// Final Sale вЂ” executed via test orders on the ABYSS DEX at $5.00/AC.
/// Variable supply; no fixed token cap.
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

    pub fn tokens_for_usd_cents(
        &self,
        contribution_usd_cents: u64,
    ) -> Result<Coin, TokenomicsError> {
        let micro_ac = contribution_usd_cents
            .checked_mul(COIN)
            .ok_or(TokenomicsError::SupplyOverflow)?
            .checked_div(self.price_usd_cents)
            .ok_or(TokenomicsError::InvalidRoundPrice)?;
        Coin::from_micro_ac(micro_ac).ok_or(TokenomicsError::SupplyOverflow)
    }
}

// в”Ђв”Ђ TeamVesting в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ

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

    pub fn tranche_a_unlocked(&self, elapsed_months: u16) -> Coin {
        VestingSchedule::linear(0, self.tranche_a_months)
            .unlocked_amount(self.tranche_a_total, elapsed_months)
    }

    pub fn tranche_b_unlocked(&self, elapsed_months: u16) -> Coin {
        VestingSchedule::linear(0, self.tranche_b_months)
            .unlocked_amount(self.tranche_b_total, elapsed_months)
    }

    pub fn total_unlocked(&self, elapsed_months: u16) -> Coin {
        self.tranche_a_unlocked(elapsed_months)
            .checked_add(self.tranche_b_unlocked(elapsed_months))
            .expect("combined unlock cannot exceed total")
    }

    pub fn unlocked_in_year(&self, year: u16) -> Coin {
        if year == 0 {
            return Coin::ZERO;
        }
        let months_end = year
            .saturating_mul(12)
            .min(self.tranche_b_months.max(self.tranche_a_months));
        let months_start = months_end.saturating_sub(12);
        self.total_unlocked(months_end)
            .checked_sub(self.total_unlocked(months_start))
            .expect("later unlock is never smaller")
    }
}

// в”Ђв”Ђ Supporting types в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ

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
        Coin::from_ac(self.exact_amount_ac).expect("allocation fits max supply")
    }
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

    pub fn tokens_for_usd_cents(
        &self,
        contribution_usd_cents: u64,
    ) -> Result<Coin, TokenomicsError> {
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
    SecondaryWindowNotEligible,
    SecondaryListingBelowMinimum { min_ac: u64, actual_ac: u64 },
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
        Some(part) if part.len() <= 2 => format!("{part:0<2}")
            .parse::<u64>()
            .map_err(|_| TokenomicsError::InvalidUsdAmount)?,
        Some(_) => return Err(TokenomicsError::InvalidUsdAmount),
        None => 0,
    };
    if parts.next().is_some() {
        return Err(TokenomicsError::InvalidUsdAmount);
    }
    dollars
        .checked_mul(100)
        .and_then(|a| a.checked_add(cents))
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
        let team = plan.team_reserve_amount();
        let public = plan
            .allocation_amount("Public sale and liquidity formation")
            .unwrap();
        assert_eq!(team, Coin::from_ac(30_000_000).unwrap());
        assert_eq!(public, Coin::from_ac(25_000_000).unwrap());
        assert_eq!(
            team.checked_add(public).unwrap(),
            Coin::from_ac(55_000_000).unwrap()
        );
    }

    #[test]
    fn sale_rounds_fit_public_allocation() {
        let plan = TokenomicsPlan::abyss_default();
        let public = plan
            .allocation_amount("Public sale and liquidity formation")
            .unwrap();
        let round_total = plan
            .sale_rounds
            .iter()
            .try_fold(Coin::ZERO, |acc, r| acc.checked_add(r.token_cap))
            .unwrap();
        assert_eq!(round_total, Coin::from_ac(25_000_000).unwrap());
        assert_eq!(public, Coin::from_ac(25_000_000).unwrap());
    }

    #[test]
    fn total_sale_cap_matches_expected_raise() {
        let plan = TokenomicsPlan::abyss_default();
        // 2M*$1 + 3M*$2 + 5M*$3 + 5M*$4 + 10M*$5 = $93M
        let expected_cents = 9_300_000_000_u64;
        assert_eq!(plan.total_sale_cap_usd_cents(), Some(expected_cents));
    }

    #[test]
    fn sale_to_investors_enforces_500k_minimum() {
        let round = SaleRound::new("Sale to Investors", 2_000_000, 100, 500_000, 0);
        assert_eq!(
            round.tokens_for_usd_cents(49_999_900),
            Err(TokenomicsError::ContributionBelowMinimum)
        );
        assert_eq!(
            round.tokens_for_usd_cents(50_000_000),
            Ok(Coin::from_ac(500_000).unwrap())
        );
    }

    // в”Ђв”Ђ Investor Secondary Window tests в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ

    #[test]
    fn secondary_window_parameters_are_correct() {
        let w = InvestorSecondaryWindow::abyss_default();
        assert_eq!(w.price_usd_cents, 300); // $3.00
        assert_eq!(w.registration_days, 14); // 2 weeks
        assert_eq!(w.min_listing_bps, 5_000); // 50%
        assert_eq!(w.min_listing_ac(), 250_000); // 50% of 500,000 AC
    }

    #[test]
    fn valid_stage1_listing_is_accepted() {
        let w = InvestorSecondaryWindow::abyss_default();
        let listing = SecondaryListing::new("investor-001", 300_000, true).unwrap();
        assert_eq!(w.validate_listing(&listing), Ok(()));
    }

    #[test]
    fn listing_below_minimum_is_rejected() {
        let w = InvestorSecondaryWindow::abyss_default();
        // 200,000 AC < minimum 250,000 AC
        let listing = SecondaryListing::new("investor-001", 200_000, true).unwrap();
        assert_eq!(
            w.validate_listing(&listing),
            Err(TokenomicsError::SecondaryListingBelowMinimum {
                min_ac: 250_000,
                actual_ac: 200_000,
            })
        );
    }

    #[test]
    fn non_stage1_investor_cannot_list() {
        let w = InvestorSecondaryWindow::abyss_default();
        let listing = SecondaryListing::new("presale-investor", 300_000, false).unwrap();
        assert_eq!(
            w.validate_listing(&listing),
            Err(TokenomicsError::SecondaryWindowNotEligible)
        );
    }

    #[test]
    fn seller_payout_is_correct_at_3_dollars() {
        let w = InvestorSecondaryWindow::abyss_default();
        // 300,000 AC Г— $3.00 = $900,000
        let tokens = Coin::from_ac(300_000).unwrap();
        assert_eq!(w.seller_payout_usd_cents(tokens), Some(90_000_000));
    }

    #[test]
    fn full_slot_listing_payout_is_1_5m() {
        let w = InvestorSecondaryWindow::abyss_default();
        // 500,000 AC Г— $3.00 = $1,500,000
        let tokens = Coin::from_ac(500_000).unwrap();
        assert_eq!(w.seller_payout_usd_cents(tokens), Some(150_000_000));
    }

    #[test]
    fn stage1_investor_3x_return_on_entry_price() {
        // Bought at $1.00, sells at $3.00 в†’ 3Г— return confirmed by payout math
        let w = InvestorSecondaryWindow::abyss_default();
        let tokens = Coin::from_ac(500_000).unwrap();
        let payout = w.seller_payout_usd_cents(tokens).unwrap();
        let entry_cost = 500_000_u64 * 100; // 500,000 AC Г— $1.00 in cents
        assert_eq!(payout, entry_cost * 3);
    }

    // в”Ђв”Ђ DEX final sale в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ

    #[test]
    fn dex_final_sale_prices_at_5_dollars() {
        let dex = DexFinalSale::abyss_default();
        assert_eq!(
            dex.tokens_for_usd_cents(50_000),
            Ok(Coin::from_ac(100).unwrap())
        );
    }

    // в”Ђв”Ђ Vesting в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ

    #[test]
    fn team_vesting_tranches_sum_to_30m() {
        let v = TeamVesting::abyss_default();
        assert_eq!(v.total(), Coin::from_ac(30_000_000).unwrap());
    }

    #[test]
    fn year_by_year_unlock_matches_published_schedule() {
        let v = TeamVesting::abyss_default();
        assert_eq!(v.unlocked_in_year(1), Coin::from_ac(15_000_000).unwrap()); // A+B yr1
        assert_eq!(v.unlocked_in_year(2), Coin::from_ac(5_000_000).unwrap()); // B only
        assert_eq!(v.unlocked_in_year(3), Coin::from_ac(5_000_000).unwrap());
        assert_eq!(v.unlocked_in_year(4), Coin::from_ac(5_000_000).unwrap());
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
}

