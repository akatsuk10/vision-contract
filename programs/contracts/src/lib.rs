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

    pub fn user_bid_product(ctx: Context<UserBidProduct>, amount: u64, slots_requested: u32) -> Result<()> {
        user_bid_product::handler(ctx, amount, slots_requested)
    }

    pub fn approve_bid(ctx: Context<ApproveBid>) -> Result<()> {
        approve_user_bid::handler(ctx)
    }

    pub fn reject_bid(ctx: Context<RejectBid>) -> Result<()> {
        reject_bid::handler(ctx)
    }

   
}
