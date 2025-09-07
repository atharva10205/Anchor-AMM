use anchor_lang::prelude::CpiContext;
use anchor_lang::prelude::*;
use anchor_lang::require;
use anchor_spl::token::{Transfer, Mint, MintTo, Token, TokenAccount, transfer, mint_to};
use anchor_spl::{
    associated_token::AssociatedToken
};

use crate::{AmmError, Config};
use constant_product_curve::ConstantProduct;

pub fn deposit(ctx: Context<Deposit>, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
    let ctx_account= &ctx.accounts;

    require!(!ctx_account.config.locked, AmmError::PoolLocked);
    require!(amount != 0, AmmError::InvalidAmount);

    let (x, y) = match ctx_account.mint_liquidity_pool.supply == 0
        && ctx_account.vault_x.amount == 0
        && ctx_account.vault_y.amount == 0
    {
        true => (max_x, max_y),
        false => {
            let amounts = ConstantProduct::xy_deposit_amounts_from_l(
                ctx_account.vault_x.amount,
                ctx_account.vault_y.amount,
                ctx_account.mint_liquidity_pool.supply,
                amount,
                6,
            )
            .unwrap();
            (amounts.x, amounts.y)
        }
    };
        require!(x <= max_x && y <= max_y, AmmError::SlippageExceded);

        deposit_token(&ctx, true, x)?;
        deposit_token(&ctx, false, y)?;

        mint_lp_tokens(ctx , amount)


}

fn deposit_token(ctx: &Context<Deposit>, is_x: bool, amount: u64) -> Result<()> {
    let ctx= &ctx.accounts;

    let(from,to) = match  
    is_x
     {
        true => (
            ctx.user_x.to_account_info(),
            ctx.vault_x.to_account_info()
        ),
        false=>(
            ctx.user_y.to_account_info(),
            ctx.vault_y.to_account_info()
        )
    };

    let cpi_program = ctx.token_program.to_account_info();


    let cpi_account = Transfer{
        from,
        to,
        authority : ctx.signer.to_account_info(),

    };

    let cpi_context = CpiContext::new(cpi_program , cpi_account);

    transfer(cpi_context, amount)
}

fn mint_lp_tokens(ctx:Context<Deposit>,amount:u64)->Result<()>{

    let ctx_account = &ctx.accounts;

    let cpi_program = ctx_account.token_program.to_account_info();

    let cpi_account = MintTo{
        mint: ctx_account.mint_liquidity_pool.to_account_info(),
        to:ctx_account.user_liquidity_pool.to_account_info(),
        authority:ctx_account.config.to_account_info()

    };

     let seeds = &[
          b"config".as_ref(),
          &ctx_account.config.seed.to_le_bytes(),
          &[ctx_account.config.config_bump]
     ];

      let signer_seeds = &[&seeds[..]];

      let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, signer_seeds);

      mint_to(cpi_ctx,amount)

}



#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        has_one = mint_x,
        has_one = mint_y,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.liquidity_pool_bump,
    )]
    pub mint_liquidity_pool: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Account<'info, TokenAccount>, //total x token in lp

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Account<'info, TokenAccount>,//total y token in lp

    #[account(
        associated_token::mint = mint_x,
        associated_token::authority = signer,
    )]
    pub user_x: Account<'info, TokenAccount>,

    #[account(
        associated_token::mint = mint_y,
        associated_token::authority = signer,
    )]
    pub user_y: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_liquidity_pool,
        associated_token::authority = signer,
    )]
    pub user_liquidity_pool: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
