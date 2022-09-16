use chall;
use chall::cpi::create_pool;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use solana_program::program::invoke;

use anchor_spl::token::{Token, TokenAccount, Mint};

declare_id!("ze11ic1111111111111111111111111111111111111");

#[program]
pub mod solve {
    use super::*;

    pub fn dummy(_ctx: Context<Dummy>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Dummy<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub chall: Program<'info, chall::program::Challenge>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}
