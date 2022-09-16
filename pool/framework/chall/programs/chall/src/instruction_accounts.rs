use anchor_lang::prelude::*;
use anchor_lang::Accounts;
use anchor_spl::token::{Burn, Mint, Token, TokenAccount, Transfer, MintTo};

use crate::pool::{WithdrawalQueueHeader, WithdrawalQueueNode, Pool};

#[account]
#[derive(Default)]
pub struct Config {
    pub next_free_pool_seed: u64,
    pub config_bump: u8,
}

pub const CONFIG_SEED: &[u8] = b"CONFIG_SEED";
pub const POOL_SEED: &[u8] = b"POOL_SEED";
pub const POOL_REDEEM_MINT_SEED: &[u8] = b"POOL_REDEEM_TOKENS_MINT_SEED";
pub const POOL_TOKEN_ACCOUNT_SEED: &[u8] = b"POOL_TOKEN_ACCOUNT_SEED";
pub const POOL_QUEUE_HEADER_SEED: &[u8] = b"POOL_QUEUE_HEADER_SEED";
pub const POOL_QUEUE_NODE_SEED: &[u8] = b"POOL_QUEUE_NODE_SEED";

#[derive(Accounts)]
pub struct InitializeInstructionAccounts<'info> {
    #[account(
        init,
        seeds = [CONFIG_SEED],
        bump,
        payer = signer,
        space = std::mem::size_of::<Config>() + 8,
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreatePoolInstructionAccounts<'info> {
    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        seeds = [
            POOL_SEED,
            config.next_free_pool_seed.to_le_bytes().as_ref()
        ],
        bump,
        payer = signer,
        space = std::mem::size_of::<Pool>() + 8,
    )]
    pub pool: Account<'info, Pool>,
    // The mint that emits tokens that can be redeemed by users to get their assets (+ interest) back
    #[account(
        init,
        seeds = [POOL_REDEEM_MINT_SEED, pool.key().as_ref()],
        bump,
        payer = signer,
        mint::authority = pool,
        mint::decimals = token_mint.decimals,
    )]
    pub pool_redeem_tokens_mint: Account<'info, Mint>,

    // The token mint
    pub token_mint: Account<'info, Mint>,

    // The pool's token account
    #[account(
        init,
        seeds = [POOL_TOKEN_ACCOUNT_SEED, pool.key().as_ref()],
        bump,
        payer = signer,
        token::authority = pool,
        token::mint = token_mint,
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,
    // The pool's withdrawal queue header
    #[account(
        init,
        seeds = [POOL_QUEUE_HEADER_SEED, pool.key().as_ref()],
        bump,
        payer = signer,
        space = std::mem::size_of::<WithdrawalQueueHeader>() + 8,
    )]
    pub withdrawal_queue_header: Account<'info, WithdrawalQueueHeader>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositInstructionAccounts<'info> {
    // The pool where the deposit is made
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            pool.seed.to_le_bytes().as_ref()
        ],
        bump,
        has_one = token_mint,
    )]
    pub pool: Account<'info, Pool>,
    // The mint that emits tokens that can be redeemed by users to get their assets (+ interest) back
    #[account(
        mut,
        seeds = [POOL_REDEEM_MINT_SEED, pool.key().as_ref()],
        bump = pool.redeem_tokens_mint_bump,
        mint::authority = pool,
        mint::decimals = token_mint.decimals,
    )]
    pub pool_redeem_tokens_mint: Account<'info, Mint>,

    // The token mint
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    // The pool's token account
    #[account(
        mut,
        seeds = [POOL_TOKEN_ACCOUNT_SEED, pool.key().as_ref()],
        bump = pool.token_account_bump,
        token::authority = pool,
        token::mint = token_mint,
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    // The user depositing tokens, must be a signer of the transaction
    #[account(mut)]
    pub user: Signer<'info>,

    // The account where the deposited tokens come from
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    // The account where the redeem tokens are emitted to
    #[account(
        mut,
        associated_token::mint = pool_redeem_tokens_mint,
        associated_token::authority = user,
    )]
    pub user_redeem_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> DepositInstructionAccounts<'info> {
    pub fn deposit_token_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.user_token_account.to_account_info(),
            to: self.pool_token_account.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let token_program = self.token_program.to_account_info();
        CpiContext::new(token_program, cpi_accounts)
    }

    pub fn deposit_token_mint_context<'a, 'b, 'c>(&self, signer_seeds: &'a[&'b[&'c[u8]]]) -> CpiContext<'a, 'b, 'c, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            authority: self.pool.to_account_info(),
            to: self.user_redeem_token_account.to_account_info(),
            mint: self.pool_redeem_tokens_mint.to_account_info(),
        };
        let token_program = self.token_program.to_account_info();
        CpiContext::new_with_signer(
            token_program,
            cpi_accounts,
            signer_seeds,
        )
    }
}

#[derive(Accounts)]
pub struct RequestWithdrawInstructionAccounts<'info> {
    // The pool where the deposit is made
    #[account(
        mut,
        seeds = [POOL_SEED, pool.seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,
    // The mint that emits tokens that can be redeemed by users to get their assets (+ interest) back
    #[account(
        mut,
        seeds = [
            POOL_REDEEM_MINT_SEED,
            pool.key().as_ref(),
        ],
        bump = pool.redeem_tokens_mint_bump,
        mint::authority = pool,
    )]
    pub pool_redeem_tokens_mint: Account<'info, Mint>,

    // The pool's withdrawal queue header
    #[account(
        mut,
        seeds = [POOL_QUEUE_HEADER_SEED, pool.key().as_ref()],
        bump = pool.withdrawal_queue_header_bump,
    )]
    pub withdrawal_queue_header: Account<'info, WithdrawalQueueHeader>,
    // The withdrawal queue node representing the user deposit
    #[account(
        init,
        seeds = [
            POOL_QUEUE_NODE_SEED,
            withdrawal_queue_header.key().as_ref(),
            withdrawal_queue_header.nonce.to_le_bytes().as_ref()
        ],
        bump,
        payer = user,
        space = std::mem::size_of::<WithdrawalQueueNode>() + 8,
    )]
    pub withdrawal_queue_node: Account<'info, WithdrawalQueueNode>,

    // The user, must be a signer of the transaction
    #[account(mut)]
    pub user: Signer<'info>,
    // The account where the redeem tokens are emitted to
    #[account(
        mut,
        associated_token::mint = pool_redeem_tokens_mint,
        associated_token::authority = user
    )]
    pub user_redeem_token_account: Account<'info, TokenAccount>,

    // SPL-token program
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> RequestWithdrawInstructionAccounts<'info> {
    pub fn redeem_token_burn_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        let cpi_accounts = Burn {
            mint: self.pool_redeem_tokens_mint.to_account_info(),
            from: self.user_redeem_token_account.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let token_program = self.token_program.to_account_info();
        return CpiContext::new(
            token_program,
            cpi_accounts,
        );
    }
}

#[derive(Accounts)]
pub struct CancelWithdrawRequestInstructionAccounts<'info> {
    // The pool where the deposit is made
    #[account(
        mut,
        seeds = [POOL_SEED, pool.seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,
    // The mint that emits tokens that can be redeemed by users to get their assets (+ interest) back
    #[account(
        mut,
        seeds = [
            POOL_REDEEM_MINT_SEED,
            pool.key().as_ref(),
        ],
        bump = pool.redeem_tokens_mint_bump,
        mint::authority = pool,
    )]
    pub pool_redeem_tokens_mint: Account<'info, Mint>,

    // The pool's withdrawal queue header
    #[account(
        mut,
        seeds = [POOL_QUEUE_HEADER_SEED, pool.key().as_ref()],
        bump = pool.withdrawal_queue_header_bump,
    )]
    pub withdrawal_queue_header: Account<'info, WithdrawalQueueHeader>,
    // The withdrawal queue node representing the user deposit
    #[account(
        mut,
        has_one=user,
        seeds = [
            POOL_QUEUE_NODE_SEED,
            withdrawal_queue_header.key().as_ref(),
            withdrawal_queue_node.nonce.to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub withdrawal_queue_node: Account<'info, WithdrawalQueueNode>,

    // The user, must be a signer of the transaction
    pub user: Signer<'info>,
    // The account where the redeem tokens are emitted to
    #[account(
        mut,
        associated_token::mint = pool_redeem_tokens_mint,
        associated_token::authority = user
    )]
    pub user_redeem_token_account: Account<'info, TokenAccount>,

    // SPL-token program
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CancelWithdrawRequestInstructionAccounts<'info> {
    pub fn redeem_token_mint_context<'a, 'b, 'c>(&self, signer_seeds: &'a[&'b[&'c[u8]]]) -> CpiContext<'a, 'b, 'c, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            authority: self.pool.to_account_info(),
            to: self.user_redeem_token_account.to_account_info(),
            mint: self.pool_redeem_tokens_mint.to_account_info(),
        };
        let token_program = self.token_program.to_account_info();
        CpiContext::new_with_signer(
            token_program,
            cpi_accounts,
            signer_seeds,
        )
    }
}

#[derive(Accounts)]
pub struct ProcessWithdrawQueueInstructionAccounts<'info> {
    // The pool where the deposit is made
    #[account(
        mut,
        seeds = [POOL_SEED, pool.seed.to_le_bytes().as_ref()],
        bump,
        has_one = token_mint,
    )]
    pub pool: Account<'info, Pool>,

    // The token mint
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    // The pool's token account
    #[account(
        mut,
        seeds = [POOL_TOKEN_ACCOUNT_SEED, pool.key().as_ref()],
        bump = pool.token_account_bump,
        token::authority = pool,
        token::mint = token_mint,
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    // The pool's withdrawal queue header
    #[account(
        mut,
        seeds = [POOL_QUEUE_HEADER_SEED, pool.key().as_ref()],
        bump = pool.withdrawal_queue_header_bump,
    )]
    pub withdrawal_queue_header: Account<'info, WithdrawalQueueHeader>,

    // SPL-token program
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
