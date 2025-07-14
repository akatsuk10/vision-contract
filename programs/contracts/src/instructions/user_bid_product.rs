use anchor_lang::prelude::*;
use anchor_lang::solana_program::{system_instruction, program::invoke};
use crate::state::{product::*, user_bid::*};
use crate::errors::*;

#[derive(Accounts)]
#[instruction(slots_requested: u32)]
pub struct UserBidProduct<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"product", product.maker.as_ref()],
        bump = product.bump
    )]
    pub product: Account<'info, Product>,

    #[account(
        init,
        seeds = [b"bid", product.key().as_ref(), user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + UserBid::INIT_SPACE
    )]
    pub user_bid: Account<'info, UserBid>,

    /// CHECK: just holding lamports, verified by seeds
    #[account(
        mut,
        seeds = [b"treasury", product.key().as_ref()],
        bump
    )]
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<UserBidProduct>, amount: u64, slots_requested: u32) -> Result<()> {
    let clock = Clock::get()?;
    let product = &ctx.accounts.product;

    require!(clock.unix_timestamp < product.bid_close_date, ContractError::BidClosed);
    require!(slots_requested > 0 && slots_requested <= 5, ContractError::InvalidSlotCount);
    require!(amount > 0, ContractError::ZeroBidAmount);

    let token_amount = product.calculate_token_amount(amount)?;

    // Save user bid state
    let user_bid = &mut ctx.accounts.user_bid;
    user_bid.user = ctx.accounts.user.key();
    user_bid.product = product.key();
    user_bid.amount = amount;
    user_bid.token_amount = token_amount;
    user_bid.status = BidStatus::Pending;
    user_bid.tokens_claimed = false;
    user_bid.funds_claimed = false;
    user_bid.slots_requested = slots_requested;
    user_bid.bump = ctx.bumps.user_bid;

    // Transfer funds to treasury PDA (escrow)
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
