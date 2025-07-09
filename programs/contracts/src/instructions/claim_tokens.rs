use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, transfer};
use crate::state::{product::*, user_bid::*};
use crate::errors::*;

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"product", product.maker.key().as_ref()],
        bump = product.bump
    )]
    pub product: Account<'info, Product>,

    #[account(
        mut,
        seeds = [b"bid", product.key().as_ref(), user.key().as_ref()],
        bump = user_bid.bump,
        constraint = user_bid.user == user.key() @ ContractError::UnauthorizedAccess,
        constraint = user_bid.status == BidStatus::Approved @ ContractError::BidNotApproved,
        constraint = !user_bid.tokens_claimed @ ContractError::TokensAlreadyClaimed
    )]
    pub user_bid: Account<'info, UserBid>,

    #[account(
        mut,
        associated_token::mint = product.token_mint,
        associated_token::authority = pool_authority
    )]
    pub token_pool: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = product.token_mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA that owns the token pool
    #[account(seeds = [b"pool", product.key().as_ref()], bump)]
    pub pool_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ClaimTokens>) -> Result<()> {
    let product = &ctx.accounts.product;
    let user_bid = &mut ctx.accounts.user_bid;

    // Check if launch date has arrived
    require!(product.is_launched(), ContractError::LaunchDateNotReached);

    // Transfer tokens from pool to user
    let product_key = product.key();
    let pool_seeds = &[b"pool", product_key.as_ref(), &[ctx.bumps.pool_authority]];
    let pool_signer = &[&pool_seeds[..]];

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.token_pool.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            pool_signer,
        ),
        user_bid.token_amount,
    )?;

    user_bid.tokens_claimed = true;

    Ok(())
}