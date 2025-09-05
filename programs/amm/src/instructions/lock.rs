use anchor_lang::prelude::*;

use crate::{AmmError, Config};

pub fn lock(ctx:Context<Lock>)->Result<()>{

    let ctx = ctx.accounts;

    require!(
       Some(ctx.signer.key()) == ctx.config.authority,
       AmmError::InvalidAuthority
    );

    ctx.config.locked = true;
    Ok(())
}


pub fn unlock(ctx:Context<Lock>)->Result<()>{

    let ctx = ctx.accounts;

    require!(
       Some(ctx.signer.key()) == ctx.config.authority,
       AmmError::InvalidAuthority
    );

    ctx.config.locked = false;
    Ok(())
}


#[derive(Accounts)]
pub struct Lock<'info>{

    #[account(mut)]
    pub signer : Signer<'info>,

    #[account(
        mut,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info , Config>
}