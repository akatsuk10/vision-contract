use anchor_lang::prelude::*;
use crate::state::{product::*, user_bid::*};
use crate::errors::*;

#[derive(Accounts)]
pub struct ApproveBid<'info> {
    #[account(mut)]
    pub product_owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"product", product_owner.key().as_ref()],
        bump = product.bump,
        constraint = product.maker == product_owner.key() @ ContractError::UnauthorizedAccess
    )]
    pub product: Account<'info, Product>,

    #[account(
        mut,
        seeds = [b"bid", product.key().as_ref(), user_bid.user.key().as_ref()],
        bump = user_bid.bump,
        constraint = user_bid.status == BidStatus::Pending @ ContractError::BidAlreadyProcessed
    )]
    pub user_bid: Account<'info, UserBid>,
}

pub fn handler(ctx: Context<ApproveBid>) -> Result<()> {
    let product = &mut ctx.accounts.product;
    let user_bid = &mut ctx.accounts.user_bid;

    // Check if we can approve more bids
    require!(product.can_approve_more_bids(), ContractError::AllSlotsFilled);

    // Approve the bid
    user_bid.status = BidStatus::Approved;
    product.approved_bids += 1;

    Ok(())
}
