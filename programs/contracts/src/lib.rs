use anchor_lang::prelude::*;

declare_id!("438bhr1JkfTdT4mV672sxiANy56Mw539wwythJjwUeJd");

#[program]
pub mod contracts {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
