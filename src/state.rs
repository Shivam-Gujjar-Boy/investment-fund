use solana_program::pubkey::Pubkey;
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct FundAccount {
    pub members: Vec<Pubkey>,
    pub total_deposit: u64,
    pub governance_mint: Pubkey,
    pub vault: Pubkey,
    pub is_initialized: bool,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct UserSpecificAccount {
    pub pubkey: Pubkey,
    pub deposit: u64,
    pub governance_token_balance: u64,
    pub is_active: bool,
    pub num_proposals: u8,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct InvestmentProposalAccount {
    pub proposer: Pubkey,
    pub from_assets: Vec<Pubkey>,
    pub to_assets: Vec<Pubkey>,
    pub amounts: Vec<u64>,
    pub dex_tags: Vec<u8>,
    pub votes_yes: u8,
    pub votes_no: u8,
    pub deadline: i64,
    pub executed: bool,
}