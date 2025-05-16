use solana_program::{
    entrypoint,
    // entrypoint::ProgramResult,
    // pubkey::Pubkey,
    // account_info::AccountInfo
};
use processor::process_instruction;

pub mod instruction;
pub mod processor;
pub mod state;
pub mod errors;
pub mod utils;

entrypoint!(process_instruction);
