use solana_program::pubkey::Pubkey;
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct FundAccount {
    pub name: [u8; 27],
    pub expected_members: u32,
    pub creator_exists: bool,
    pub total_deposit: u64,
    pub governance_mint: Pubkey,
    pub vault: Pubkey,
    pub current_proposal_index: u8,
    pub created_at: i64,
    pub is_private: u8,
    pub members: Vec<Pubkey>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct VaultAccount {
    pub fund: Pubkey,
    pub last_deposit_time: i64,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct UserAccount {
    pub user: Pubkey,
    pub funds: Vec<UserSpecific>,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq)]
pub struct UserSpecific {
    pub fund: Pubkey,
    pub governance_token_balance: u64,
    pub is_pending: bool,
    pub is_eligible: bool,
    pub join_time: i64
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ProposalAggregatorAccount {
    pub fund: Pubkey,
    pub index: u8,
    pub proposals: Vec<Proposal>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Proposal {
    pub proposer: Pubkey,
    pub from_assets: Vec<Pubkey>,
    pub to_assets: Vec<Pubkey>,
    pub amounts: Vec<u64>,
    pub slippages: Vec<u16>,
    pub votes_yes: u64,
    pub votes_no: u64,
    pub creation_time: i64,
    pub deadline: i64,
    pub executed: bool,
    pub vec_index: u16,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct JoinProposalAggregator {
    pub fund: Pubkey,
    pub index: u8,
    pub join_proposals: Vec<JoinProposal>
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct JoinProposal {
    pub joiner: Pubkey,
    pub votes_yes: u64,
    pub votes_no: u64,
    pub creation_time: i64,
    pub proposal_index: u8
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VoteAccount {
    pub proposal_index: u8,
    pub vec_index: u16,
    pub voters: Vec<(Pubkey, u8)>
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct JoinVoteAccount {
    pub proposal_index: u8,
    pub voters: Vec<(Pubkey, u8)>
}
