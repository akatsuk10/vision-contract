use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};
use anchor_spl::token::{Mint, Token, TokenAccount, MintTo, mint_to};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::product::*;
use crate::errors::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct LaunchProductArgs {
    pub name: String,
    pub description: String,
    pub token_symbol: String,
    pub initial_deposit: u64,
    pub ipo_slots: u32,
    pub initial_token_supply: u64,
    pub launch_date: i64,
}

#[derive(Accounts)]
#[instruction(args: LaunchProductArgs)]
pub struct LaunchProduct<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        init,
        seeds = [b"product", maker.key().as_ref()],
        bump,
        payer = maker,
        space = 8 + Product::INIT_SPACE,
    )]
    pub product: Account<'info, Product>,

    #[account(
        init,
        seeds = [b"treasury", product.key().as_ref()],
        bump,
        payer = maker,
        space = 0,
    )]
    /// CHECK: Just a SOL-holding PDA
    pub treasury: UncheckedAccount<'info>,

    #[account(
        init,
        payer = maker,
        mint::decimals = 9,
        mint::authority = pool_authority,
        mint::freeze_authority = pool_authority
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = token_mint,
        associated_token::authority = pool_authority
    )]
    pub token_pool: Account<'info, TokenAccount>,

    /// CHECK: pool authority PDA
    #[account(seeds = [b"pool", product.key().as_ref()], bump)]
    pub pool_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<LaunchProduct>, args: LaunchProductArgs) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    require!(args.initial_deposit > 0, ContractError::ZeroBidAmount);
    require!(args.ipo_slots > 0, ContractError::ZeroBidAmount);
    require!(args.initial_token_supply > 0, ContractError::ZeroBidAmount);
    require!(args.launch_date > now, ContractError::InvalidLaunchDate);
    require!(args.name.len() <= 50, ContractError::ZeroBidAmount);
    require!(args.description.len() <= 200, ContractError::ZeroBidAmount);
    require!(args.token_symbol.len() <= 10, ContractError::ZeroBidAmount);

    let product = &mut ctx.accounts.product;
    let bid_close_date = args.launch_date - (7 * 24 * 60 * 60);

    product.maker = ctx.accounts.maker.key();
    product.name = args.name;
    product.description = args.description;
    product.token_symbol = args.token_symbol;
    product.initial_deposit = args.initial_deposit;
    product.ipo_slots = args.ipo_slots;
    product.approved_bids = 0;
    product.total_token_supply = args.initial_token_supply;
    product.token_mint = ctx.accounts.token_mint.key();
    product.token_pool = ctx.accounts.token_pool.key();
    product.launch_date = args.launch_date;
    product.bid_close_date = bid_close_date;
    product.phase = ProductPhase::Bidding;
    product.created_at = now;
    product.funds_claimed = false;
    product.bump = ctx.bumps.product;

    invoke(
        &system_instruction::transfer(
            ctx.accounts.maker.key,
            ctx.accounts.treasury.key,
            args.initial_deposit,
        ),
        &[
            ctx.accounts.maker.to_account_info(),
            ctx.accounts.treasury.to_account_info(),
        ],
    )?;

    let binding = product.key();
    let pool_seeds = &[b"pool", binding.as_ref(), &[ctx.bumps.pool_authority]];
    let signer_seeds = &[&pool_seeds[..]];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.token_pool.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            signer_seeds,
        ),
        args.initial_token_supply,
    )?;

    Ok(())
}
