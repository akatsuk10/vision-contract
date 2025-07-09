use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};
use crate::state::{product::*, user_bid::*};
use crate::errors::*;

#[derive(Accounts)]
pub struct RejectBid<'info> {
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

    #[account(
        mut,
        seeds = [b"treasury", product.key().as_ref()],
        bump,
    )]
    /// CHECK: Treasury PDA for holding SOL
    pub treasury: SystemAccount<'info>,

    /// CHECK: User account to receive refund
    #[account(
        mut,
        constraint = user_account.key() == user_bid.user @ ContractError::UnauthorizedAccess
    )]
    pub user_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RejectBid>) -> Result<()> {
    let user_bid = &mut ctx.accounts.user_bid;
    let product = &ctx.accounts.product;

    // Reject the bid
    user_bid.status = BidStatus::Rejected;

    // Refund the user immediately
    let product_key = product.key();
    let treasury_seeds = &[b"treasury", product_key.as_ref(), &[ctx.bumps.treasury]];
    let treasury_signer = &[&treasury_seeds[..]];

    invoke(
        &system_instruction::transfer(
            ctx.accounts.treasury.key,
            ctx.accounts.user_account.key,
            user_bid.amount,
        ),
        &[
            ctx.accounts.treasury.to_account_info(),
            ctx.accounts.user_account.to_account_info(),
        ],
    )?;

    user_bid.funds_claimed = true;

    Ok(())
}
