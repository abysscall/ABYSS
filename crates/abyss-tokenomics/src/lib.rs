//! Tokenomics and sale planning primitives for ABYSS.
//!
//! This module is a deterministic planning model, not a legal wrapper and not
//! an on-chain sale contract. Any real token sale must be reviewed by qualified
//! counsel in the target jurisdictions before accepting funds.

use abyss_core::{Coin, COIN, MAX_SUPPLY};

pub const BASIS_POINTS: u16 = 10_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenomicsPlan {
    pub symbol: &'static str,
    pub max_supply: Coin,
    pub allocations: Vec<Allocation>,
    pub sale_rounds: Vec<SaleRound>,
}

impl TokenomicsPlan {
    pub fn abyss_default() -> Self {
        Self {
            symbol: "AC",
            max_supply: Coin::MAX,
            allocations: vec![
                Allocation::new("Validator rewards and network security", 2_500),
                Allocation::new("Ecosystem grants, apps, audits, bug bounties", 2_000),
                Allocation::new("Public sale and liquidity formation", 2_000),
                Allocation::new("Foundation treasury with long vesting", 1_500),
                Allocation::new("Core contributors with long vesting", 1_000),
                Allocation::new("DEX liquidity reserve", 1_000),
            ],
            sale_rounds: vec![
                SaleRound::new("Strategic round", 2_000_000, 100, 100_000, 24),
                SaleRound::new("Private presale", 3_000_000, 200, 250, 18),
                SaleRound::new("Public presale stage I", 4_000_000, 300, 50, 12),
                SaleRound::new("Launch liquidity round", 2_000_000, 500, 25, 6),
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
}

impl Allocation {
    pub const fn new(name: &'static str, basis_points: u16) -> Self {
        Self { name, basis_points }
    }

    pub fn amount(&self) -> Coin {
        let units = (MAX_SUPPLY / BASIS_POINTS as u64) * self.basis_points as u64;
        Coin::from_micro_ac(units).expect("allocation is derived from capped max supply")
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
            "Strategic round" => "strategic",
            "Private presale" => "private",
            "Public presale stage I" => "public-stage-1",
            "Launch liquidity round" => "launch-liquidity",
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

        assert_eq!(public_sale, Coin::from_ac(11_000_000).unwrap());
        assert_eq!(round_total, Coin::from_ac(11_000_000).unwrap());
    }

    #[test]
    fn calculates_tokens_for_contribution() {
        let round = SaleRound::new("Private presale", 3_000_000, 200, 250, 18);

        assert_eq!(
            round.tokens_for_usd_cents(1_000_00),
            Ok(Coin::from_ac(500).unwrap())
        );
    }

    #[test]
    fn rejects_small_tickets() {
        let round = SaleRound::new("Private presale", 3_000_000, 200, 250, 18);

        assert_eq!(
            round.tokens_for_usd_cents(249_99),
            Err(TokenomicsError::ContributionBelowMinimum)
        );
    }

    #[test]
    fn creates_contribution_receipt_for_approved_investor() {
        let round = SaleRound::new("Public presale stage I", 4_000_000, 300, 50, 12);
        let investor = InvestorProfile {
            investor_id: "investor-001".to_string(),
            jurisdiction: "EU".to_string(),
            kyc_status: KycStatus::Approved,
            accredited_or_professional: false,
            max_contribution_usd_cents: 10_000_00,
        };

        let receipt = ContributionReceipt::create(&investor, &round, 900_00).unwrap();

        assert_eq!(receipt.token_amount, Coin::from_ac(300).unwrap());
        assert_eq!(receipt.vesting.unlocked_amount(receipt.token_amount, 11), Coin::ZERO);
        assert_eq!(
            receipt.vesting.unlocked_amount(receipt.token_amount, 12),
            receipt.token_amount
        );
    }

    #[test]
    fn strategic_round_requires_professional_investor() {
        let round = SaleRound::new("Strategic round", 2_000_000, 100, 100_000, 24);
        let investor = InvestorProfile {
            investor_id: "retail-001".to_string(),
            jurisdiction: "EU".to_string(),
            kyc_status: KycStatus::Approved,
            accredited_or_professional: false,
            max_contribution_usd_cents: 200_000_00,
        };

        assert_eq!(
            ContributionReceipt::create(&investor, &round, 100_000_00),
            Err(TokenomicsError::ProfessionalInvestorRequired)
        );
    }

    #[test]
    fn finds_round_by_id() {
        let plan = TokenomicsPlan::abyss_default();

        assert_eq!(
            plan.sale_round("public-stage-1").map(|round| round.name),
            Some("Public presale stage I")
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
}
