use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum BidStatus {
    Pending,
    Approved,
    Rejected,
}

#[account]
pub struct UserBid {
    pub user: Pubkey,           // bidder address
    pub product: Pubkey,        // linked product
    pub amount: u64,            // total SOL bid amount
    pub token_amount: u64,      // tokens to be received on success
    pub slots_requested: u32,   // number of IPO slots requested (max 5)
    pub status: BidStatus,      // current bid status
    pub tokens_claimed: bool,   // has the user received tokens
    pub funds_claimed: bool,    // has funds been refunded
    pub created_at: i64,        // timestamp of the bid
    pub bump: u8,               // PDA bump
}

impl UserBid {
    pub const INIT_SPACE: usize = 
        8 +  // discriminator
        32 + // user
        32 + // product
        8 +  // amount
        8 +  // token_amount
        4 +  // slots_requested
        1 +  // status
        1 +  // tokens_claimed
        1 +  // funds_claimed
        8 +  // created_at
        1;   // bump
}
