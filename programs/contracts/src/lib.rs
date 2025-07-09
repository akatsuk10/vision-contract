use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;

use instructions::*;
use errors::*;

declare_id!("3oqHGMC4cKpkMUXbpRUUjAG9RSTSRohpUco3zPBkCEvg");

#[program]
pub mod contracts {
    use super::*;

    pub fn init_protocol(ctx: Context<InitProtocol>) -> Result<()> {
        init_protocol::handler(ctx)
    }

    pub fn launch_product(ctx: Context<LaunchProduct>, args: LaunchProductArgs) -> Result<()> {
        launch_product::handler(ctx, args)
    }

    pub fn user_bid_product(ctx: Context<UserBidProduct>, amount: u64) -> Result<()> {
        user_bid_product::handler(ctx, amount)
    }

    pub fn approve_bid(ctx: Context<ApproveBid>) -> Result<()> {
        approve_bid::handler(ctx)
    }

    pub fn reject_bid(ctx: Context<RejectBid>) -> Result<()> {
        reject_bid::handler(ctx)
    }

    pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
        claim_tokens::handler(ctx)
    }

    pub fn claim_funds(ctx: Context<ClaimFunds>) -> Result<()> {
        claim_funds::handler(ctx)
    }
}
