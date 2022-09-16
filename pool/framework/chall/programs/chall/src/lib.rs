extern crate core;

pub mod instruction_accounts;
mod pool;

use anchor_lang::prelude::*;
use anchor_spl;

use crate::instruction_accounts::*;
use crate::pool::WithdrawalQueueNode;
use spl_associated_token_account::get_associated_token_address;

declare_id!("7Ex3YxehDTJSYeZ3Bhp965F7pMfqDUyzUvg3V21BwxkV");

// TODO: properly use lifetimes and avoid passing signer seeds like a caveman
// TODO: close accounts when they're not needed anymore

#[program]
pub mod challenge {
    use super::*;

    pub fn initialize(ctx: Context<InitializeInstructionAccounts>) -> Result<()> {
        let ref mut config = ctx.accounts.config;
        config.next_free_pool_seed = 0;
        config.config_bump = ctx.bumps["config"];
        Ok(())
    }

    pub fn create_pool(ctx: Context<CreatePoolInstructionAccounts>) -> Result<()> {
        let ref mut pool = ctx.accounts.pool;
        pool.seed = ctx.accounts.config.next_free_pool_seed;
        ctx.accounts.config.next_free_pool_seed += 1;
        pool.token_mint = ctx.accounts.token_mint.key();
        pool.redeem_tokens_mint_bump = ctx.bumps["pool_redeem_tokens_mint"];
        pool.token_account_bump = ctx.bumps["pool_token_account"];
        pool.withdrawal_queue_header_bump = ctx.bumps["withdrawal_queue_header"];
        Ok(())
    }

    pub fn deposit(ctx: Context<DepositInstructionAccounts>, deposit_amount: u64) -> Result<()> {
        // Deposit from user to pool account
        anchor_spl::token::transfer(ctx.accounts.deposit_token_transfer_context(), deposit_amount)?;

        // Mint redeem tokens to the user
        let pool_nonce = ctx.accounts.pool.seed.to_le_bytes();
        let pool_bump = vec![ctx.bumps["pool"]];
        let pool_seeds = vec![
            instruction_accounts::POOL_SEED,
            pool_nonce.as_ref(),
            pool_bump.as_ref(),
        ];
        let signers_seeds: Vec<&[&[u8]]> = vec![
            pool_seeds.as_ref(),
        ];
        anchor_spl::token::mint_to(ctx.accounts.deposit_token_mint_context(signers_seeds.as_ref()), deposit_amount)?;
        Ok(())
    }

    pub fn request_withdraw(ctx: Context<RequestWithdrawInstructionAccounts>, withdraw_amount: u64) -> Result<()> {
        // TODO: the amount to withdraw should be tied to the balance of the pool token account for this to be more realistic
        // Burn redeem tokens
        anchor_spl::token::burn(ctx.accounts.redeem_token_burn_context(), withdraw_amount)?;

        // Record the request in the queue
        ctx.accounts.withdrawal_queue_node.amount = withdraw_amount;
        ctx.accounts.withdrawal_queue_node.user = ctx.accounts.user.key.clone();
        // TODO: should `push` be responsible for this?
        ctx.accounts.withdrawal_queue_node.nonce = ctx.accounts.withdrawal_queue_header.nonce;

        ctx.accounts.withdrawal_queue_header.push(
            &mut ctx.accounts.withdrawal_queue_node,
            ctx.remaining_accounts.get(0)
        )?;

        Ok(())
    }

    pub fn cancel_withdraw_request(ctx: Context<CancelWithdrawRequestInstructionAccounts>) -> Result<()> {
        let withdraw_amount = ctx.accounts.withdrawal_queue_node.amount;

        // Remove withdraw request from the queue
        ctx.accounts.withdrawal_queue_header.remove(
            &ctx.accounts.withdrawal_queue_node,
            ctx.remaining_accounts.get(0)
        )?;

        // Mint redeem tokens to the user again
        let pool_nonce = ctx.accounts.pool.seed.to_le_bytes();
        let pool_bump = vec![ctx.bumps["pool"]];
        let pool_seeds = vec![
            instruction_accounts::POOL_SEED,
            pool_nonce.as_ref(),
            pool_bump.as_ref(),
        ];
        let signers_seeds: Vec<&[&[u8]]> = vec![
            pool_seeds.as_ref(),
        ];
        anchor_spl::token::mint_to(ctx.accounts.redeem_token_mint_context(signers_seeds.as_ref()), withdraw_amount)?;

        Ok(())
    }

    pub fn process_withdraw_queue<'a, 'b, 'c, 'info>(ctx: Context<'a, 'b, 'c, 'info, ProcessWithdrawQueueInstructionAccounts<'info>>) -> Result<()> {
        assert!(ctx.remaining_accounts.len() % 2 == 0);

        for queue_node_and_user_token_account in ctx.remaining_accounts.chunks(2) {
            let mut queue_node: Account<WithdrawalQueueNode> = Account::try_from(&queue_node_and_user_token_account[0])?;
            let user_token_account_info = &queue_node_and_user_token_account[1];

            let expected_token_account = get_associated_token_address(&queue_node.user, &ctx.accounts.token_mint.key());
            assert_eq!(user_token_account_info.key(), expected_token_account);

            // Send the tokens to the user
            let pool_nonce = ctx.accounts.pool.seed.to_le_bytes();
            let pool_bump = vec![ctx.bumps["pool"]];
            let pool_seeds = vec![
                instruction_accounts::POOL_SEED,
                pool_nonce.as_ref(),
                pool_bump.as_ref(),
            ];
            let signers_seeds: Vec<&[&[u8]]> = vec![
                pool_seeds.as_ref(),
            ];
            let cpi_accounts = anchor_spl::token::Transfer {
                authority: ctx.accounts.pool.to_account_info(),
                to: user_token_account_info.clone(),
                from: ctx.accounts.pool_token_account.to_account_info(),
            };
            let token_program = ctx.accounts.token_program.to_account_info();
            let cpi_context = CpiContext::new_with_signer(
                token_program,
                cpi_accounts,
                &signers_seeds,
            );
            anchor_spl::token::transfer(cpi_context, queue_node.amount)?;

            // Remove entry from withdraw queue
            ctx.accounts.withdrawal_queue_header.pop(&mut queue_node)?;
        }

        Ok(())
    }
}
