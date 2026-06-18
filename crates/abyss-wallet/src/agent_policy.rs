use abyss_core::Coin;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentPolicy {
    permissions: Vec<AgentPermission>,
    agent_trade_limit: Coin,
}

impl Default for AgentPolicy {
    fn default() -> Self {
        Self {
            permissions: vec![
                AgentPermission::ReadPortfolio,
                AgentPermission::DraftTransactions,
                AgentPermission::DetectScams,
            ],
            agent_trade_limit: Coin::ZERO,
        }
    }
}

impl AgentPolicy {
    pub fn permissions(&self) -> &[AgentPermission] {
        &self.permissions
    }

    pub fn grant(&mut self, permission: AgentPermission) {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
        }
    }

    pub fn revoke(&mut self, permission: AgentPermission) {
        self.permissions.retain(|item| *item != permission);
    }

    pub fn set_agent_trade_limit(&mut self, limit: Coin) {
        self.agent_trade_limit = limit;
    }

    pub fn transaction_allowed(&self, amount: Coin) -> bool {
        if self.permissions.contains(&AgentPermission::ExecuteLimitedTrades) {
            return amount <= self.agent_trade_limit;
        }

        amount.is_zero()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentPermission {
    ReadPortfolio,
    DraftTransactions,
    ExecuteLimitedTrades,
    DetectScams,
    ModerateSocial,
    SummarizePrivateMessages,
    AnalyzeGovernance,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_agent_cannot_execute_nonzero_trade() {
        let policy = AgentPolicy::default();

        assert!(!policy.transaction_allowed(Coin::from_ac(1).unwrap()));
    }

    #[test]
    fn limited_trade_permission_respects_limit() {
        let mut policy = AgentPolicy::default();
        policy.grant(AgentPermission::ExecuteLimitedTrades);
        policy.set_agent_trade_limit(Coin::from_ac(5).unwrap());

        assert!(policy.transaction_allowed(Coin::from_ac(5).unwrap()));
        assert!(!policy.transaction_allowed(Coin::from_ac(6).unwrap()));
    }
}
