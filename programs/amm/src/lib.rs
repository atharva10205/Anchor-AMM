pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

use instructions::*;
use state::*;

declare_id!("E9GKzL7A9YkDAjy7SavXcY8KF4emuAGMJW9vReLZxDVu");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(
        mut ctx: Context<Initialize>,
        seed: u64,
        fee: u16,
        authority: Option<Pubkey>,
    ) -> Result<()> {
        instructions::initialize::initialize(&mut ctx, seed, fee, authority)
    }
    pub fn deposit(mut ctx: Context<Deposit>, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        instructions::deposit::deposit(&mut ctx, amount, max_x, max_y)
    }
    pub fn swap(mut ctx: Context<Swap>, is_x: bool, amount: u64, min: u64) -> Result<()> {
        instructions::swap::swap(&mut ctx, is_x, amount, min)
    }
    pub fn withdraw(mut ctx: Context<Withdraw>, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        instructions::withdraw::withdraw(&mut ctx, amount, min_x, min_y)
    }
    pub fn lock(mut ctx: Context<Lock>) -> Result<()> {
        instructions::lock::lock(&mut ctx)
    }
    pub fn unlock(mut ctx: Context<Lock>) -> Result<()> {
        instructions::lock::lock(&mut ctx)
    }
}
