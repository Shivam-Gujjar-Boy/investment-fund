use solana_program::{
    program_error::ProgramError
};
use crate::errors::FundError;
use borsh::{BorshSerialize, BorshDeserialize};

const BYTE_SIZE_8: usize = 8;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum FundInstruction {
    // tag = 0
    InitFundAccount { 
        privacy: u8,
        expected_members: u32,
        symbol_str: String,
        fund_name: String,
    },

    // tag = 1
    InitProposalInvestment {
        amounts: Vec<u64>,
        slippage: Vec<u16>,
        deadline: i64,
        fund_name: String,
    },
    
    // tag = 2
    Vote {
        vote: u8,
        proposal_index: u8,
        vec_index: u16,
        fund_name: String,
    },
    
    // tag = 3
    AddFundMember {
        fund_name: String,
        vec_index: u8,
    },

    // tag = 4
    ExecuteProposalInvestment {
        swap_number: u8,
        fund_name: String,
        proposal_index: u8,
        vec_index: u8
    },

    // tag = 5
    InitRentAccount { },

    // tag = 6
    InitUserAccount { 
        cid: String,
    },

    // tag = 7
    InitDepositToken {
        is_unwrapped_sol: u8,
        amount: u64,
        mint_amount: u64,
        fund_name: String,
        fund_type: u8,
    },

    // tag = 8
    DeleteFund {},

    // tag = 9
    LeaveFund{fund_name: String },

    // tag = 10
    InitJoinProposal {
        fund_name: String,
    },

    // tag = 11
    JoinVote {
        vote: u8,
        fund_name: String,
        proposal_index: u8,
    },

    // tag = 12
    CancelJoinProposal {
        fund_name: String,
        proposal_index: u8,
    },

    // tag = 13
    CancelInvestmentProposal {
        fund_name: String,
        proposal_index: u8,
        vec_index: u16,
    },

    // tag = 14
    InitIncrementProposal {
        fund_name: String,
        new_size: u32,
        refund_type: u8,
    },

    // tag = 15
    VoteOnIncrement {
        fund_name: String,
        vote: u8,
    },

    // tag = 16
    CancelIncrementProposal {
        fund_name: String,
    },

    // tag = 17
    ToggleRefundType {
        fund_name: String,
        refund_type: u8,
    },

    // tag = 18
    InitLightFundAccount {
        fund_name: String,
        num_of_members: u8,
        max_num_members: u8,
        tags: u32,
        add_members_later: u8,
        fund_type: u8,
        is_eligible: u8,
    },

    // tag = 19
    HandleInvition {
        fund_name: String,
        response: u8,
    },

    // tag = 20
    InviteToFund {
        fund_name: String,
        fund_type: u8,
        is_eligible: u8,
    },

    // tag = 21
    WithdrawOrLeaveFromLightFund {
        fund_name: String,
        task: u8,
        stake_percent: u64,
        num_of_tokens: u8,
    }

}

impl FundInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input
            .split_first()
            .ok_or(FundError::InvaildVaultAccount)?;

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
                let (vec_index_bytes, rest) = rest.split_at(2 as usize);
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();
                let vec_index = u16::from_le_bytes(vec_index_bytes.try_into().expect("Wrong Vec Index"));
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
                let (cid_bytes, _rest) = rest.split_at(59 as usize);
                let cid_raw = cid_bytes
                    .iter()
                    .take_while(|&&b| b != 0)
                    .cloned()
                    .collect::<Vec<u8>>();
                let cid = String::from_utf8(cid_raw).unwrap();
                Self::InitUserAccount { cid }
            }
            7 => {
                let (&is_unwrapped_sol, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (&fund_type, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (amount, rest) = Self::unpack_amount(rest)?;
                let (mint_amount, rest) = Self::unpack_amount(rest)?;
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();
                Self::InitDepositToken {
                    is_unwrapped_sol,
                    amount,
                    mint_amount,
                    fund_name,
                    fund_type
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
                let (&proposal_index, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();
                Self::JoinVote {
                    vote,
                    fund_name,
                    proposal_index
                }
            }
            12 => {
                let (&proposal_index, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();
                Self::CancelJoinProposal { fund_name, proposal_index }
            }
            13 => {
                let (&proposal_index, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (vec_index_bytes, rest) = rest.split_at(2 as usize);
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();
                let vec_index = u16::from_le_bytes(vec_index_bytes.try_into().expect("Invalid Vec Index"));

                Self::CancelInvestmentProposal { fund_name, proposal_index, vec_index }
            }
            14 => {
                let (&refund_type, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (new_size_bytes, rest) = rest.split_at(4 as usize);
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();
                let new_size = u32::from_le_bytes(new_size_bytes.try_into().expect("Invalid New Size"));

                Self::InitIncrementProposal { fund_name, new_size, refund_type }
            }
            15 => {
                let (&vote, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();

                Self::VoteOnIncrement { fund_name, vote }
            }
            16 => {
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();

                Self::CancelIncrementProposal { fund_name }
            }
            17 => {
                let (&refund_type, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();

                Self::ToggleRefundType { fund_name, refund_type }
            }
            18 => {
                let (&fund_type, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (&is_eligible, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (&add_members_later, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (&num_of_members, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (&max_num_members, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (tag_bytes, rest) = rest.split_at(4 as usize);
                let fund_name = std::str::from_utf8(rest).map_err(|_| FundError::InstructionUnpackError)?.to_string();
                let tags = u32::from_be_bytes(tag_bytes.try_into().expect("Invalid tags"));

                Self::InitLightFundAccount { fund_name, num_of_members, max_num_members, tags, add_members_later, fund_type, is_eligible }
            }
            19 => {
                let (&response, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let fund_name = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?.to_string();

                Self::HandleInvition { fund_name, response }
            }
            20 => {
                let (&fund_type, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (&is_eligible, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let fund_name = std::str::from_utf8(rest).map_err(|_| FundError::InstructionUnpackError)?.to_string();

                Self::InviteToFund { fund_name, fund_type, is_eligible }
            }
            21 => {
                let (&num_of_tokens, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (&task, rest) = rest.split_first().ok_or(FundError::InstructionUnpackError)?;
                let (percent_bytes, rest) = rest.split_at(8);
                let stake_percent = u64::from_le_bytes(percent_bytes.try_into().expect("Wrong Stake Percent"));
                let fund_name = std::str::from_utf8(rest).map_err(|_| FundError::InstructionUnpackError)?.to_string();

                Self::WithdrawOrLeaveFromLightFund { fund_name, task, stake_percent, num_of_tokens }
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
