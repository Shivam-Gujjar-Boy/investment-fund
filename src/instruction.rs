use solana_program::{
    program_error::ProgramError, pubkey::Pubkey
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
        privacy: u8,
        expected_members: u32,
        symbol_str: String,
        fund_name: String,
    },

    InitUserAccount { },

    InitJoinProposal {
        fund_name: String,
    },

    JoinVote {
        vote: u8,
        fund_name: String,
        vec_index: u8,
    },

    AddFundMember {
        fund_name: String,
        vec_index: u8,
    },

    // 1. Governance Mint Account
    // 2. Vault Account
    // 3. Fund Account
    // 4. System Program
    // 5. Token Program
    // 6. Member's Governance Token Account
    // 7. Member's Wallet
    // 8. User-specific PDA

    InitDepositToken {
        amount: u64,
        mint_amount: u64,
        fund_name: String,
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
        slippage: Vec<u16>,
        // dex_tags: Vec<u8>,
        deadline: i64,
        fund_name: String,
    },

    // 1. Voter Account
    // 2. Vote Account
    // 3. Proposal Account
    // 4. System Program
    // 5. User PDA Account
    // 6. Fund Account
    // 7. Governance Mint Account
    // 8. Voter Governance Token Account
    Vote {
        vote: u8,
        proposal_index: u8,
        vec_index: u8,
        fund_name: String,
    },

    DeleteFund {},

    InitRentAccount { },

    // 1. Proposal Account
    // 2. Fund Account
    // 3. Vault Account
    // 4. System Program
    // 5. Token Program
    ExecuteProposalInvestment {
        swap_number: u8,
        fund_name: String,
        proposal_index: u8,
        vec_index: u8
    },
    Execute { proposal: Pubkey },
    LeaveFund{fund_name: String },
}

impl FundInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input
            .split_first()
            .ok_or(FundError::InstructionUnpackError)?;

        Ok(match tag {
            0 => {
                let (privacy, rest) = Self::unpack_members(rest)?;
                let (expected_members, rest) = Self::unpack_expected(rest)?;
                let (symbol_bytes, rest) = rest.split_at(5 as usize);
                let symbol = symbol_bytes
                    .iter()
                    .take_while(|&&b| b != 0)
                    .cloned()
                    .collect::<Vec<u8>>();
                let symbol_str = String::from_utf8(symbol).unwrap();
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();
                Self::InitFundAccount {
                    privacy,
                    expected_members,
                    symbol_str,
                    fund_name,
                }
            }
            1 => {
                let (&num_of_swaps, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (amounts, rest) = Self::unpack_amounts(rest, num_of_swaps)?;
                let (slippage, rest) = Self::unpack_slippage(rest, num_of_swaps)?;
                let (deadline, rest) = Self::unpack_deadline(rest)?;
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();

                Self::InitProposalInvestment {
                    amounts,
                    slippage,
                    deadline,
                    fund_name,
                }
            }
            2 => {
                let (&vote, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (&proposal_index, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (&vec_index, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();
                Self::Vote {
                    vote,
                    proposal_index,
                    vec_index,
                    fund_name,
                }
            }
            3 => {
                let (&vec_index, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();
                Self::AddFundMember { fund_name, vec_index }
            }
            4 => {
                let (&swap_number, rest) = rest
                    .split_first()
                    .ok_or(FundError::InstructionUnpackError)?;
                let (&proposal_index, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (&vec_index, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();

                Self::ExecuteProposalInvestment {
                    swap_number,
                    fund_name,
                    proposal_index,
                    vec_index
                }
            }
            5 => {
                Self::InitRentAccount {  }
            }
            6 => {
                Self::InitUserAccount {  }
            }
            7 => {
                let (amount, rest) = Self::unpack_amount(rest)?;
                let (mint_amount, rest) = Self::unpack_amount(rest)?;
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();
                Self::InitDepositToken {
                    amount,
                    mint_amount,
                    fund_name,
                }
            }
            8 => {
                Self::DeleteFund { }
            }
            9 => {
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();

                Self::LeaveFund { fund_name }
            }
            10 => {
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();
                Self::InitJoinProposal { fund_name }
            }
            11 => {
                let (&vote, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (&vec_index, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();
                Self::JoinVote {
                    vote,
                    fund_name,
                    vec_index
                }
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

    fn unpack_expected(input: &[u8]) -> Result<(u32, &[u8]), ProgramError> {
        if input.len() < (4 as usize) {
            return Err(FundError::InstructionUnpackError.into());
        }

        let (expected_bytes, rest) = input.split_at(4 as usize);
        let expected_members = u32::from_le_bytes(expected_bytes.try_into().expect("Invalid members length"));

        Ok((expected_members, rest))
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

    fn unpack_slippage(input: &[u8], num_of_swaps: u8) -> Result<(Vec<u16>, &[u8]), ProgramError> {
        if input.len() < 2*(num_of_swaps as usize) {
            return Err(FundError::InstructionUnpackError.into());
        }

        let mut slippages: Vec<u16> = Vec::new();
        let mut input_slice = input;
        for _i in 0..num_of_swaps {
            let (slippage, rest) = Self::unpack_two_bytes(input_slice)?;
            slippages.push(slippage);
            input_slice = rest;
        }

        Ok((slippages, input_slice))
    }

    fn unpack_two_bytes(input: &[u8]) -> Result<(u16, &[u8]), ProgramError> {
        if input.len() < (2 as usize) {
            return Err(FundError::InstructionUnpackError.into());
        }

        let (slippage_bytes, rest) = input.split_at(2 as usize);
        let slippage = u16::from_le_bytes(slippage_bytes.try_into().expect("Invalid amount length"));

        Ok((slippage, rest))
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
