use anchor_lang::prelude::*;

use crate::Config;

pub fn lock(ctx:Context<Lock>)->Result<()>{

}


#[derive(Account)]
pub struct Lock<'info>{

    #[account]
    pub signer : Signer<'info>,

    #[account(
        mut,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info , Config>
}