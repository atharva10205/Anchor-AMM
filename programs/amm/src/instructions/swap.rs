use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use constant_product_curve::{ConstantProduct, LiquidityPair};

use crate::{error::AmmError, Config};

pub fn swap(ctx: &mut Context<Swap>, is_x: bool, amount: u64, min: u64) -> Result<()> {
    let ctx_account = &ctx.accounts;

    require!(!ctx_account.config.locked, AmmError::PoolLocked);
    require!(amount != 0, AmmError::InvalidAmount);

    let mut curve = ConstantProduct::init(
        ctx_account.vault_x.amount,
        ctx_account.vault_y.amount,
        ctx_account.mint_lp.supply,
        ctx_account.config.fee,
        None,
    )
    .map_err(AmmError::from)?;

    let pair = if is_x {
        LiquidityPair::X
    } else {
        LiquidityPair::Y
    };
    let response = curve.swap(pair, amount, min).map_err(AmmError::from)?;

    require!(
        response.deposit != 0 && response.withdraw != 0,
        AmmError::InvalidAmount
    );
    deposit(ctx, response.deposit, is_x)?;
    withdraw(ctx, response.withdraw, is_x)?;

    Ok(())
}

fn deposit(ctx: &mut Context<Swap>, amount: u64, is_x: bool) -> Result<()> {
    let ctx = &ctx.accounts;

    let (from, to) = if is_x {
        (ctx.user_x.to_account_info(), ctx.vault_x.to_account_info())
    } else {
        (ctx.user_y.to_account_info(), ctx.vault_y.to_account_info())
    };

    let cpi_program = ctx.token_program.to_account_info();
    let cpi_accounts = Transfer {
        from,
        to,
        authority: ctx.signer.to_account_info(),
    };

    let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

    transfer(cpi_context, amount)
}

fn withdraw(ctx: &mut Context<Swap>, amount: u64, is_x: bool) -> Result<()> {
    let ctx = &ctx.accounts;

    let (from, to) = if is_x {
        (ctx.vault_y.to_account_info(), ctx.user_y.to_account_info())
    } else {
        (ctx.vault_x.to_account_info(), ctx.user_x.to_account_info())
    };

    let cpi_program = ctx.token_program.to_account_info();
    let cpi_account = Transfer {
        to,
        from,
        authority: ctx.config.to_account_info(),
    };

    let seeds = &[
        &b"config"[..],
        &ctx.config.seed.to_le_bytes(),
        &[ctx.config.config_bump],
    ];

    let signer_seeds = &[&seeds[..]];

    let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_account, signer_seeds);

    transfer(cpi_context, amount)
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.liquidity_pool_bump,
    )]
    pub mint_lp: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        has_one = mint_x,
        has_one = mint_y,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = signer,
    )]
    pub user_x: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = signer,
    )]
    pub user_y: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
