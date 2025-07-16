use solana_program::program_error::ProgramError;

#[derive(Debug)]
pub enum FundError {
    InvalidMemberInfo,
    NotInvited,
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
    NoVotingPower,
    NotAFundMember,
    DeadlineReached,
    FundAlreadyFull,
    InvalidNewSize,
    IncrementProposalExists,
    InvalidInviter,
    AlreadyInvited,
    InvalidNumberOfWithdrawals,
    InvalidStakePercent,
    InvalidSigner,
}

impl From<FundError> for ProgramError {
    fn from(e: FundError) -> Self { ProgramError::Custom(e as u32) }
}