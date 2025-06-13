use std::io::Write;
use std::vec;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::{Instruction, AccountMeta};
use solana_program:: pubkey;
use solana_program::{
    account_info::{next_account_info, AccountInfo}, clock::Clock, entrypoint::ProgramResult, msg, program::{invoke, invoke_signed}, program_pack::Pack, pubkey:: Pubkey, system_instruction, sysvar::{rent::Rent, Sysvar}
};
use spl_token::state::Account as TokenAccount;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token_2022::instruction::initialize_mint2;
use spl_token_2022::extension::ExtensionType;
use spl_token_2022::state::Mint;
use crate::state::{JoinProposal, JoinProposalAggregator, JoinVoteAccount, UserSpecific};
use crate::{
    errors::FundError,
    instruction::FundInstruction,
    state::{FundAccount, Proposal, ProposalAggregatorAccount, UserAccount, VaultAccount, VoteAccount}
};
use mpl_token_metadata::types::DataV2;
use mpl_token_metadata::instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs};

pub const TOKEN_METADATA_PROGRAM_ID: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a[AccountInfo<'a>],
    data: &[u8]
) -> ProgramResult {
    let instruction = FundInstruction::unpack(data)?;
    match instruction {

        FundInstruction::InitFundAccount { privacy, expected_members, fund_name} => {
            msg!("Instruction: Init Fund Account");
            process_init_fund_account(program_id, accounts, fund_name, privacy, expected_members)
        }

        FundInstruction::InitUserAccount {  } => {
            msg!("Instruction: Init User Account");
            process_init_user_account(program_id, accounts)
        }

        FundInstruction::AddFundMember { fund_name, vec_index } => {
            msg!("Instruction: Add Fund Member");
            process_add_member(program_id, accounts, fund_name, vec_index)
        }

        FundInstruction::InitDepositToken { amount, mint_amount, fund_name } => {
            msg!("Instruction: Init Deposit Token");
            process_init_deposit_token(program_id, accounts, amount, mint_amount, fund_name)
        }

        FundInstruction::InitProposalInvestment { 
            amounts,
            slippage,
            deadline,
            fund_name,
        } => {
            msg!("Instruction: Init Proposal");
            process_init_investment_proposal(program_id, accounts, amounts, slippage, deadline, fund_name)
        }

        FundInstruction::Vote {vote, proposal_index, vec_index, fund_name} => {
            msg!("Instruction: Voting on Proposal");
            process_vote_on_proposal(program_id, accounts, vote, proposal_index, vec_index, fund_name)
        }

        FundInstruction::InitRentAccount {  } => {
            msg!("Instruction: Init Rent Account");
            process_init_rent_account(program_id, accounts)
        }

        FundInstruction::LeaveFund { fund_name } => {
            msg!("Instruction: Leave Fund {}", fund_name);
            Ok(())
            // process_leave_fund(program_id, fund_name, accounts)
        }

        FundInstruction::ExecuteProposalInvestment { swap_number, fund_name, proposal_index, vec_index} => {
            msg!("Instruction: Execute Proposal");
            process_execute_proposal(program_id, accounts, swap_number, fund_name, proposal_index, vec_index)
        }

        FundInstruction::InitJoinProposal { fund_name } => {
            msg!("Instruction: Init Join Proposal");
            process_init_join_proposal(program_id, accounts, fund_name)
        }

        FundInstruction::JoinVote { vote, fund_name, vec_index } => {
            msg!("Instruction: Voting on Join Proposal");
            process_vote_on_join_proposal(program_id, accounts, vote, fund_name, vec_index)
        }

        // FundInstruction::DeleteFund {  } => {
        //     msg!("Instruction: Delete Fund");
        //     process_delete_fund(program_id, accounts)
        // }

        _ => Err(FundError::InvalidInstruction.into()),
    }
}

fn process_init_fund_account<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    fund_name: String,
    privacy: u8,
    expected_members: u32,
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let governance_mint_info = next_account_info(accounts_iter)?; // Governance Mint .........................
    let vault_account_info = next_account_info(accounts_iter)?; // Vault PDA Account .........................
    let system_program_info = next_account_info(accounts_iter)?; // System Program ...........................
    // let token_program_info = next_account_info(accounts_iter)?; // Token Program (2020) ......................
    let fund_account_info = next_account_info(accounts_iter)?; // Fund PDA Account ...........................
    let creator_wallet_info = next_account_info(accounts_iter)?; // Creator Wallet Address ...................
    let metadata_account_info = next_account_info(accounts_iter)?; // Metadat PDA for Governance Mint ........
    let rent_sysvar_info = next_account_info(accounts_iter)?; // Rent Sysvar .................................
    let token_metadata_program_info = next_account_info(accounts_iter)?; // Token Metadata Program ...........
    let user_account_info = next_account_info(accounts_iter)?; // Global User Account ........................
    let proposal_aggregator_info = next_account_info(accounts_iter)?; // first proposal aggregator ...........
    let join_proposal_aggregator_info = next_account_info(accounts_iter)?; // join proposal aggregator .......

    // Creator should be signer
    if !creator_wallet_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let index: u8 = 0;

    // Deriving required PDAs
    let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault", fund_pda.as_ref()], program_id);
    let (governance_mint, governance_bump) = Pubkey::find_program_address(&[b"governance", fund_pda.as_ref()], program_id);
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", creator_wallet_info.key.as_ref()], program_id);
    let (proposal_pda, proposal_bump) = Pubkey::find_program_address(&[b"proposal-aggregator", &[index], fund_pda.as_ref()], program_id);
    let (join_aggregator_pda, join_aggregator_bump) = Pubkey::find_program_address(&[b"join-proposal-aggregator", &[index], fund_pda.as_ref()], program_id);

    // Check if any of the provided PDA differes from the derived
    if *fund_account_info.key != fund_pda ||
       *vault_account_info.key != vault_pda ||
       *governance_mint_info.key != governance_mint ||
       *user_account_info.key != user_pda ||
       *proposal_aggregator_info.key != proposal_pda ||
       *join_proposal_aggregator_info.key != join_aggregator_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    // Check if the an account already exists on that PDA
    if fund_account_info.lamports() > 0 {
        msg!("Fund already exists!!");
        return Err(FundError::InvalidAccountData.into());
    }

    // Calculate Rent
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let fund_space = 150 as usize;
    let vault_space = 40 as usize;
    let extensions = vec![ExtensionType::NonTransferable];
    // let mint_space = spl_token::state::Mint::LEN;
    let mint_space = ExtensionType::try_calculate_account_len::<Mint>(&extensions)?;
    let proposal_space = 37 as usize;
    let join_proposal_space = 37 as usize;

    // Creating the Fund Account PDA
    invoke_signed(
        &system_instruction::create_account(
            creator_wallet_info.key,
            fund_account_info.key,
            rent.minimum_balance(fund_space),
            fund_space as u64,
            program_id,
        ),
        &[creator_wallet_info.clone(), fund_account_info.clone(), system_program_info.clone()],
        &[&[b"fund", fund_name.as_bytes(), &[fund_bump]]]
    )?;

    // Creating the Vault Account PDA
    invoke_signed(
        &system_instruction::create_account(
            creator_wallet_info.key,
            vault_account_info.key,
            rent.minimum_balance(vault_space),
            vault_space as u64,
            program_id,
        ),
        &[creator_wallet_info.clone(), vault_account_info.clone(), system_program_info.clone()],
        &[&[b"vault", fund_pda.as_ref(), &[vault_bump]]],
    )?;

    // Creating the Governance Mint account
    invoke_signed(
        &system_instruction::create_account(
            creator_wallet_info.key,
            governance_mint_info.key,
            rent.minimum_balance(mint_space),
            mint_space as u64,
            &spl_token_2022::id(),
        ),
        &[creator_wallet_info.clone(), governance_mint_info.clone(), system_program_info.clone()],
        &[&[b"governance", fund_pda.as_ref(), &[governance_bump]]],
    )?;
    invoke_signed(
        &initialize_mint2(
            &spl_token_2022::id(),
            governance_mint_info.key,
            fund_account_info.key,
            None,
            0,
        )?,
        &[governance_mint_info.clone(), rent_sysvar_info.clone()],
        &[&[b"governance", fund_pda.as_ref(), &[governance_bump]]],
    )?;

    // creating the proposal aggregator account
    invoke_signed(
        &system_instruction::create_account(
            creator_wallet_info.key,
            proposal_aggregator_info.key,
            rent.minimum_balance(proposal_space),
            proposal_space as u64,
            program_id
        ),
        &[creator_wallet_info.clone(), system_program_info.clone(), proposal_aggregator_info.clone()],
        &[&[b"proposal-aggregator", &[index], fund_pda.as_ref(), &[proposal_bump]]]
    )?;

    // creating the joining proposal aggregator account
    invoke_signed(
        &system_instruction::create_account(
            creator_wallet_info.key,
            join_proposal_aggregator_info.key,
            rent.minimum_balance(join_proposal_space),
            join_proposal_space as u64,
            program_id
        ),
        &[creator_wallet_info.clone(), system_program_info.clone(), join_proposal_aggregator_info.clone()],
        &[&[b"join-proposal-aggregator", &[index], fund_pda.as_ref(), &[join_aggregator_bump]]]
    )?;

    // Deriving PDA to store Mint metadata
    let (metadata_pda, _bump) = Pubkey::find_program_address(
        &[
            b"metadata",
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            governance_mint_info.key.as_ref(),
        ],
        &TOKEN_METADATA_PROGRAM_ID,
    );

    // Token metadata
    let token_name = fund_name.clone();
    let token_symbol = String::from("GOV");
    let token_uri = "".to_string();

    // Instruction to create Mtadata account at derived PDA
    let create_metadata_ix = CreateMetadataAccountV3 {
        metadata: metadata_pda,
        mint: *governance_mint_info.key,
        mint_authority: *fund_account_info.key,
        payer: *creator_wallet_info.key,
        update_authority: (*fund_account_info.key, true),
        system_program: *system_program_info.key,
        rent: None,
    }.instruction(CreateMetadataAccountV3InstructionArgs {
        data: DataV2 {
            name: token_name,
            symbol: token_symbol,
            uri: token_uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        is_mutable: true,
        collection_details: None,
    });

    // Invoking instruction to create Metadata PDA account
    invoke_signed(
        &create_metadata_ix,
        &[
            metadata_account_info.clone(),
            governance_mint_info.clone(),
            fund_account_info.clone(),
            fund_account_info.clone(),
            creator_wallet_info.clone(),
            system_program_info.clone(),
            rent_sysvar_info.clone(),
            token_metadata_program_info.clone(),
        ],
        &[&[b"fund", fund_name.as_bytes(), &[fund_bump]]]
    )?;

    // Converting the fund_name to an array of u8 of fixed size 32
    let bytes = fund_name.as_bytes();
    let mut array = [0u8; 27];
    let len = bytes.len().min(27);
    array[..len].copy_from_slice(&bytes[..len]);

    let members: Vec<Pubkey> = vec![*creator_wallet_info.key];

    // Deserialization and Serialization of Fund data
    let fund_data = FundAccount {
        name: array,
        expected_members,
        creator_exists: true,
        total_deposit: 0 as u64,
        governance_mint: *governance_mint_info.key,
        vault: *vault_account_info.key,
        current_proposal_index: 0 as u8,
        created_at: current_time,
        is_private: privacy,
        members,
    };
    fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

    // Deserializing the User Global PDA
    let mut user_data = UserAccount::try_from_slice(&user_account_info.data.borrow())?;

    // Calculate current size and new size of User PDA
    let current_size = user_account_info.data_len();
    let new_size = current_size + 50;

    // Calculate current min rent-exempt and new rent-exempt
    let new_min_balance = rent.minimum_balance(new_size);
    let current_balance = user_account_info.lamports();

    // Deposit lamports if required
    if new_min_balance > current_balance {
        invoke(
            &system_instruction::transfer(
                creator_wallet_info.key,
                user_account_info.key,
                new_min_balance - current_balance,
            ),
            &[creator_wallet_info.clone(), user_account_info.clone(), system_program_info.clone()],
        )?;
    }

    // Reallocation for new bytes
    user_account_info.realloc(new_size, false)?;
    // Add the new fund details to user funds
    user_data.funds.push(UserSpecific {
        fund: *fund_account_info.key,
        governance_token_balance: 0 as u64,
        num_proposals: 0 as u16,
        join_time: current_time
    });
    user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;

    // Deserialization and Serialization of Vault Account Data
    let vault_data = VaultAccount {
        fund: *fund_account_info.key,
        last_deposit_time: 0,
    };
    vault_data.serialize(&mut &mut vault_account_info.data.borrow_mut()[..])?;

    let proposals: Vec<Proposal> = vec![];

    let proposal_aggregator_data = ProposalAggregatorAccount {
        fund: *fund_account_info.key,
        index: 0 as u8,
        proposals
    };

    proposal_aggregator_data.serialize(&mut &mut proposal_aggregator_info.data.borrow_mut()[..])?;

    let join_proposals: Vec<JoinProposal> = vec![];

    let join_proposal_aggreagtor_data = JoinProposalAggregator {
        fund: *fund_account_info.key,
        index: 0 as u8,
        join_proposals,
    };

    join_proposal_aggreagtor_data.serialize(&mut &mut join_proposal_aggregator_info.data.borrow_mut()[..])?;

    msg!("[FUND-ACTIVITY] {} {} Fund created by: {}", fund_account_info.key.to_string(), current_time, creator_wallet_info.key.to_string());

    Ok(())
}

fn process_init_user_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo]
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let creator_account_info = next_account_info(accounts_iter)?; // User Wallet
    let user_account_info = next_account_info(accounts_iter)?; // User PDA Account to be created
    let system_program_info = next_account_info(accounts_iter)?; // System Program

    // User should be the signer
    if !creator_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    // Derive User PDA and check if provided is same as expected
    let (user_pda, user_bump) = Pubkey::find_program_address(&[b"user", creator_account_info.key.as_ref()], program_id);
    if *user_account_info.key != user_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    // If user PDA already exists
    if !user_account_info.data_is_empty() {
        msg!("User already exists");
        return Ok(());
    }

    // Calculate rent-exempt
    let rent = Rent::get()?;
    let user_space = 32 + 4 as usize;
    
    // Create the User PDA Account which is it's global identity
    invoke_signed(
        &system_instruction::create_account(
            creator_account_info.key,
            user_account_info.key,
            rent.minimum_balance(user_space),
            user_space as u64,
            program_id
        ),
        &[creator_account_info.clone(), user_account_info.clone(), system_program_info.clone()],
        &[&[b"user", creator_account_info.key.as_ref(), &[user_bump]]]
    )?;

    // Initially user is joined in no Funds
    let funds:Vec<UserSpecific> = vec![];

    // Deserialization and Serialization of User Account Data
    let user_data = UserAccount {
        user: *creator_account_info.key,
        funds,
    };
    user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;
    msg!("User Account created successfully");

    Ok(())

}

fn process_init_join_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: String,
) -> ProgramResult {
    let creation_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let joiner_account_info = next_account_info(accounts_iter)?; // joiner wallet ............................
    let join_proposal_aggregator_info = next_account_info(accounts_iter)?; // join proposal aggregator .......
    let fund_account_info = next_account_info(accounts_iter)?; // fund account ...............................
    let vote_account_info = next_account_info(accounts_iter)?; // join votes aggregator ......................
    let system_program_info = next_account_info(accounts_iter)?; // system program ...........................

    if !joiner_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let index = 0 as u8;

    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let (join_proposal_pda, _join_proposal_bump) = Pubkey::find_program_address(&[b"join-proposal-aggregator", &[index], fund_pda.as_ref()], program_id);

    if *fund_account_info.key != fund_pda || *join_proposal_aggregator_info.key != join_proposal_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let mut join_proposal_data = JoinProposalAggregator::try_from_slice(&join_proposal_aggregator_info.data.borrow())?;

    let rent = Rent::get()?;
    let extra_space = 57 as usize;
    let current_space = join_proposal_aggregator_info.data_len();
    let current_rent = join_proposal_aggregator_info.lamports();
    let new_space = current_space + extra_space;
    let new_rent = rent.minimum_balance(new_space);

    if new_rent > current_rent {
        invoke(
            &system_instruction::transfer(
                joiner_account_info.key,
                join_proposal_aggregator_info.key,
                new_rent - current_rent,
            ),
            &[joiner_account_info.clone(), join_proposal_aggregator_info.clone(), system_program_info.clone()]
        )?;
    }

    join_proposal_aggregator_info.realloc(new_space, false)?;
    join_proposal_data.join_proposals.push(JoinProposal {
        joiner: *joiner_account_info.key,
        votes_yes: 0 as u64,
        votes_no: 0 as u64,
        creation_time,
        executed: false
    });

    join_proposal_data.serialize(&mut &mut join_proposal_aggregator_info.data.borrow_mut()[..])?;

    let vec_index: u8 = (join_proposal_data.join_proposals.len() - 1) as u8;
    let (join_vote_pda, join_vote_bump) = Pubkey::find_program_address(&[b"join-vote", &[vec_index], fund_pda.as_ref()], program_id);
    if *vote_account_info.key != join_vote_pda {
        return Err(FundError::InvalidVoteAccount.into());
    }

    if !vote_account_info.data_is_empty() {
        return Err(FundError::InvalidVoteAccount.into());
    }

    let vote_space = 5 as usize;
    
    invoke_signed(
        &system_instruction::create_account(
            joiner_account_info.key,
            vote_account_info.key,
            rent.minimum_balance(vote_space),
            vote_space as u64,
            program_id
        ),
        &[joiner_account_info.clone(), vote_account_info.clone(), system_program_info.clone()],
        &[&[b"join-vote", &[vec_index], fund_pda.as_ref(), &[join_vote_bump]]]
    )?;

    let voters: Vec<(Pubkey, u8)> = vec![];

    let join_vote_data = JoinVoteAccount {
        vec_index,
        voters,
    };

    join_vote_data.serialize(&mut &mut vote_account_info.data.borrow_mut()[..])?;

    msg!("[FUND-ACTIVITY] {} {} {} created proposal to join the fund", fund_account_info.key.to_string(), creation_time, joiner_account_info.key.to_string());

    Ok(())
}

fn process_vote_on_join_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vote: u8,
    fund_name: String,
    vec_index: u8
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let voter_account_info = next_account_info(accounts_iter)?; // Voter Wallet ..................................
    let vote_account_info = next_account_info(accounts_iter)?; // Join Votes aggregator ..........................
    let join_proposal_aggregator_info = next_account_info(accounts_iter)?; // Join proposal aggregator ...........
    let system_program_info = next_account_info(accounts_iter)?; // System Program ...............................
    let fund_account_info = next_account_info(accounts_iter)?; // fund Account ...................................
    let governance_token_mint_info = next_account_info(accounts_iter)?; // Governance Mint account ...............
    let voter_token_account_info = next_account_info(accounts_iter)?; // Voter's governance token account ........

    if !voter_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let index = 0 as u8;
    let (join_proposal_pda, _join_proposal_bump) = Pubkey::find_program_address(&[b"join-proposal-aggregator", &[index], fund_pda.as_ref()], program_id);
    let (join_vote_pda, _join_vote_bump) = Pubkey::find_program_address(&[b"join-vote", &[vec_index], fund_pda.as_ref()], program_id);
    let (governance_mint, _governance_bump) = Pubkey::find_program_address(&[b"governance", fund_pda.as_ref()], program_id);

    if *fund_account_info.key != fund_pda ||
       *join_proposal_aggregator_info.key != join_proposal_pda ||
       *vote_account_info.key != join_vote_pda ||
       *governance_token_mint_info.key != governance_mint {
        return Err(FundError::InvalidAccountData.into());
       }

    let token_account = spl_associated_token_account::get_associated_token_address(
        voter_account_info.key,
        governance_token_mint_info.key
    );
    if token_account != *voter_token_account_info.key {
        return Err(FundError::InvalidTokenAccount.into());
    }

    if vote_account_info.data_is_empty() {
        return Err(FundError::InvalidVoteAccount.into());
    }

    let mut vote_data = JoinVoteAccount::try_from_slice(&mut &mut vote_account_info.data.borrow())?;

    // check if already voted
    let voter_exists = vote_data
        .voters
        .iter()
        .any(|(key, _)| *key == *voter_account_info.key);

    if voter_exists {
        return Err(FundError::AlreadyVoted.into());
    }

    let rent = Rent::get()?;
    let current_space = vote_account_info.data_len();
    let new_space = current_space + 33 as usize;
    let current_rent = vote_account_info.lamports();
    let new_rent = rent.minimum_balance(new_space);

    // transfer reallocation lamports
    if new_rent > current_rent {
        invoke(
            &system_instruction::transfer(
                voter_account_info.key,
                vote_account_info.key,
                new_rent - current_rent
            ),
            &[voter_account_info.clone(), vote_account_info.clone(), system_program_info.clone()]
        )?;
    }

    vote_account_info.realloc(new_space, false)?;
    vote_data.voters.push((*voter_account_info.key, vote));
    vote_data.serialize(&mut &mut vote_account_info.data.borrow_mut()[..])?;

    let join_proposal_data = JoinProposalAggregator::try_from_slice(&join_proposal_aggregator_info.data.borrow())?;

    msg!("[FUND-ACTIVITY] {} {} {} voted for addition of {}", fund_account_info.key.to_string(), current_time, voter_account_info.key.to_string(), join_proposal_data.join_proposals[vec_index as usize].joiner.to_string());

    Ok(())
}

fn process_add_member<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    fund_name: String,
    vec_index: u8,
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let fund_account_info = next_account_info(accounts_iter)?; // Fund Account ..................................
    let member_account_info = next_account_info(accounts_iter)?; // User to be added ............................
    let system_program_info = next_account_info(accounts_iter)?; // System Program ..............................
    let user_account_info = next_account_info(accounts_iter)?; // User Global identity account ..................
    let rent_reserve_info = next_account_info(accounts_iter)?; // peerfund's rent reserve .......................

    // User should be signer
    if !member_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    // Derive PDAs and check if it is same as provided in accounts
    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", member_account_info.key.as_ref()], program_id);
    let (rent_pda, rent_bump) = Pubkey::find_program_address(&[b"rent"], program_id);
    if *fund_account_info.key != fund_pda || *user_account_info.key != user_pda || *rent_reserve_info.key != rent_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    // Check if Fund exist or not!
    if fund_account_info.data_is_empty() {
        return Err(FundError::InvalidAccountData.into());
    }

    // Deserialize User Data and check if User is already a member of provided Fund
    let mut user_data = UserAccount::try_from_slice(&user_account_info.data.borrow())?;
    if user_data.funds.iter().any(|entry| entry.fund == *fund_account_info.key) {
        msg!("User is already a member");
        return Ok(());
    }

    // Deserialize the fund data
    let mut fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    let total_voting_power = fund_data.total_deposit;

    // If fund is private
    if fund_data.is_private == 1 as u8 {
        let join_proposal_aggregator_info = next_account_info(accounts_iter)?;
        let index = 0 as u8;
        let (join_proposal_pda, _join_proposal_bump) = Pubkey::find_program_address(&[b"join-proposal-aggregator", &[index], fund_pda.as_ref()], program_id);
        if *join_proposal_aggregator_info.key != join_proposal_pda {
            return Err(FundError::InvalidProposalAccount.into());
        }

        let mut join_proposal_data = JoinProposalAggregator::try_from_slice(&join_proposal_aggregator_info.data.borrow())?;
        if join_proposal_data.join_proposals[vec_index as usize].executed {
            return Err(FundError::AlreadyExecuted.into());
        }

        if 2*(join_proposal_data.join_proposals[vec_index as usize].votes_yes) < total_voting_power {
            return Err(FundError::NotEnoughVotes.into());
        }

        join_proposal_data.join_proposals[vec_index as usize].executed = true;
        join_proposal_data.serialize(&mut &mut join_proposal_aggregator_info.data.borrow_mut()[..])?;
    }

    // Calculate if more lamports are needed for reallocation of User Global Account
    let user_current_size = user_account_info.data_len();
    let user_new_size = user_current_size + 50;
    let rent = Rent::get()?;
    let user_new_min_balance = rent.minimum_balance(user_new_size);
    let user_current_balance = user_account_info.lamports();
    if user_new_min_balance > user_current_balance {
        invoke(
            &system_instruction::transfer(
                member_account_info.key,
                user_account_info.key,
                user_new_min_balance - user_current_balance,
            ),
            &[member_account_info.clone(), user_account_info.clone(), system_program_info.clone()],
        )?;
    }

    // Reallocate new bytes ofr storage of new Fund details
    user_account_info.realloc(user_new_size, false)?;
    user_data.funds.push(UserSpecific {
        fund: *fund_account_info.key,
        governance_token_balance: 0 as u64,
        num_proposals: 0 as u16,
        join_time: current_time
    });
    user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;


    // Check if user already exists
    if fund_data.members.contains(member_account_info.key) {
        msg!("User is already in the fund!");
        return Err(FundError::InvalidAccountData.into());
    }

    // Calculate if more lamports are needed for reallocation of Fund Account
    let fund_current_size = fund_account_info.data_len();
    let fund_new_size = fund_current_size + 32;
    let fund_new_min_balance = rent.minimum_balance(fund_new_size);
    let fund_current_balance = fund_account_info.lamports();

    if fund_new_min_balance > fund_current_balance {
        invoke(
            &system_instruction::transfer(
                member_account_info.key,
                fund_account_info.key,
                fund_new_min_balance - fund_current_balance
            ),
            &[member_account_info.clone(), fund_account_info.clone(), system_program_info.clone()]
        )?;
    }

    // Reallocate new bytes for storage of new member pubkey
    fund_account_info.realloc(fund_new_size, false)?;
    let mut fund_data_2 = fund_account_info.data.borrow_mut();
    msg!("raw fund data after reallocation: {:?}", fund_data_2);
    fund_data.members.push(*member_account_info.key);
    fund_data.serialize(&mut &mut fund_data_2[..])?;

    // Refund logic
    if fund_data.expected_members >= fund_data.members.len() as u32 {
        let refund_per_member: u64 = 25_000_000 / (fund_data.expected_members as u64);
        invoke(
            &system_instruction::transfer(
                member_account_info.key,
                rent_reserve_info.key,
                refund_per_member
            ),
            &[member_account_info.clone(), rent_reserve_info.clone(), system_program_info.clone()]
        )?;
    }

    // If expected number of members are achieved, refund back to creator
    if fund_data.expected_members == fund_data.members.len() as u32 {
        let fund_creator_info = next_account_info(accounts_iter)?;

        if *fund_creator_info.key != fund_data.members[0] {
            return Err(FundError::InvalidFundCreator.into());
        }

        let refund_to_creator: u64 = ((fund_data.expected_members - 1) as u64 * 22_000_000) / (fund_data.expected_members as u64);

        invoke_signed(
            &system_instruction::transfer(
                rent_reserve_info.key,
                fund_creator_info.key,
                refund_to_creator,
            ),
            &[fund_creator_info.clone(), rent_reserve_info.clone(), system_program_info.clone()],
            &[&[b"rent", &[rent_bump]]]
        )?;
    }

    msg!("[FUND-ACTIVITY] {} {} Member joined: {}", fund_account_info.key.to_string(), current_time, member_account_info.key.to_string());

    Ok(())

}

fn process_init_deposit_token(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    mint_amount: u64,
    fund_name: String,
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let member_account_info = next_account_info(accounts_iter)?; // Depositor wallet .................................
    let member_ata_info = next_account_info(accounts_iter)?; // Depositor's ATA for the depositing mint ..............
    let vault_account_info = next_account_info(accounts_iter)?; // Vault PDA .........................................
    let vault_ata_info = next_account_info(accounts_iter)?; // Vault PDA's ATA for the depositing mint ...............
    let mint_account_info = next_account_info(accounts_iter)?; // Mint account of token to be deposited ..............
    let token_program_info = next_account_info(accounts_iter)?; // Token Progarm .....................................
    let ata_program_info = next_account_info(accounts_iter)?; // Associated Token Program ............................
    let fund_account_info = next_account_info(accounts_iter)?; // Fund PDA ...........................................
    let user_account_info = next_account_info(accounts_iter)?; // user global account ................................
    let system_program_info = next_account_info(accounts_iter)?; // System program ...................................
    let rent_sysvar_info = next_account_info(accounts_iter)?; // Rent Sysvar Account .................................
    let governance_token_account_info = next_account_info(accounts_iter)?; // Governance Token Account of depositor ..
    let governance_mint_info = next_account_info(accounts_iter)?; // Governance Mint Account .........................

    // Depositor should be signer
    if !member_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    // Derive the PDAs and check for equality with provided ones
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault", fund_account_info.key.as_ref()], program_id);
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", member_account_info.key.as_ref()], program_id);
    let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    if *vault_account_info.key != vault_pda || *fund_account_info.key != fund_pda || *user_account_info.key != user_pda {
        return Err(FundError::InvalidAccountData.into());
    }
    let expected_ata = spl_associated_token_account::get_associated_token_address(
        member_account_info.key,
        governance_mint_info.key,
    );
    if *governance_token_account_info.key != expected_ata {
        return Err(FundError::InvalidTokenAccount.into());
    }

    // If user doesn't exist in the fund, it can't deposit
    let mut user_data = UserAccount::try_from_slice(&user_account_info.data.borrow())?;
    if !user_data.funds.iter().any(|entry| entry.fund == *fund_account_info.key) {
        msg!("User is not a member of the fund");
        return Err(FundError::InvalidAccountData.into());
    }

    // If depositor's governance token account doesn't exist, create one
    if governance_token_account_info.data_is_empty() {
        invoke(
            &spl_associated_token_account::instruction::create_associated_token_account(
                member_account_info.key,
                member_account_info.key,
                governance_mint_info.key,
                token_program_info.key,
            ),
            &[
                member_account_info.clone(),
                governance_token_account_info.clone(),
                token_program_info.clone(),
                governance_mint_info.clone(),
                rent_sysvar_info.clone(),
            ]
        )?;
    }

    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let rent_req = rent.minimum_balance(TokenAccount::LEN);

    // If vault's token account account for the depositing mint doesn't exist, create it
    if vault_ata_info.data_is_empty() {
        msg!("Creating Vault ATA...");

        invoke_signed(
            &create_associated_token_account(
                member_account_info.key,
                vault_account_info.key,
                mint_account_info.key,
                token_program_info.key
            ),
            &[
                member_account_info.clone(),
                vault_ata_info.clone(),
                vault_account_info.clone(),
                mint_account_info.clone(),
                system_program_info.clone(),
                token_program_info.clone(),
                ata_program_info.clone(),
                rent_sysvar_info.clone(),
            ],
            &[&[b"vault", fund_account_info.key.as_ref(), &[vault_bump]]]
        )?;
    }

    let mint: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

    if *mint_account_info.key == mint {
        invoke(
            &system_instruction::create_account(
                member_account_info.key,
                member_ata_info.key,
                rent_req+amount,
                TokenAccount::LEN as u64,
                token_program_info.key,
            ),
            &[
                member_account_info.clone(),
                member_ata_info.clone(),
                token_program_info.clone(),
                system_program_info.clone(),
                rent_sysvar_info.clone(),
            ]
        )?;

        invoke(
            &spl_token::instruction::initialize_account(
                token_program_info.key,
                member_ata_info.key,
                mint_account_info.key,
                member_account_info.key,
            )?,
            &[
                token_program_info.clone(),
                member_ata_info.clone(),
                mint_account_info.clone(),
                member_account_info.clone(),
                rent_sysvar_info.clone(),
            ]
        )?;

        invoke(
            &spl_token::instruction::transfer(
                token_program_info.key,
                member_ata_info.key,
                vault_ata_info.key,
                member_account_info.key,
                &[],
                amount
            )?,
            &[
                token_program_info.clone(),
                member_ata_info.clone(),
                vault_ata_info.clone(),
                member_account_info.clone(),
            ]
        )?;

        invoke(
            &spl_token::instruction::close_account(
                token_program_info.key,
                member_ata_info.key,
                member_account_info.key,
                member_account_info.key,
                &[]
            )?,
            &[
                token_program_info.clone(),
                member_account_info.clone(),
                member_ata_info.clone()
               ]
        )?;
    } else {
        msg!("Transferring tokens...");
        // Now transfer the required number of tokens from depositor's token account to vault's token account
        invoke_signed(
            &spl_token::instruction::transfer(
                token_program_info.key,
                member_ata_info.key,
                vault_ata_info.key,
                member_account_info.key,
                &[],
                amount,
            )?,
            &[
                member_ata_info.clone(),
                vault_ata_info.clone(),
                member_account_info.clone(),
                token_program_info.clone(),
            ],
            &[]
        )?;
    }


    // Now mint equivalent quantity of governance tokens to the depositor's governance token account
    invoke_signed(
        &spl_token::instruction::mint_to(
            token_program_info.key,
            governance_mint_info.key,
            governance_token_account_info.key,
            fund_account_info.key,
            &[],
            mint_amount,
        )?,
        &[
            governance_mint_info.clone(),
            governance_token_account_info.clone(),
            fund_account_info.clone(),
            token_program_info.clone(),
        ],
        &[&[b"fund", fund_name.as_bytes(), &[fund_bump]]],
    )?;

    // In vault account, set the last deposit time
    let mut vault_data = VaultAccount::try_from_slice(&vault_account_info.data.borrow())?;
    vault_data.last_deposit_time = current_time;
    vault_data.serialize(&mut &mut vault_account_info.data.borrow_mut()[..])?;

    // In fund account increase the deposited amount (unit lamports)
    let mut fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    fund_data.total_deposit += mint_amount;
    fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

    if let Some(entry) = user_data.funds.iter_mut().find(|entry| entry.fund == *fund_account_info.key) {
        entry.governance_token_balance += mint_amount;
    } else {
        msg!("Fund entry not found for user");
        return Err(FundError::InvalidAccountData.into());
    }

    user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;

    msg!("[FUND-ACTIVITY] {} {} Token deposit: {} of {} by {}", fund_account_info.key.to_string(), current_time, amount, mint_account_info.key.to_string(), member_account_info.key.to_string());

    Ok(())
}

fn process_init_investment_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amounts: Vec<u64>,
    slippage: Vec<u16>,
    deadline: i64,
    fund_name: String,
) -> ProgramResult {
    let creation_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let proposer_account_info = next_account_info(accounts_iter)?; // Proposer Wallet ............................
    let user_account_info = next_account_info(accounts_iter)?; // Proposer's Global Account ......................
    let fund_account_info = next_account_info(accounts_iter)?; // Fund Account ...................................
    let proposal_aggregator_info = next_account_info(accounts_iter)?; // Proposal Account ........................
    let system_program_info = next_account_info(accounts_iter)?; // System Program ...............................
    let new_proposal_aggregator_info = next_account_info(accounts_iter)?; // new proposal aggregator account .....
    let vote_account_a_info = next_account_info(accounts_iter)?; // vote account 1 ...............................
    let vote_account_b_info = next_account_info(accounts_iter)?; // vote account 2 ...............................

    // Proposer needs to be signer
    if !proposer_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    // Derive PDAs and check for equality with the provided ones
    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", proposer_account_info.key.as_ref()], program_id);
    let mut fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    let current_index = fund_data.current_proposal_index;

    if *fund_account_info.key != fund_pda || *user_account_info.key != user_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let (proposal_pda, _proposal_bump) = Pubkey::find_program_address(
        &[
            b"proposal-aggregator",
            &[current_index],
            fund_pda.as_ref()
        ],
        program_id
    );

    let (new_proposal_pda, new_proposal_bump) = Pubkey::find_program_address(
        &[
            b"proposal-aggregator",
            &[current_index + 1],
            fund_pda.as_ref()
        ],
        program_id
    );

    if *proposal_aggregator_info.key != proposal_pda || *new_proposal_aggregator_info.key != new_proposal_pda {
        msg!("Proposal pds are invalid");
        return Err(FundError::InvalidProposalAccount.into());
    }

    // Extract From Assets Mint
    let from_assets_info : Vec<&AccountInfo> = accounts_iter
        .take(amounts.len())
        .collect();
    if from_assets_info.len() != amounts.len() {
        return Err(FundError::InvalidAccountData.into());
    }
    let from_assets_mints: Vec<Pubkey> = from_assets_info.iter().map(|m| *m.key).collect();

    // Extract To Assets Mint
    let to_assets_info: Vec<&AccountInfo> = accounts_iter
        .take(amounts.len())
        .collect();
    if to_assets_info.len() != amounts.len() {
        return Err(FundError::InvalidAccountData.into());
    }
    let to_assets_mints: Vec<Pubkey> = to_assets_info.iter().map(|m| *m.key).collect();

    // Deserialization and Serialization of Proposal Account data
    // let proposal_info = 

    // Rent Calculation
    let rent = Rent::get()?;
    // let proposal_space = 81 + to_assets_info.len()*74;
    let extra_proposal_space = (81 + to_assets_info.len()*74) as usize;
    let current_proposal_space = proposal_aggregator_info.data_len();

    let mut flag = false;

    let mut proposal_aggregator_data = ProposalAggregatorAccount::try_from_slice(&proposal_aggregator_info.data.borrow())?;

    // check if new proposal aggregator is required
    if (current_proposal_space + extra_proposal_space) > 10240 as usize {
        // Create Proposal Account
        invoke_signed(
            &system_instruction::create_account(
                proposer_account_info.key,
                new_proposal_aggregator_info.key,
                rent.minimum_balance(37 + extra_proposal_space),
                (37 + extra_proposal_space) as u64,
                program_id
            ),
            &[
                new_proposal_aggregator_info.clone(),
                proposer_account_info.clone(),
                system_program_info.clone(),
            ],
            &[&[
                b"proposal-aggregator",
                &[current_index + 1],
                fund_account_info.key.as_ref(),
                &[new_proposal_bump]
            ]]
        )?;

        let proposals_vec: Vec<Proposal> = vec![ Proposal {
            proposer: *proposer_account_info.key,
            from_assets: from_assets_mints,
            to_assets: to_assets_mints,
            amounts,
            slippage,
            votes_yes: 0 as u64,
            votes_no: 0 as u64,
            creation_time,
            deadline,
            executed: false
        }];

        let new_proposal_data = ProposalAggregatorAccount {
            fund: *fund_account_info.key,
            index: current_index + 1,
            proposals: proposals_vec
        };

        new_proposal_data.serialize(&mut &mut new_proposal_aggregator_info.data.borrow_mut()[..])?;

        fund_data.current_proposal_index += 1;
        fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;
        flag = true;
    } else {
        let new_aggregator_size = extra_proposal_space + current_proposal_space as usize;
        let current_rent_exempt = proposal_aggregator_info.lamports();
        let new_rent_exempt = rent.minimum_balance(new_aggregator_size);

        if new_rent_exempt > current_rent_exempt {
            invoke(
                &system_instruction::transfer(
                    proposer_account_info.key,
                    proposal_aggregator_info.key,
                    new_rent_exempt-current_rent_exempt
                ),
                &[proposal_aggregator_info.clone(), proposer_account_info.clone(), system_program_info.clone()]
            )?;
        }

        proposal_aggregator_info.realloc(new_aggregator_size, false)?;
        
        proposal_aggregator_data.proposals.push( Proposal {
            proposer: *proposer_account_info.key,
            from_assets: from_assets_mints,
            to_assets: to_assets_mints,
            amounts,
            slippage,
            votes_yes: 0 as u64,
            votes_no: 0 as u64,
            creation_time,
            deadline,
            executed: false
        });

        proposal_aggregator_data.serialize(&mut &mut proposal_aggregator_info.data.borrow_mut()[..])?;
    }

    let mut proposal_index = current_index;
    let mut vec_index = 0 as u8;

    if flag {
        proposal_index = current_index + 1;
    } else {
        let proposal_data = ProposalAggregatorAccount::try_from_slice(&proposal_aggregator_info.data.borrow())?;
        vec_index = (proposal_data.proposals.len()) as u8;
    }

    let (vote_pda, vote_bump) = Pubkey::find_program_address(&[b"vote", &[proposal_index], &[vec_index], fund_account_info.key.as_ref()], program_id);

    if *vote_account_a_info.key != vote_pda && *vote_account_b_info.key != vote_pda {
        msg!("Wrong vote account");
        return Err(FundError::InvalidVoteAccount.into());
    }

    let vote_account_size = 6 as usize;
    let vote_rent = rent.minimum_balance(vote_account_size);

    let voters: Vec<(Pubkey, u8)> = vec![];

    let vote_info = VoteAccount {
        proposal_index,
        vec_index,
        voters
    };

    if *vote_account_a_info.key == vote_pda {
        msg!("yaha aaya");
        invoke_signed(
            &system_instruction::create_account(
                proposer_account_info.key,
                vote_account_a_info.key,
                vote_rent,
                vote_account_size as u64,
                program_id
            ),
            &[proposer_account_info.clone(), vote_account_a_info.clone(), system_program_info.clone()],
            &[&[b"vote", &[proposal_index], &[vec_index], fund_pda.as_ref(), &[vote_bump]]]
        )?;

        vote_info.serialize(&mut &mut vote_account_a_info.data.borrow_mut()[..])?;
    } else {
        invoke_signed(
            &system_instruction::create_account(
                proposer_account_info.key,
                vote_account_b_info.key,
                vote_rent,
                vote_account_size as u64,
                program_id
            ),
            &[proposer_account_info.clone(), vote_account_b_info.clone(), system_program_info.clone()],
            &[&[b"vote", &[proposal_index], &[vec_index], fund_pda.as_ref(), &[vote_bump]]]
        )?;

        vote_info.serialize(&mut &mut vote_account_b_info.data.borrow_mut()[..])?;
    }

    let mut user_data = UserAccount::try_from_slice(&user_account_info.data.borrow())?;

    if let Some(user_specific) = user_data
        .funds
        .iter_mut()
        .find(|entry| entry.fund == *fund_account_info.key) {
            user_specific.num_proposals += 1;
        } else {
            msg!("User is not a member in this fund");
            return Err(FundError::InvalidAccountData.into());
        }

    user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;

    msg!("[FUND-ACTIVITY] {} {} Proposal created: ({}, {}) by {}", fund_account_info.key.to_string(), creation_time, proposal_index, vec_index, proposer_account_info.key.to_string());

    Ok(())
}


fn process_vote_on_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vote: u8,
    proposal_index: u8,
    vec_index: u8,
    fund_name: String,
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let voter_account_info = next_account_info(accounts_iter)?; // Voter's Wallet ................................
    let vote_account_info = next_account_info(accounts_iter)?; // Voter's vote account to be created .............
    let proposal_aggregator_info = next_account_info(accounts_iter)?; // Proposal account ........................
    let system_program_info = next_account_info(accounts_iter)?; // System Program ...............................
    let fund_account_info = next_account_info(accounts_iter)?; // fund Account ...................................
    let governance_token_mint_info = next_account_info(accounts_iter)?; // Governance Mint account ...............
    let voter_token_account_info = next_account_info(accounts_iter)?; // Voter's governance token account ........

    // Voter needs to be signer
    if !voter_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    // Pdas derivation
    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let (vote_pda, _vote_bump) = Pubkey::find_program_address(&[b"vote", &[proposal_index], &[vec_index], fund_account_info.key.as_ref()], program_id);
    let token_account = spl_associated_token_account::get_associated_token_address(
        voter_account_info.key,
        governance_token_mint_info.key
    );

    let (proposal_pda, _proposal_bump) = Pubkey::find_program_address(&[b"proposal-aggregator", &[proposal_index], fund_account_info.key.as_ref()], program_id);

    // Pdas verification
    if *fund_account_info.key != fund_pda ||
       *vote_account_info.key != vote_pda ||
       token_account != *voter_token_account_info.key ||
       *proposal_aggregator_info.key != proposal_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    if fund_data.governance_mint != *governance_token_mint_info.key {
        return Err(FundError::InvalidGovernanceMint.into());
    }

    let rent = Rent::get()?;
    let extra_vote_space = 33 as usize;
    let current_vote_space = vote_account_info.data_len();
    let new_vote_space = current_vote_space + extra_vote_space;
    let new_rent = rent.minimum_balance(new_vote_space);
    let current_rent = vote_account_info.lamports();

    let mut vote_data = VoteAccount::try_from_slice(&vote_account_info.data.borrow())?;

    let voter_exists = vote_data
        .voters
        .iter()
        .any(|(key, _)| *key == *voter_account_info.key);

    if voter_exists {
        return Err(FundError::AlreadyVoted.into());
    }

    let mut proposal_aggregator_data = ProposalAggregatorAccount::try_from_slice(&proposal_aggregator_info.data.borrow())?;
    // let mut proposal_data = proposal_aggregator_data.proposals[vec_index as usize];
    if proposal_aggregator_data.proposals[vec_index as usize].deadline < current_time {
        return Err(FundError::VotingCeased.into());
    }

    if vote_account_info.data_is_empty() {
        msg!("Vote account should be already created");
        return Err(FundError::InvalidVoteAccount.into());
    } else {
        if new_rent > current_rent {
            invoke(
                &system_instruction::transfer(
                    voter_account_info.key,
                    vote_account_info.key,
                    new_rent - current_rent
                ),
                &[system_program_info.clone(), voter_account_info.clone(), vote_account_info.clone()]
            )?;
        }

        vote_account_info.realloc(new_vote_space, false)?;
        
        vote_data.voters.push((*voter_account_info.key, vote));
        // msg!("vote data : {:?}", vote_data);
        vote_data.serialize(&mut &mut vote_account_info.data.borrow_mut()[..])?;

        let token_account_data = TokenAccount::unpack(&voter_token_account_info.data.borrow())?;

        msg!("token account gadbad kr rha hai kya");
        if vote != 0 {
            proposal_aggregator_data.proposals[vec_index as usize].votes_yes += token_account_data.amount;
        } else {
            proposal_aggregator_data.proposals[vec_index as usize].votes_no += token_account_data.amount;
        }

        proposal_aggregator_data.serialize(&mut &mut proposal_aggregator_info.data.borrow_mut()[..])?;
    }

    msg!("[FUND-ACTIVITY] {} {} Vote: {} on proposal ({}, {})", fund_account_info.key.to_string(), current_time, voter_account_info.key.to_string(), proposal_index, vec_index);

    Ok(())
}

fn process_init_rent_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let rent_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    let god_father_info = next_account_info(accounts_iter)?;

    if !god_father_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (rent_pda, rent_bump) = Pubkey::find_program_address(&[b"rent"], program_id);

    if *rent_account_info.key != rent_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let rent = Rent::get()?;
    let data_len = 0 as usize;
    let rent_exemption_amount = rent.minimum_balance(data_len);

    invoke_signed(
        &system_instruction::create_account(
            god_father_info.key,
            rent_account_info.key,
            rent_exemption_amount,
            data_len as u64,
            program_id
        ),
        &[god_father_info.clone(), rent_account_info.clone(), system_program_info.clone()],
        &[&[b"rent", &[rent_bump]]]
    )?;

    Ok(())
}


//    LEAVING FUND
//    delete fund specific pda
//    change member array size in fund pda
//    change User account pda's size

// fn process_leave_fund(
//     program_id: &Pubkey,
//     fund_name: String,
//     accounts: &[AccountInfo],
    
// ) -> ProgramResult {

//     // let current_time = Clock::get()?.unix_timestamp;

//     let accounts_iter = &mut accounts.iter();
//     let member_wallet_info =next_account_info(accounts_iter)?;
//     // let user_specific_info = next_account_info(accounts_iter)?; // to be deleted
//     let fund_account_info= next_account_info(accounts_iter)?;  // to change number of members
//     let user_account_info=next_account_info(accounts_iter)?;   // to change fund details
//     // let system_program_info = next_account_info(accounts_iter)?;  // to delete fund specific account
//     // let voter_account_info = next_account_info(accounts_iter)?;
//     // let proposal_account_info = next_account_info(accounts_iter)?;

//     if !member_wallet_info.is_signer {
//         return Err(FundError::InvalidAccountData.into());
//     }


//     let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
//     // let (user_specific_pda, _user_specific_bump) = Pubkey::find_program_address(&[b"user", fund_pda.as_ref(), member_wallet_info.key.as_ref()], program_id);
//     let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", member_wallet_info.key.as_ref()], program_id);
//     let mut user_data = UserAccount::try_from_slice(&user_account_info.data.borrow())?;
//     let mut fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
//     // let (proposal_pda, _proposal_bump) = Pubkey::find_program_address(&[b"proposal-investment", user_account_info.key.as_ref(), &[user_data.num_proposals]], program_id);
//     // let proposal_data = InvestmentProposalAccount::try_from_slice(&proposal_account_info.data.borrow())?;
//     if *fund_account_info.key != fund_pda ||
//     // *user_specific_info.key != user_specific_pda ||
//     *user_account_info.key != user_pda {
//         return Err(FundError::InvalidAccountData.into());
//     }



//     // if !user_specific_info.data_is_empty(){
//     //     let lamports = **user_specific_info.try_borrow_lamports()?;
//     //     **member_wallet_info.try_borrow_mut_lamports()? += lamports;
//     //     **user_specific_info.try_borrow_mut_lamports()? = 0;
    
//     //     // let mut data = user_specific_info.try_borrow_mut_data()?;
//     //     // for byte in data.iter_mut() {
//     //     //     *byte = 0;
//     //     // }
    
//     //     msg!("User-specific fund account closed and lamports sent to user");
        
//     // }
    
//     let current_rent = user_account_info.lamports();

//     let mut flag = false;

//     user_data.funds.retain(|key| {
//         let keep = key != fund_account_info.key;
//         if !keep {
//             flag = true;
//         }
//         keep
//     });

//     if flag {

//         fund_data.members-=1;

//         let rent = Rent::get()?;

//         let current_size = user_account_info.data_len();
//         let new_size= current_size-32;
//         let new_rent = rent.minimum_balance(new_size);
    
//         if new_rent < current_rent {
//             // let lamports = **user_account_info.try_borrow_lamports()?;
//             **user_account_info.try_borrow_mut_lamports()? -= current_rent-new_rent;
//             **member_wallet_info.try_borrow_mut_lamports()? += current_rent-new_rent;
//         }

//         user_account_info.realloc(new_size, false)?;
//     }

//     user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;
//     fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

//     Ok(())
// }


// fn process_delete_fund(
//     _program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     // fund_name: String,
// ) -> ProgramResult {
//     let accounts_iter = &mut accounts.iter();
//     let fund_account_info = next_account_info(accounts_iter)?;
//     let recipient_account_info = next_account_info(accounts_iter)?;
//     let vault_account_info = next_account_info(accounts_iter)?;
//     let governance_mint_info = next_account_info(accounts_iter)?;

//     **recipient_account_info.lamports.borrow_mut() += **governance_mint_info.lamports.borrow();
//     **governance_mint_info.lamports.borrow_mut() = 0;
//     let mut gov_data = governance_mint_info.try_borrow_mut_data()?;
//     for byte in gov_data.iter_mut() {
//         *byte = 0;
//     }

//     **recipient_account_info.lamports.borrow_mut() += **vault_account_info.lamports.borrow();
//     **vault_account_info.lamports.borrow_mut() = 0;
//     let mut gov_data = vault_account_info.try_borrow_mut_data()?;
//     for byte in gov_data.iter_mut() {
//         *byte = 0;
//     }

//     **recipient_account_info.lamports.borrow_mut() += **fund_account_info.lamports.borrow();
//     **fund_account_info.lamports.borrow_mut() = 0;
//     let mut gov_data = fund_account_info.try_borrow_mut_data()?;
//     for byte in gov_data.iter_mut() {
//         *byte = 0;
//     }

//     Ok(())
// }


fn process_execute_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    swap_number: u8,
    fund_name: String,
    proposal_index: u8,
    vec_index: u8,
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let account_iter = &mut accounts.iter();
    let payer_info = next_account_info(account_iter)?; // payer ...............................................
    // let rent_account_info = next_account_info(account_iter)?; // rent account .................................
    let fund_account_info = next_account_info(account_iter)?; // fund account .................................
    let vault_account_info = next_account_info(account_iter)?; // fund's vault account ........................
    let proposal_aggregator_info = next_account_info(account_iter)?; // proposal account ......................
    let token_program_2022_info = next_account_info(account_iter)?; // tokn program 2022 ......................
    let token_program_std_info = next_account_info(account_iter)?; // tokn program 2020 .......................
    let raydium_clmm_program = next_account_info(account_iter)?; // raydium clmm program ......................
    let amm_config = next_account_info(account_iter)?; // Amm config account ..................................
    let pool_state = next_account_info(account_iter)?; // Pool state account ..................................
    let input_token_account = next_account_info(account_iter)?; // Fund's vault input token account .. Should exist
    let output_token_account = next_account_info(account_iter)?; // fund's output token account .. might need to be created
    let input_vault_ata = next_account_info(account_iter)?; // Raydium pool's input vault ata .................
    let output_vault_ata = next_account_info(account_iter)?; // Raysium pool's output vault ata ...............
    let observation_state = next_account_info(account_iter)?; // Observation state account ....................
    let input_token_mint = next_account_info(account_iter)?; // Input token mint account ......................
    let output_token_mint = next_account_info(account_iter)?; // Output token mint account ....................
    let memo_program = next_account_info(account_iter)?; // memo program ......................................
    let system_program_info = next_account_info(account_iter)?; // system program .............................
    let ata_program_info = next_account_info(account_iter)?; // ata program ...................................
    let rent_sysvar_info = next_account_info(account_iter)?; // rent sysvar account ...........................

    if !payer_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (proposal_pda, _proposal_bump) = Pubkey::find_program_address(&[b"proposal-aggregator", &[proposal_index], fund_account_info.key.as_ref()], program_id);
    if *proposal_aggregator_info.key != proposal_pda {
        msg!("Wrong proposal aggregator account");
        return Err(FundError::InvalidProposalAccount.into());
    }

    let mut proposal_aggregator_data= ProposalAggregatorAccount::try_from_slice(&proposal_aggregator_info.data.borrow())?;
    let is_executed = proposal_aggregator_data.proposals[vec_index as usize].executed;

    // if proposal is executed then return
    if is_executed {
        msg!("Proposal already executed");
        return Err(FundError::InvalidAccountData.into());
    }

    let deadline = proposal_aggregator_data.proposals[vec_index as usize].deadline;

    // if voting deadline hasn't reached yet, return
    if current_time <= deadline {
        msg!("The proposal is still under voting.");
        return Err(FundError::InvalidAccountData.into());
    }

    let vote_yes = proposal_aggregator_data.proposals[vec_index as usize].votes_yes;
    let vote_no = proposal_aggregator_data.proposals[vec_index as usize].votes_no;

    let fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    let strength = fund_data.total_deposit;

    // if quorum not reached, error
    if (vote_yes + vote_no) < strength * 3 / 10 {
        msg!("Quorum not reached");
        return Err(FundError::InvalidInstruction.into());
    }

    // if proposal not in majority, error
    if vote_yes <= vote_no {
        msg!("Not enough votes favouring the trades");
        return Err(FundError::InvalidInstruction.into());
    }

    // Check mints
    let input_token_mint_key = proposal_aggregator_data.proposals[vec_index as usize].from_assets[swap_number as usize];
    let output_token_mint_key = proposal_aggregator_data.proposals[vec_index as usize].to_assets[swap_number as usize];
    if input_token_mint_key != *input_token_mint.key || output_token_mint_key != *output_token_mint.key {
        msg!("Wrong Mints");
        return Err(FundError::InvalidMints.into());
    }

    // check fund account
    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    if *fund_account_info.key != fund_pda || fund_pda != proposal_aggregator_data.fund {
        msg!("Wrong Fund details");
        return Err(FundError::InvalidFundDetails.into());
    }

    // verify vault account
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault", fund_pda.as_ref()], program_id);
    if *vault_account_info.key != vault_pda {
        msg!("Wring vault account");
        return Err(FundError::InvaildVaultAccount.into());
    }

    // verify vault account
    // let (rent_pda, _rent_bump) = Pubkey::find_program_address(&[b"rent"], program_id);
    // if *rent_account_info.key != rent_pda {
    //     msg!("Wrong Rent account details");
    //     return Err(FundError::InvalidRentAccount.into());
    // }

    // verify the vault's token accounts
    let input_vault_token_account = spl_associated_token_account::get_associated_token_address(
        &vault_pda,
        &input_token_mint_key
    );
    if *input_token_account.key != input_vault_token_account || input_token_account.data_is_empty() {
        return Err(FundError::InvalidTokenAccount.into());
    }

    let output_vault_token_account = spl_associated_token_account::get_associated_token_address(
        &vault_pda,
        &output_token_mint_key
    );
    if *output_token_account.key != output_vault_token_account {
        return Err(FundError::InvalidTokenAccount.into());
    }

    // if vault's output token account doesn't exist, create it
    if output_token_account.data_is_empty() {
        msg!("Creating Vault ATA...");

        invoke_signed(
            &create_associated_token_account(
                payer_info.key,
                vault_account_info.key,
                output_token_mint.key,
                token_program_std_info.key
            ),
            &[
                payer_info.clone(),
                output_token_account.clone(),
                vault_account_info.clone(),
                output_token_mint.clone(),
                system_program_info.clone(),
                token_program_std_info.clone(),
                ata_program_info.clone(),
                rent_sysvar_info.clone(),
            ],
            &[&[b"vault", fund_account_info.key.as_ref(), &[vault_bump]]]
        )?;
    }

    let amount = proposal_aggregator_data.proposals[vec_index as usize].amounts[swap_number as usize];
    let slippage = proposal_aggregator_data.proposals[vec_index as usize].slippage[swap_number as usize];

    let discriminator: &[u8] = &[0x2b, 0x04, 0xed, 0x0b, 0x1a, 0xc9, 0x1e, 0x62];
    msg!("discriminator length: {}", discriminator.len());
    let min_amount_out = amount
        .checked_mul(10000u64 - (slippage as u64))
        .unwrap()
        / 10000u64;
    let other_amount_threshold = min_amount_out as u64;
    let sqrt_price_limit_x64 = 0 as u128;
    let is_base_input = true;

    let mut args_buf = Vec::with_capacity(33);
    args_buf.write_all(&amount.to_le_bytes()).unwrap();
    args_buf.write_all(&other_amount_threshold.to_le_bytes()).unwrap();
    args_buf.write_all(&(sqrt_price_limit_x64 as u64).to_le_bytes()).unwrap();
    args_buf.write_all(&((sqrt_price_limit_x64 >> 64) as u64).to_le_bytes()).unwrap();
    args_buf.write_all(&[is_base_input as u8]).unwrap();

    let mut instruction_data = discriminator.to_vec();
    instruction_data.extend(args_buf);

    let mut accounts_needed: Vec<AccountMeta> = vec![
        AccountMeta::new(vault_account_info.key.clone(), true),
        AccountMeta::new_readonly(amm_config.key.clone(), false),
        AccountMeta::new(pool_state.key.clone(), false),
        AccountMeta::new(input_token_account.key.clone(), false),
        AccountMeta::new(output_token_account.key.clone(), false),
        AccountMeta::new(input_vault_ata.key.clone(), false),
        AccountMeta::new(output_vault_ata.key.clone(), false),
        AccountMeta::new(observation_state.key.clone(), false),
        AccountMeta::new_readonly(token_program_std_info.key.clone(), false),
        AccountMeta::new_readonly(token_program_2022_info.key.clone(), false),
        AccountMeta::new_readonly(memo_program.key.clone(), false),
        AccountMeta::new(input_token_mint.key.clone(), false),
        AccountMeta::new(output_token_mint.key.clone(), false),
    ];

    let mut account_infos = vec![
        vault_account_info.clone(),
        amm_config.clone(),
        pool_state.clone(),
        input_token_account.clone(),
        output_token_account.clone(),
        input_vault_ata.clone(),
        output_vault_ata.clone(),
        observation_state.clone(),
        token_program_std_info.clone(),
        token_program_2022_info.clone(),
        memo_program.clone(),
        input_token_mint.clone(),
        output_token_mint.clone(),
    ];

    for acc in account_iter {
        accounts_needed.push(AccountMeta::new(acc.key.clone(), false));
        account_infos.push(acc.clone());
    }

    let swap_cpi_instruction = Instruction {
        program_id: *raydium_clmm_program.key,
        accounts: accounts_needed,
        data: instruction_data
    };

    invoke_signed(&swap_cpi_instruction, &account_infos, &[&[b"vault", fund_account_info.key.as_ref(), &[vault_bump]]])?;

    proposal_aggregator_data.proposals[vec_index as usize].executed = true;
    proposal_aggregator_data.serialize(&mut &mut proposal_aggregator_info.data.borrow_mut()[..])?;

    msg!("[FUND-ACTIVITY] {} {} Proposal executed: ({}, {})", fund_account_info.key.to_string(), current_time, proposal_index, vec_index);

    Ok(())
}