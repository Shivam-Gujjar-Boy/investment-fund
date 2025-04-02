use solana_program::program_error::ProgramError;

#[derive(Debug)]
pub enum FundError {
    InstructionUnpackError,
    MissingRequiredSignature,
    InvalidAccountData,
    InvalidGovernanceMint,
    InvalidTokenAccount,
    InvalidInstruction,
    NotEnoughFunds,
}

impl From<FundError> for ProgramError {
    fn from(e: FundError) -> Self { ProgramError::Custom(e as u32) }
}