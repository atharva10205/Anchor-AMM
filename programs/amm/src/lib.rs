use anchor_lang::prelude::*;

pub mod state;
pub use state::*;

pub mod instructions;
pub use instructions::*;

pub mod error;
pub use error::*;

declare_id!("E9GKzL7A9YkDAjy7SavXcY8KF4emuAGMJW9vReLZxDVu");

#[program]
pub mod amm {
    use super::*;

}
