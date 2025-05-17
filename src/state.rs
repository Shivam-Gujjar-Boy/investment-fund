use solana_program::pubkey::Pubkey;
use borsh::{BorshSerialize, BorshDeserialize};
use mpl_token_metadata::types::DataV2;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct FundAccount {
    pub creator: Pubkey,
    pub members: Vec<Pubkey>,
    pub total_deposit: u64,
    pub governance_mint: Pubkey,
    pub vault: Pubkey,
    pub is_initialized: bool,
    pub dex_program_ids: Vec<(u8, Pubkey)>,
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
    pub votes_yes: u64,
    pub votes_no: u64,
    pub deadline: i64,
    pub executed: bool,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct VoteAccount {
    pub voter: Pubkey,
    pub vote: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum MetadataInstruction {
    CreateMetadataAccountsV3 {
        data: DataV2,
        is_mutable: bool,
        update_authority_is_signer: bool,
        collection_details: Option<u8>
    }
}
