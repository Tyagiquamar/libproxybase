use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BalanceState {
    pub wallet_address: String,
    pub buyer_available: i64,
    pub buyer_reserved: i64,
    pub buyer_spent: i64,
    pub seller_pending: i64,
    pub seller_available: i64,
    pub seller_payout_locked: i64,
    pub spendable_balance: i64,
}

impl BalanceState {
    /// Client-side display of available funds for the buyer role.
    pub fn buyer_available(&self) -> i64 {
        self.buyer_available
    }

    /// Total spendable (buyer_available + seller_available).
    pub fn total_spendable(&self) -> i64 {
        self.spendable_balance
    }
}
