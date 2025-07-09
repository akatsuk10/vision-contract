use anchor_lang::prelude::*;
use crate::state::{product::*, user_bid::*};

#[error_code]
pub enum ErrorCode {
    AlreadyApproved,
}

#[derive(Accounts)]

pub struct ApproveUserBid<'info> {
    pub maker: Signer<'info>,

    #[account(mut, seeds = [b"product", maker.key().as_ref()], bump)]
    pub product: Account<'info, Product>,

    #[account(mut, seeds = [b"bid", product.key().as_ref(), user_bid.user.key().as_ref()], bump)]
    pub user_bid: Account<'info, UserBid>,
}

pub fn handler(ctx: Context<ApproveUserBid>) -> Result<()> {
    let bid = &mut ctx.accounts.user_bid;
    require!(!bid.approved, ErrorCode::AlreadyApproved);

    bid.approved = true;
    ctx.accounts.product.approved_bids += 1;

    Ok(())
}


