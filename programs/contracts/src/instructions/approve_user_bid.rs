use anchor_lang::prelude::*;
use crate::state::{product::*, user_bid::*};
use crate::errors::*;

#[derive(Accounts)]
pub struct ApproveBid<'info> {
    #[account(mut)]
    pub product_owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"product", product.maker.as_ref()],
        bump = product.bump,
        constraint = product.maker == product_owner.key() @ ContractError::UnauthorizedAccess,
    )]
    pub product: Account<'info, Product>,

    #[account(
        mut,
        seeds = [b"bid", product.key().as_ref(), user_bid.user.as_ref()],
        bump = user_bid.bump
    )]
    pub user_bid: Account<'info, UserBid>,
}

pub fn handler(ctx: Context<ApproveBid>) -> Result<()> {
    let user_bid = &mut ctx.accounts.user_bid;

    // Validate status
    match user_bid.status {
        BidStatus::Pending => {}
        _ => return err!(ContractError::BidAlreadyProcessed),
    }

    user_bid.status = BidStatus::Approved;
    user_bid.tokens_claimed = false;

    // Track approved bids count on product
    ctx.accounts.product.approved_bids += 1;

    Ok(())
}
