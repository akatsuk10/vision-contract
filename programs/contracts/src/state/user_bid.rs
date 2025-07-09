use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum BidStatus {
    Pending,
    Approved,
    Rejected,
}

#[account]
pub struct UserBid {
    pub user: Pubkey,
    pub product: Pubkey,
    pub amount: u64,
    pub token_amount: u64,
    pub status: BidStatus,
    pub tokens_claimed: bool,
    pub funds_claimed: bool,
    pub created_at: i64,
    pub bump: u8,
}

impl UserBid {
    pub const INIT_SPACE: usize = 32 + // user
        32 + // product
        8 + // amount
        8 + // token_amount
        1 + // status
        1 + // tokens_claimed
        1 + // funds_claimed
        8 + // created_at
        1; // bump
}
