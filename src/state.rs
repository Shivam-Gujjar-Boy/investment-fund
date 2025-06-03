use solana_program::pubkey::Pubkey;
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct FundAccount {
    pub name: [u8; 32],
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
    pub num_proposals: u16,
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
    pub slippage: Vec<u16>,
    pub votes_yes: u64,
    pub votes_no: u64,
    pub creation_time: i64,
    pub deadline: i64,
    pub executed: bool,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct VoteAccount {
    pub proposal_index: u8,
    pub vec_index: u8,
    pub voters: Vec<(Pubkey, u8)>
}

#[derive(BorshSerialize)]
pub struct SwapV2Args {
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit_x64: u128,
    pub is_base_input: bool
}
