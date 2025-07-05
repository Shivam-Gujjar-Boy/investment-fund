use solana_program::pubkey::Pubkey;
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct LightFundAccount {
    pub name: [u8; 32],
    pub creator_exists: bool,
    pub total_deposit: u64,
    pub vault: Pubkey,
    pub current_proposal_index: u8,
    pub created_at: i64,
    pub tags: u32,
    pub max_members: u8,
    pub members: Vec<Pubkey>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct FundAccount {
    pub name: [u8; 26],
    pub is_refunded: bool,
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
    pub last_deposit_time: i64,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct UserAccount {
    pub user_cid: [u8; 59],
    pub funds: Vec<UserSpecific>,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq)]
pub struct UserSpecific {
    pub fund: Pubkey,
    pub fund_type: u8,
    pub governance_token_balance: u64,
    pub is_pending: bool,
    pub is_eligible: u8,
    pub join_time: i64
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ProposalAggregatorAccount {
    pub index: u8,
    pub proposals: Vec<Proposal>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Proposal {
    pub proposer: Pubkey,
    pub cid: [u8; 59],
    pub votes_yes: u64,
    pub votes_no: u64,
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

#[derive(BorshSerialize, BorshDeserialize)]
pub struct IncrementProposalAccount {
    pub proposer: Pubkey,
    pub new_size: u32,
    pub refund_type: u8,
    pub votes_yes: u64,
    pub votes_no: u64,
    pub voters: Vec<(Pubkey, u8)>
}