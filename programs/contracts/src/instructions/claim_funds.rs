use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};
use crate::state::product::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct ClaimFunds<'info> {
    #[account(mut)]
    pub product_owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"product", product_owner.key().as_ref()],
        bump = product.bump,
        constraint = product.maker == product_owner.key() @ ContractError::UnauthorizedAccess,
        constraint = !product.funds_claimed @ ContractError::FundsAlreadyClaimed
    )]
    pub product: Account<'info, Product>,

    #[account(
        mut,
        seeds = [b"treasury", product.key().as_ref()],
        bump,
    )]
    /// CHECK: Treasury PDA for holding SOL
    pub treasury: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimFunds>) -> Result<()> {
    let product = &mut ctx.accounts.product;

    // Check if launch date has arrived
    require!(product.is_launched(), ContractError::LaunchDateNotReached);

    // Calculate total approved bid amount
    let total_approved_amount = product.approved_bids as u64 * product.get_token_price()? * product.total_token_supply / product.ipo_slots as u64;

    // Transfer funds from treasury to product owner
    let product_key = product.key();
    let treasury_seeds = &[b"treasury", product_key.as_ref(), &[ctx.bumps.treasury]];
    let treasury_signer = &[&treasury_seeds[..]];

    invoke(
        &system_instruction::transfer(
            ctx.accounts.treasury.key,
            ctx.accounts.product_owner.key,
            total_approved_amount + product.initial_deposit,
        ),
        &[
            ctx.accounts.treasury.to_account_info(),
            ctx.accounts.product_owner.to_account_info(),
        ],
    )?;

    product.funds_claimed = true;
    product.phase = ProductPhase::Launched;

    Ok(())
}