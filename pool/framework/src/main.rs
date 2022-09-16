use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use chall;
use std::env;
use std::io::Write;

use chall::instruction_accounts::{CONFIG_SEED, POOL_SEED, POOL_REDEEM_MINT_SEED, POOL_TOKEN_ACCOUNT_SEED, POOL_QUEUE_HEADER_SEED, POOL_QUEUE_NODE_SEED};
use sol_ctf_framework::ChallengeBuilder;

use solana_sdk::compute_budget::ComputeBudgetInstruction;

use solana_program::instruction::Instruction;
use solana_program::system_instruction;
use solana_program_test::tokio;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::pubkey;
use solana_sdk::signature::Signer;
use solana_sdk::signer::keypair::Keypair;
use std::error::Error;

use std::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;

    println!("starting server at port 8080!");

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        tokio::spawn(async {
            if let Err(err) = handle_connection(stream).await {
                println!("error: {:?}", err);
            }
        });
    }
    Ok(())
}

async fn handle_connection(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut builder = ChallengeBuilder::try_from(socket.try_clone().unwrap()).unwrap();

    let chall_id = builder.add_program("./chall/target/deploy/chall.so", Some(chall::ID));
    let solve_id = builder.input_program()?;

    let valid_pubkey = pubkey!("ze11ic1111111111111111111111111111111111111");
    if solve_id != valid_pubkey {
        writeln!(socket, "bad pubkey, got: {} expected: {}", solve_id, valid_pubkey)?;
        return Ok(());
    }

    let mut chall = builder.build().await;

    let admin_keypair = &chall.ctx.payer;
    let admin = admin_keypair.pubkey().clone();

    // Initialize the challenge
    writeln!(socket, "Initializing the challenge")?;
    let config = Pubkey::find_program_address(&[CONFIG_SEED], &chall_id).0;

    let ix = chall::instruction::Initialize {};
    let ix_accounts = chall::accounts::InitializeInstructionAccounts {
        config,
        signer: admin,
        system_program: solana_program::system_program::ID,
    };
    chall.run_ix(Instruction::new_with_bytes(
        chall_id,
        &ix.data(),
        ix_accounts.to_account_metas(None),
    )).await?;

    // Create token mint
    writeln!(socket, "Creating token mint")?;
    let token_mint = chall.add_mint().await?;

    // Create pool
    writeln!(socket, "Creating pool")?;
    let pool = Pubkey::find_program_address(&[POOL_SEED, (0 as u64).to_le_bytes().as_ref()], &chall_id).0;
    let pool_redeem_tokens_mint = Pubkey::find_program_address(&[POOL_REDEEM_MINT_SEED, pool.as_ref()], &chall_id).0;
    let pool_token_account = Pubkey::find_program_address(&[POOL_TOKEN_ACCOUNT_SEED, pool.as_ref()], &chall_id).0;
    let withdrawal_queue_header = Pubkey::find_program_address(&[POOL_QUEUE_HEADER_SEED, pool.as_ref()], &chall_id).0;

    let ix = chall::instruction::CreatePool {};
    let ix_accounts = chall::accounts::CreatePoolInstructionAccounts {
        config,
        pool,
        pool_redeem_tokens_mint,
        token_mint,
        pool_token_account,
        withdrawal_queue_header,
        signer: admin,
        system_program: solana_program::system_program::ID,
        token_program: spl_token::ID,
        rent: solana_program::sysvar::rent::ID,
    };
    chall.run_ix(Instruction::new_with_bytes(
        chall_id,
        &ix.data(),
        ix_accounts.to_account_metas(None),
    )).await?;

    // Create and fund admin token accounts
    writeln!(socket, "Creating admin token account")?;
    let admin_token_account = chall.add_associated_token_account(&token_mint, &admin).await?;
    writeln!(socket, "Minting tokens to admin")?;
    chall.mint_to(1000, &token_mint, &admin_token_account).await?;

    // Create admin redeem token account
    writeln!(socket, "Creating admin redeem token account")?;
    let admin_redeem_token_account = chall.add_associated_token_account(&pool_redeem_tokens_mint, &admin).await?;

    // Deposit tokens into pool
    writeln!(socket, "Deposit tokens into pool")?;
    let ix = chall::instruction::Deposit {deposit_amount: 100};
    let ix_accounts = chall::accounts::DepositInstructionAccounts {
        pool,
        pool_redeem_tokens_mint,
        token_mint,
        pool_token_account,
        user: admin,
        user_token_account: admin_token_account,
        user_redeem_token_account: admin_redeem_token_account,
        system_program: solana_program::system_program::ID,
        token_program: spl_token::ID,
        rent: solana_program::sysvar::rent::ID,
    };
    chall.run_ix(Instruction::new_with_bytes(
        chall_id,
        &ix.data(),
        ix_accounts.to_account_metas(None),
    )).await?;

    // Request withdrawal
    let withdrawal_queue_node = Pubkey::find_program_address(&[POOL_QUEUE_NODE_SEED, withdrawal_queue_header.as_ref(), (0 as u64).to_le_bytes().as_ref()], &chall_id).0;
    let ix = chall::instruction::RequestWithdraw {withdraw_amount: 10};
    let ix_accounts = chall::accounts::RequestWithdrawInstructionAccounts {
        pool,
        pool_redeem_tokens_mint,
        withdrawal_queue_header,
        withdrawal_queue_node,
        user: admin,
        user_redeem_token_account: admin_redeem_token_account,
        system_program: solana_program::system_program::ID,
        token_program: spl_token::ID,
    };
    chall.run_ix(Instruction::new_with_bytes(
        chall_id,
        &ix.data(),
        ix_accounts.to_account_metas(None),
    )).await?;

    // -- Start user part
    let user_keypair = Keypair::new();
    let user = user_keypair.pubkey();
    let user_token_account = chall.add_associated_token_account(&token_mint, &user).await?;

    writeln!(socket, "Accounts:")?;
    writeln!(socket, "config: {}", config)?;
    writeln!(socket, "mint: {}", token_mint)?;
    writeln!(socket, "pool: {}", pool)?;
    writeln!(socket, "pool_redeem_tokens_mint: {}", pool_redeem_tokens_mint)?;
    writeln!(socket, "pool_token_account: {}", pool_token_account)?;
    writeln!(socket, "withdrawal_queue_header: {}", withdrawal_queue_header)?;
    writeln!(socket, "withdrawal_queue_node: {}", withdrawal_queue_node)?;
    writeln!(socket, "admin: {}", admin)?;
    writeln!(socket, "admin_token_account: {}", admin_token_account)?;
    writeln!(socket, "user: {}", user)?;
    // Send tokens here to win
    writeln!(socket, "user_token_account: {}", user_token_account)?;

    // Fund the user account
    chall.run_ix(system_instruction::transfer(&admin, &user, 100_000_000_000)).await?;

    // Run the user transaction
    let bump_budget = ComputeBudgetInstruction::set_compute_unit_limit(10_000_000u32);
    let solve_ix = chall.read_instruction(solve_id)?;

    chall
        .run_ixs_full(
            &[bump_budget, solve_ix],
            &[&user_keypair],
            &user_keypair.pubkey(),
        )
        .await?;

    // Check win condition
    let user_token_account = chall.read_token_account(user_token_account).await?;
    if user_token_account.amount > 0 {
        writeln!(socket, "congrats!")?;
        if let Ok(flag) = env::var("FLAG") {
            writeln!(socket, "flag: {:?}", flag)?;
        } else {
            writeln!(socket, "flag not found, please contact admin")?;
        }
    } else {
        writeln!(socket, "keep trying")?;
    }

    Ok(())
}
