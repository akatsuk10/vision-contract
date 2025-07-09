use anchor_lang::prelude::*;

#[account]
pub struct GlobalConfig {
    pub protocol_admin: Pubkey,
    pub bump: u8,
}

impl GlobalConfig {
    pub const INIT_SPACE: usize = 32 + 1;
}
