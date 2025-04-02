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
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ProposalAccount {
    pub proposer: Pubkey,
    pub asset: Pubkey,
    pub amount: u64,
    pub dex: Pubkey,
    pub votes_yes: u64,
    pub votes_no: u64,
    pub deadline: i64,
    pub executed: bool,
}