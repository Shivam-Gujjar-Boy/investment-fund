use std::vec;
use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};
use crate::errors::FundError;
use borsh::{BorshSerialize, BorshDeserialize};

const BYTE_SIZE_8: usize = 8;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum FundInstruction {

    // 1. Governance Mint Account
    // 2. Vault Account
    // 3. Fund Account
    // 4. System Program
    // 5. Token Program
    // 6. Rent Account
    // 7. [..] Array of Fund Members
    InitFundAccount { 
        number_of_members: u8,
    },

    // 1. Governance Mint Account
    // 2. Vault Account
    // 3. Fund Account
    // 4. System Program
    // 5. Token Program
    // 6. Member's Governance Token Account
    // 7. Member's Wallet
    // 8. User-specific PDA
    InitDepositSol {
        amount: u64
    },

    // Proposals can be of the following types:
    // 1. Investment -> tag 0
    // 2. Addition of New Member -> tag 1
    // 3. Removal of any member -> tag 2
    // 4. Withdrawl -> tag 3

    // 1. Proposer Account
    // 2. [..] From Assets Mints
    // 3. [..] To Assets Mints
    InitProposalInvestment {
        amounts: Vec<u64>,
        dex_tags: Vec<u8>,
        deadline: i64,
    },

    Vote {
        proposal: Pubkey,
        yes: bool,
        amount: u64
    },

    ExecuteProposalInvestment {},
    Execute { proposal: Pubkey },
}

impl FundInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input
            .split_first()
            .ok_or(FundError::InstructionUnpackError)?;

        Ok(match tag {
            0 => {
                let (num, _rest) = Self::unpack_members(rest)?;

                Self::InitFundAccount {
                    number_of_members: num,
                }
            }
            1 => {
                let (amount, _rest) = Self::unpack_amount(rest)?;

                Self::InitDepositSol {
                    amount,
                }
            }
            2 => {
                let (&num_of_swaps, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (amounts, rest) = Self::unpack_amounts(rest, num_of_swaps)?;
                let (dex_tags, rest) = Self::unpack_dex_tags(rest, num_of_swaps)?;
                let (deadline, _rest) = Self::unpack_deadline(rest)?;

                Self::InitProposalInvestment {
                    amounts,
                    dex_tags,
                    deadline,
                }
            }
            3 => {

            }
            4 => {
                Self::ExecuteProposalInvestment {}
            }
            _ => {
                return Err(FundError::InstructionUnpackError.into());
            }
        })

    }

    fn unpack_members(input: &[u8]) -> Result<(u8, &[u8]), ProgramError> {
        let (&num, rest) = input
            .split_first()
            .ok_or(FundError::InstructionUnpackError)?;

        Ok((num, rest))
    }

    fn unpack_amount(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        if input.len() < BYTE_SIZE_8 {
            return Err(FundError::InstructionUnpackError.into());
        }
        let (amount_bytes, rest) = input.split_at(BYTE_SIZE_8);

        let amount = u64::from_le_bytes(amount_bytes.try_into().expect("Invalid amount length"));

        Ok((amount, rest))
    }

    fn unpack_amounts(input: &[u8], num_of_swaps: u8) -> Result<(Vec<u64>, &[u8]), ProgramError> {
        if input.len() < BYTE_SIZE_8*(num_of_swaps as usize) {
            return Err(FundError::InstructionUnpackError.into());
        }

        let mut amounts: Vec<u64> = Vec::new();
        let mut input_slice = input;
        for _i in 0..num_of_swaps {
            let (amount, rest) = Self::unpack_amount(input_slice)?;
            amounts.push(amount);
            input_slice = rest;
        }

        Ok((amounts, input_slice))
    }

    fn unpack_dex_tags(input: &[u8], num_of_swaps: u8) -> Result<(Vec<u8>, &[u8]), ProgramError> {
        if input.len() < num_of_swaps as usize {
            return Err(FundError::InstructionUnpackError.into());
        }

        let mut dex_tags: Vec<u8> = Vec::new();
        let mut input_slice = input;
        for _i in 0..num_of_swaps {
            let (dex_tag, rest) = input_slice.split_first().ok_or(FundError::InstructionUnpackError)?;
            dex_tags.push(*dex_tag);
            input_slice = rest;
        }

        Ok((dex_tags, input_slice))
    }

    fn unpack_deadline(input: &[u8]) -> Result<(i64, &[u8]), ProgramError> {
        if input.len() < BYTE_SIZE_8 {
            return Err(FundError::InstructionUnpackError.into());
        }

        let (deadline_data, rest) = input.split_at(BYTE_SIZE_8);
        let deadline = i64::from_le_bytes(deadline_data.try_into().expect("Failed to get Deadline"));
        Ok((deadline, rest))
    }
}
