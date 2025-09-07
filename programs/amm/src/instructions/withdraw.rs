use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Burn,burn, Mint, Token, TokenAccount, Transfer},
};
use constant_product_curve::ConstantProduct;

use crate::{AmmError, Config};

pub fn withdraw(ctx: Context<Withdraw>, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
    let ctx = ctx.accounts;

    require!(!ctx.config.locked, AmmError::PoolLocked);
    require!(amount != 0, AmmError::InvalidAmount);

    let amounts =ConstantProduct::xy_deposit_amounts_from_l(
    ctx.vault_x.amount,
    ctx.vault_y.amount,
    ctx.mint_lp.supply,
    amount,
    6
).map_err(AmmError::from)?;


    require!(
        amounts.x <= min_x && amounts.y <= min_y,
        AmmError::SlippageExceded
    );

    withdraw_tokens(ctx, true, amount);
    
    Ok(())
}

fn withdraw_tokens(ctx: Context<Withdraw>,is_x: bool, amount: u64)->Result<()>{

    let ctx = ctx.accounts;

    let (from , to) = if 
    is_x{(ctx.vault_x.to_account_info() , ctx.user_x.to_account_info())}
    else{(ctx.vault_y.to_account_info() , ctx.user_y.to_account_info())};

    let cpi_program = ctx.token_program.to_account_info();

    let cpi_account = Transfer{
        to,
        from,
        authority:ctx.signer.to_account_info()
    };

    let cpi_context = CpiContext::new(cpi_program, cpi_account);

    transfer(cpi_context , amount)
   
}

fn burn_token(ctx: Context<Withdraw>,amount: u64)->Result<()>{

    let ctx = ctx.accounts;

    let cpi_account = Burn {
        mint: ctx.mint_lp.to_account_info(),
        from: ctx.user_lp.to_account_info(),
        authority:ctx.signer.to_account_info()
    };

    let cpi_context = CpiContext::new(ctx.token_program.to_account_info(), cpi_account);

    burn(cpi_context, amount)
  
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.liquidity_pool_bump
    )]
    pub mint_lp: Account<'info, Mint>,

    #[account(
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        has_one = mint_x,
        has_one = mint_y
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub vault_x: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub vault_y: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = signer
    )]
    pub user_x: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = signer
    )]
    pub user_y: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = signer
    )]
    pub user_lp: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
