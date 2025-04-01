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
    let instr = FundInstruction::unpack(data)?;
    match instr {
        FundInstruction::CreateFund { members } => {
            let fund_acc = next_account_info(&mut accounts.iter())?;
            // Add validation, init logic
            Ok(())
        },
        _ => Err(FundError::InvalidInstruction.into()),
    }
}
