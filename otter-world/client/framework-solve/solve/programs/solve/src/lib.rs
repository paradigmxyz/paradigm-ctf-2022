use anchor_lang::prelude::*;

use anchor_spl::token::{Token, TokenAccount};

declare_id!("osecio1111111111111111111111111111111111111");

#[program]
pub mod solve {
    use super::*;

    pub fn get_flag(ctx: Context<GetFlag>) -> Result<()> {

        let cpi_accounts = chall::cpi::accounts::GetFlag {
            flag: ctx.accounts.flag.clone(),
            payer: ctx.accounts.payer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.chall.to_account_info(), cpi_accounts);

        chall::cpi::get_flag(cpi_ctx, 0x1337 /* TODO */)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct GetFlag<'info> {
    #[account(mut)]
    pub flag: AccountInfo<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub chall: Program<'info, chall::program::Chall>
}
