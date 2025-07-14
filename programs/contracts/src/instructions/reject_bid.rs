use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke_signed, system_instruction};
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
    pub treasury: UncheckedAccount<'info>,

    /// CHECK: User account to receive refund
    #[account(
        mut,
        constraint = user_account.key() == user_bid.user @ ContractError::UnauthorizedAccess
    )]
    pub user_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RejectBid>) -> Result<()> {
    let user_bid = &mut ctx.accounts.user_bid;
    let product = &ctx.accounts.product;
    let treasury = &ctx.accounts.treasury;
    let user_account = &ctx.accounts.user_account;

    // Store the refund amount before modifying the bid
    let refund_amount = user_bid.amount;

    // Verify treasury has sufficient balance
    let treasury_balance = treasury.lamports();
    require!(treasury_balance >= refund_amount, ContractError::InsufficientFunds);

    // Reject the bid
    user_bid.status = BidStatus::Rejected;
    user_bid.funds_claimed = true;

    // Create the seeds for the treasury PDA
    let product_key = product.key();
    let treasury_seeds = &[
        b"treasury",
        product_key.as_ref(),
        &[ctx.bumps.treasury],
    ];
    let treasury_signer = &[&treasury_seeds[..]];

    // Method 1: Direct lamport transfer (recommended for PDAs)
    **treasury.try_borrow_mut_lamports()? -= refund_amount;
    **user_account.try_borrow_mut_lamports()? += refund_amount;

    msg!("Bid rejected and {} lamports refunded to user", refund_amount);

    Ok(())
}

// Alternative method using invoke_signed (if the above doesn't work)
pub fn handler_alternative(ctx: Context<RejectBid>) -> Result<()> {
    let user_bid = &mut ctx.accounts.user_bid;
    let product = &ctx.accounts.product;
    let treasury = &ctx.accounts.treasury;
    let user_account = &ctx.accounts.user_account;

    // Store the refund amount before modifying the bid
    let refund_amount = user_bid.amount;

    // Verify treasury has sufficient balance
    let treasury_balance = treasury.lamports();
    require!(treasury_balance >= refund_amount, ContractError::InsufficientFunds);

    // Reject the bid
    user_bid.status = BidStatus::Rejected;
    user_bid.funds_claimed = true;

    // Create the seeds for the treasury PDA
    let product_key = product.key();
    let treasury_seeds = &[
        b"treasury",
        product_key.as_ref(),
        &[ctx.bumps.treasury],
    ];
    let treasury_signer = &[&treasury_seeds[..]];

    // Transfer SOL from treasury PDA back to user using invoke_signed
    invoke_signed(
        &system_instruction::transfer(
            &treasury.key(),
            &user_account.key(),
            refund_amount,
        ),
        &[
            treasury.to_account_info(),
            user_account.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        treasury_signer,
    )?;

    msg!("Bid rejected and {} lamports refunded to user", refund_amount);

    Ok(())
}