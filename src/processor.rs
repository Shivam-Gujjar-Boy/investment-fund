use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    program_error::ProgramError
};
use crate::{
    instruction::FundInstruction,
    state::{FundAccount, ProposalAccount},
    errors::FundError
};

pub fn process_instruction(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let instruction_data = FundInstruction::unpack(data)?;
    match instruction_data {
        FundInstruction::CreateFund { members, total_deposit , governance_mint} => {
            let fund_acc = next_account_info(&mut accounts.iter())?;
            // Add validation, init logic
            Ok(())
        },
        _ => Err(FundError::InvalidInstruction.into()),
    }
}
