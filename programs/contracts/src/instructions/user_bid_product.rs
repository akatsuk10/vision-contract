use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};
use crate::state::{product::*, user_bid::*};
use crate::errors::*;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct UserBidProduct<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"product", product.maker.key().as_ref()],
        bump = product.bump
    )]
    pub product: Account<'info, Product>,

    #[account(
        init,
        seeds = [b"bid", product.key().as_ref(), user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + UserBid::INIT_SPACE,
    )]
    pub user_bid: Account<'info, UserBid>,

    #[account(
        mut,
        seeds = [b"treasury", product.key().as_ref()],
        bump,
    )]
    /// CHECK: Treasury PDA for holding SOL
    pub treasury: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<UserBidProduct>, amount: u64) -> Result<()> {
    require!(amount > 0, ContractError::ZeroBidAmount);

    let product = &ctx.accounts.product;
    
    // Check if bidding is still open
    require!(product.is_bidding_open(), ContractError::BiddingPeriodEnded);

    // Calculate token amount
    let token_amount = product.calculate_token_amount(amount)?;

    // Initialize user bid
    let user_bid = &mut ctx.accounts.user_bid;
    user_bid.user = ctx.accounts.user.key();
    user_bid.product = product.key();
    user_bid.amount = amount;
    user_bid.token_amount = token_amount;
    user_bid.status = BidStatus::Pending;
    user_bid.tokens_claimed = false;
    user_bid.funds_claimed = false;
    user_bid.created_at = Clock::get()?.unix_timestamp;
    user_bid.bump = ctx.bumps.user_bid;

    // Transfer SOL to treasury (escrow)
    invoke(
        &system_instruction::transfer(
            ctx.accounts.user.key,
            ctx.accounts.treasury.key,
            amount,
        ),
        &[
            ctx.accounts.user.to_account_info(),
            ctx.accounts.treasury.to_account_info(),
        ],
    )?;

    Ok(())
}