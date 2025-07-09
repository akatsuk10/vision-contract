use anchor_lang::prelude::*;
use crate::errors::ContractError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum ProductPhase {
    Bidding,
    Launched,
    Completed,
}

#[account]
pub struct Product {
    pub maker: Pubkey,
    pub name: String,
    pub description: String,
    pub token_symbol: String,
    pub initial_deposit: u64,
    pub ipo_slots: u32,
    pub approved_bids: u32,
    pub total_token_supply: u64,
    pub token_mint: Pubkey,
    pub token_pool: Pubkey,
    pub launch_date: i64,
    pub bid_close_date: i64,
    pub phase: ProductPhase,
    pub created_at: i64,
    pub funds_claimed: bool,
    pub bump: u8,
}

impl Product {
    pub const INIT_SPACE: usize = 32 + // maker
        4 + 50 + // name (max 50 chars)
        4 + 200 + // description (max 200 chars)
        4 + 10 + // token_symbol (max 10 chars)
        8 + // initial_deposit
        4 + // ipo_slots
        4 + // approved_bids
        8 + // total_token_supply
        32 + // token_mint
        32 + // token_pool
        8 + // launch_date
        8 + // bid_close_date
        1 + // phase
        8 + // created_at
        1 + // funds_claimed
        1; // bump

    pub fn get_token_price(&self) -> Result<u64> {
        if self.total_token_supply == 0 {
            return Err(ContractError::ArithmeticOverflow.into());
        }
        // Price per token in lamports
        Ok(self.initial_deposit / self.total_token_supply)
    }

    pub fn calculate_token_amount(&self, sol_amount: u64) -> Result<u64> {
        let token_price = self.get_token_price()?;
        if token_price == 0 {
            return Err(ContractError::ArithmeticOverflow.into());
        }
        Ok(sol_amount / token_price)
    }

    pub fn is_bidding_open(&self) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        self.phase == ProductPhase::Bidding && now < self.bid_close_date
    }

    pub fn is_launched(&self) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        now >= self.launch_date
    }

    pub fn can_approve_more_bids(&self) -> bool {
        self.approved_bids < self.ipo_slots
    }
}