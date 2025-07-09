use anchor_lang::prelude::*;
use crate::state::global_config::*;

#[derive(Accounts)]
pub struct InitProtocol<'info> {
    #[account(
        init,
        seeds = [b"global-config"],
        bump,
        payer = signer,
        space = 8 + GlobalConfig::INIT_SPACE
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitProtocol>) -> Result<()> {
    let config = &mut ctx.accounts.global_config;
    config.protocol_admin = ctx.accounts.signer.key();
    config.bump = ctx.bumps.global_config;
    Ok(())
}