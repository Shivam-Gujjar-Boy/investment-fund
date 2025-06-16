use solana_program::program_error::ProgramError;

#[derive(Debug)]
pub enum FundError {
    InstructionUnpackError,
    MissingRequiredSignature,
    InvalidAccountData,
    InvalidGovernanceMint,
    AlreadyVoted,
    VotingCeased,
    InvalidTokenAccount,
    InvalidInstruction,
    NotEnoughFunds,
    InvalidFundDetails,
    InvalidMints,
    InvalidRentAccount,
    InvaildVaultAccount,
    InvalidProposerInfo,
    InvalidProposalAccount,
    InvalidVoteAccount,
    NotEnoughVotes,
    AlreadyExecuted,
    InvalidFundCreator,
    AlreadyAppliedForEntry,
    AlreadyMember,
}

impl From<FundError> for ProgramError {
    fn from(e: FundError) -> Self { ProgramError::Custom(e as u32) }
}