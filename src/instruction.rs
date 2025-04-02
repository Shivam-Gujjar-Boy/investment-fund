use solana_program::{
    program_error::ProgramError,
    pubkey::{Pubkey, PUBKEY_BYTES},
    msg
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
    InitDepositSol {
        amount: u64
    },
    Propose { asset: Pubkey, amount: u64, dex: Pubkey, deadline: i64 },
    Vote { proposal: Pubkey, yes: bool, amount: u64 },
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
            _ => {
                msg!("Instruction cannot be unpacked");
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

    // fn unpack_pubkeys(input: &[u8]) -> Result<(Vec<Pubkey>, &[u8]), ProgramError> {
    //     let (&num_members, rest) = input
    //         .split_first()
    //         .ok_or(FundError::InstructionUnpackError)?;

    //     if input.len() < PUBKEY_BYTES*(num_members as usize) {
    //         msg!("Pubkeys cannot be unpacked");
    //         return Err(FundError::InstructionUnpackError.into());
    //     }

    //     let mut pubkey_vec : Vec<Pubkey> = Vec::new();
    //     let mut input_slice = input;

    //     for _i in 0..num_members {
    //         let (key, rest) = input_slice.split_at(PUBKEY_BYTES);
    //         let pubkey = Pubkey::new_from_array(key.try_into().expect("Invalid Pubkey Length"));
    //         pubkey_vec.push(pubkey);
    //         input_slice = rest;
    //     }

    //     Ok((pubkey_vec, rest))
    // }

    fn unpack_amount(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        if input.len() < BYTE_SIZE_8 {
            msg!("Amount cannot be unpacked");
            return Err(FundError::InstructionUnpackError.into());
        }
        let (amount_bytes, rest) = input.split_at(BYTE_SIZE_8);

        let amount = u64::from_le_bytes(amount_bytes.try_into().expect("Invalid amount length"));

        Ok((amount, rest))
    }

    // fn unpack_mint(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
    //     if input.len() < PUBKEY_BYTES {
    //         msg!("Governance Mint cannot be unpacked");
    //         return Err(FundError::InstructionUnpackError.into());
    //     }
    //     let (governance_key, rest) = input.split_at(PUBKEY_BYTES);

    //     let governance_mint = Pubkey::new_from_array(governance_key.try_into().expect("Invalid Pubkey Length"));
    //     Ok((governance_mint, rest))
    // }
}
