mod instruction;
/** 
    THE AFBN RAFFLE SYSTEM CASINO
    AUTHOR: COAT
    CREATED: 10th of APRIL, 2024
    OWNER: AFBN inc.
**/
mod storage;

pub use crate::storage::Instructions;

/* IMPORTS */
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

//Defining entrypoints
entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    //creating an account iter ref
    let accounts_iter = &mut accounts.iter();
    let data_account = next_account_info(accounts_iter)?;
    //check if its owned by this contract
    if data_account.owner != program_id {
        msg!("Account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }
    //process instructions
    let inst_data = Instructions::try_from_slice(&instruction_data)?;
    return inst_data.start(&accounts, &program_id);
}
