use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, clock::Clock, entrypoint::ProgramResult, instruction::AccountMeta, msg, program::{invoke, invoke_signed}, program_pack::Pack, pubkey::Pubkey, system_instruction, sysvar::{rent::Rent, Sysvar},
    pubkey,
    instruction::{Instruction},
};
use spl_token::state::Account as TokenAccount;
use crate::{
    errors::FundError,
    instruction::FundInstruction,
    state::{FundAccount, InvestmentProposalAccount, UserSpecificAccount, VoteAccount, MetadataInstruction}
};
use mpl_token_metadata::types::DataV2;

pub const TOKEN_METADATA_PROGRAM_ID: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8]
) -> ProgramResult {
    let instruction = FundInstruction::unpack(data)?;
    match instruction {

        FundInstruction::InitFundAccount { fund_name} => {
            msg!("Instruction: Init Fund Account");
            process_init_fund_account(program_id, accounts, fund_name)
        }

        FundInstruction::AddFundMember { fund_name } => {
            msg!("Instruction: Add Fund Member");
            process_add_member(program_id, accounts, fund_name)
        }

        FundInstruction::InitDepositSol { amount , fund_name} => {
            msg!("Instruction: Init Deposit");
            process_init_deposit_sol(program_id, accounts, amount, fund_name)
        }

        FundInstruction::InitProposalInvestment { 
            amounts,
            dex_tags,
            deadline,
            fund_name,
        } => {
            msg!("Instruction: Init Proposal");
            process_init_investment_proposal(program_id, accounts, amounts, dex_tags, deadline, fund_name)
        }

        FundInstruction::Vote {vote, fund_name} => {
            msg!("Instruction: Voting on Proposal");
            process_vote_on_proposal(program_id, accounts, vote, fund_name)
        }

        FundInstruction::InitRentAccount {  } => {
            msg!("Instruction: Init Rent Account");
            process_init_rent_account(program_id, accounts)
        }

        _ => Err(FundError::InvalidInstruction.into()),
    }
}

fn process_init_fund_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: Vec<u8>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let governance_mint_info = next_account_info(accounts_iter)?; // Governance Mint
    let vault_account_info = next_account_info(accounts_iter)?; // Vault PDA Account
    let system_program_info = next_account_info(accounts_iter)?; // System Program
    let token_program_info = next_account_info(accounts_iter)?; // Token Program (2020)
    let fund_account_info = next_account_info(accounts_iter)?; // Fund PDA Account
    let rent_account_info = next_account_info(accounts_iter)?; // Rent PDA Account
    let creator_wallet_info = next_account_info(accounts_iter)?; // Creator Wallet Address
    let metadata_account_info = next_account_info(accounts_iter)?;
    let rent_sysvar_info = next_account_info(accounts_iter)?;

    if !creator_wallet_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", &fund_name], program_id);
    let (vault_pda, _vault_bump) = Pubkey::find_program_address(&[b"vault", fund_pda.as_ref()], program_id);
    let (rent_pda, rent_bump) = Pubkey::find_program_address(&[b"rent"], program_id);
    if *fund_account_info.key != fund_pda || *vault_account_info.key != vault_pda || *rent_account_info.key != rent_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let rent = Rent::get()?;
    let fund_space = 106 + 32 + 32;
    let vault_space = 165;
    let mint_space = 82;
    let total_rent = rent.minimum_balance(fund_space) + rent.minimum_balance(vault_space) + rent.minimum_balance(mint_space);
    let fee_rent_total = total_rent + total_rent/10;

    invoke(
        &system_instruction::transfer(
            creator_wallet_info.key,
            rent_account_info.key,
            fee_rent_total
        ),
        &[creator_wallet_info.clone(), rent_account_info.clone(), system_program_info.clone()],
    )?;

    invoke_signed(
        &system_instruction::create_account(
            rent_account_info.key,
            fund_account_info.key,
            rent.minimum_balance(fund_space),
            fund_space as u64,
            program_id
        ),
        &[rent_account_info.clone(), fund_account_info.clone(), system_program_info.clone()],
        &[&[b"rent", &[rent_bump]]],
    )?;

    invoke_signed(
        &system_instruction::create_account(
            rent_account_info.key,
            vault_account_info.key,
            rent.minimum_balance(vault_space),
            vault_space as u64,
            token_program_info.key
        ),
        &[rent_account_info.clone(), vault_account_info.clone(), system_program_info.clone()],
        &[&[b"rent", &[rent_bump]]],
    )?;

    invoke_signed(
        &system_instruction::create_account(
            rent_account_info.key,
            governance_mint_info.key,
            rent.minimum_balance(mint_space),
            mint_space as u64,
            token_program_info.key,
        ),
        &[rent_account_info.clone(), governance_mint_info.clone(), system_program_info.clone()],
        &[&[b"rent", &[rent_bump]]],
    )?;
    invoke(
        &spl_token::instruction::initialize_mint(
            token_program_info.key,
            governance_mint_info.key,
            fund_account_info.key,
            None,
            9,
        )?,
        &[governance_mint_info.clone(), token_program_info.clone()],
    )?;

    let (metadata_pda, _bump) = Pubkey::find_program_address(
        &[
            b"metadata",
            TOKEN_METADATA_PROGRAM_ID.as_ref(),
            governance_mint_info.key.as_ref(),
        ],
        program_id,
    );

    let name = String::from_utf8(fund_name.clone()).unwrap();
    let symbol = String::from("TKN");
    let uri = String::from(" ");

    let ix_data = MetadataInstruction::CreateMetadataAccountsV3 {
        data: DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        is_mutable: false,
        update_authority_is_signer: true,
        collection_details: None,
    }.try_to_vec().unwrap();

    let accounts_for_mint = vec![
        AccountMeta::new(metadata_pda, false),
        AccountMeta::new(*governance_mint_info.key, false),
        AccountMeta::new(*rent_account_info.key, true),
        AccountMeta::new(*rent_account_info.key, true),
        AccountMeta::new(*rent_account_info.key, true),
        AccountMeta::new_readonly(*system_program_info.key, false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::ID, false),
    ];

    let ix = Instruction {
        program_id: TOKEN_METADATA_PROGRAM_ID,
        accounts: accounts_for_mint,
        data: ix_data,
    };

    invoke_signed(
        &ix,
        &[
            metadata_account_info.clone(),
            governance_mint_info.clone(),
            rent_account_info.clone(), // PDA signer
            rent_account_info.clone(),
            rent_account_info.clone(),
            system_program_info.clone(),
            rent_sysvar_info.clone(),
        ],
        &[&[b"rent", &[rent_bump]]]
    )?;

    let mut dex_program_ids: Vec<(u8, Pubkey)> = Vec::new();
    dex_program_ids.push((1, *fund_account_info.key));

    let mut member_pubkeys: Vec<Pubkey> = Vec::new();
    member_pubkeys.push(*creator_wallet_info.key);

    let fund_data = FundAccount {
        creator: *creator_wallet_info.key,
        members: member_pubkeys,
        total_deposit: 0,
        governance_mint: *governance_mint_info.key,
        vault: *vault_account_info.key,
        is_initialized: true,
        dex_program_ids,
    };
    fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

    Ok(())


}

fn process_add_member(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: Vec<u8>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let fund_account_info = next_account_info(accounts_iter)?;
    let member_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !member_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"fund", &fund_name], program_id);

    if *fund_account_info.key != fund_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let mut fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;

    if fund_data.members.contains(member_account_info.key) {
        msg!("User is already a member");
        return Ok(())
    }

    let current_size = fund_account_info.data_len();
    let new_size = current_size + 32;

    let rent = Rent::get()?;
    let new_min_balance = rent.minimum_balance(new_size);
    let current_balance = fund_account_info.lamports();

    if new_min_balance > current_balance {
        invoke(
            &system_instruction::transfer(
                member_account_info.key,
                fund_account_info.key,
                new_min_balance - current_balance,
            ),
            &[member_account_info.clone(), fund_account_info.clone(), system_program_info.clone()],
        )?;
    }

    invoke_signed(
        &system_instruction::allocate(fund_account_info.key, new_size as u64),
        &[fund_account_info.clone(), system_program_info.clone()],
        &[&[b"fund", &fund_name, &[fund_bump]]]
    )?;
    
    fund_data.members.push(*member_account_info.key);
    fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

    Ok(())
}

fn process_init_deposit_sol(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    fund_name: Vec<u8>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let governance_mint_info = next_account_info(accounts_iter)?;
    let vault_account_info = next_account_info(accounts_iter)?;
    let fund_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    let token_program_info = next_account_info(accounts_iter)?;
    let governance_token_account_info = next_account_info(accounts_iter)?;
    let member_account_info = next_account_info(accounts_iter)?;
    let user_specific_pda_info = next_account_info(accounts_iter)?;

    if !member_account_info.is_signer {
        msg!("Required Signer not found");
        return Err(FundError::MissingRequiredSignature.into());
    }

    let fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"fund", &fund_name], program_id);
    let (vault_pda, _vault_bump) = Pubkey::find_program_address(&[b"vault", fund_pda.as_ref()], program_id);
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", fund_pda.as_ref(), member_account_info.key.as_ref()], program_id);
    if *fund_account_info.key != fund_pda || *vault_account_info.key != vault_pda || *user_specific_pda_info.key != user_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    if fund_data.governance_mint != *governance_mint_info.key {
        return Err(FundError::InvalidGovernanceMint.into());
    }

    let expected_ata = spl_associated_token_account::get_associated_token_address(
        member_account_info.key,
        governance_mint_info.key,
    );
    if *governance_token_account_info.key != expected_ata {
        return Err(FundError::InvalidTokenAccount.into());
    }

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
            ]
        )?;
    }

    invoke(
        &system_instruction::transfer(
            member_account_info.key,
            vault_account_info.key,
            amount,
        ),
        &[
            member_account_info.clone(),
            vault_account_info.clone(),
            system_program_info.clone(),
        ]
    )?;

    invoke_signed(
        &spl_token::instruction::mint_to(
            token_program_info.key,
            governance_mint_info.key,
            governance_token_account_info.key,
            fund_account_info.key,
            &[],
            amount,
        )?,
        &[
            governance_mint_info.clone(),
            governance_token_account_info.clone(),
            fund_account_info.clone(),
            token_program_info.clone(),
        ],
        &[&[b"fund", &fund_name, &[fund_bump]]],
    )?;

    let rent = Rent::get()?;
    let user_space = 50;
    let total_rent = rent.minimum_balance(user_space);

    invoke(
        &system_instruction::create_account(
            member_account_info.key,
            user_specific_pda_info.key,
            total_rent,
            user_space as u64,
            program_id
        ),
        &[
            member_account_info.clone(),
            user_specific_pda_info.clone(),
            system_program_info.clone(),
        ],
    )?;

    let mut fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    fund_data.total_deposit += amount;
    fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

    let mut user_specific_data = UserSpecificAccount::try_from_slice(&user_specific_pda_info.data.borrow())?;
    user_specific_data.deposit += amount;
    user_specific_data.governance_token_balance += amount;
    user_specific_data.pubkey = *user_specific_pda_info.key;
    user_specific_data.is_active = true;
    user_specific_data.serialize(&mut &mut user_specific_pda_info.data.borrow_mut()[..])?;

    Ok(())
}

fn process_init_investment_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amounts: Vec<u64>,
    dex_tags: Vec<u8>,
    deadline: i64,
    fund_name: Vec<u8>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let proposer_account_info = next_account_info(accounts_iter)?;
    let user_specific_pda_info = next_account_info(accounts_iter)?;
    let fund_account_info = next_account_info(accounts_iter)?;
    let proposal_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !proposer_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", &fund_name], program_id);
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", fund_pda.as_ref(), proposer_account_info.key.as_ref()], program_id);
    let mut user_data = UserSpecificAccount::try_from_slice(&user_specific_pda_info.data.borrow())?;
    let (proposal_pda, _proposal_bump) = Pubkey::find_program_address(&[b"proposal-investment", proposer_account_info.key.as_ref(), &[user_data.num_proposals]], program_id);
    if *fund_account_info.key != fund_pda || *user_specific_pda_info.key != user_pda || *proposal_account_info.key != proposal_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let rent = Rent::get()?;
    let proposal_space = 43 + amounts.len()*73;
    let total_rent = rent.minimum_balance(proposal_space);

    invoke(
        &system_instruction::create_account(
            proposer_account_info.key,
            proposal_account_info.key,
            total_rent,
            proposal_space as u64,
            program_id
        ),
        &[
            proposal_account_info.clone(),
            proposer_account_info.clone(),
            system_program_info.clone(),
        ],
    )?;

    let from_assets_info : Vec<&AccountInfo> = accounts_iter
        .take(amounts.len())
        .collect();

    if from_assets_info.len() != amounts.len() {
        return Err(FundError::InvalidAccountData.into());
    }

    let from_assets_mints: Vec<Pubkey> = from_assets_info.iter().map(|m| *m.key).collect();

    let to_assets_info: Vec<&AccountInfo> = accounts_iter
        .take(amounts.len())
        .collect();

    if to_assets_info.len() != amounts.len() {
        return Err(FundError::InvalidAccountData.into());
    }

    let to_assets_mints: Vec<Pubkey> = to_assets_info.iter().map(|m| *m.key).collect();

    let proposal_data = InvestmentProposalAccount {
        proposer: *proposer_account_info.key,
        from_assets: from_assets_mints,
        to_assets: to_assets_mints,
        amounts,
        dex_tags,
        deadline,
        votes_yes: 0,
        votes_no: 0,
        executed: false
    };
    proposal_data.serialize(&mut &mut proposal_account_info.data.borrow_mut()[..])?;
    user_data.num_proposals += 1;
    user_data.serialize(&mut &mut user_specific_pda_info.data.borrow_mut()[..])?;

    Ok(())

}

fn process_vote_on_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vote: u8,
    fund_name: Vec<u8>,
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let voter_account_info = next_account_info(accounts_iter)?;
    let vote_account_info = next_account_info(accounts_iter)?;
    let proposal_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    let user_specific_pda_info = next_account_info(accounts_iter)?;
    let fund_account_info = next_account_info(accounts_iter)?;
    let governance_token_mint_info = next_account_info(accounts_iter)?;
    let voter_token_account_info = next_account_info(accounts_iter)?;

    if !voter_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", &fund_name], program_id);
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", fund_pda.as_ref(), voter_account_info.key.as_ref()], program_id);
    let user_data = UserSpecificAccount::try_from_slice(&user_specific_pda_info.data.borrow())?;
    let (proposal_pda, _proposal_bump) = Pubkey::find_program_address(&[b"proposal-investment", voter_account_info.key.as_ref(), &[user_data.num_proposals]], program_id);
    let (vote_pda, _vote_bump) = Pubkey::find_program_address(&[b"vote", voter_account_info.key.as_ref(), proposal_pda.as_ref()], program_id);
    let token_account = spl_associated_token_account::get_associated_token_address(
        voter_account_info.key,
        governance_token_mint_info.key
    );

    if *fund_account_info.key != fund_pda ||
        *user_specific_pda_info.key != user_pda ||
        *proposal_account_info.key != proposal_pda ||
        *vote_account_info.key != vote_pda ||
        token_account != *voter_token_account_info.key {
        return Err(FundError::InvalidAccountData.into());
    }

    let fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    if fund_data.governance_mint != *governance_token_mint_info.key {
        return Err(FundError::InvalidGovernanceMint.into());
    }

    let mut proposal_data = InvestmentProposalAccount::try_from_slice(&proposal_account_info.data.borrow())?;
    if proposal_data.deadline < current_time {
        return Err(FundError::VotingCeased.into());
    }

    let rent = Rent::get()?;
    let vote_space = 33 as usize;
    let total_rent = rent.minimum_balance(vote_space);

    if vote_account_info.data_is_empty() {
        invoke(
            &system_instruction::create_account(
                voter_account_info.key,
                vote_account_info.key,
                total_rent,
                vote_space as u64,
                program_id
            ),
            &[
                voter_account_info.clone(),
                vote_account_info.clone(),
                system_program_info.clone(),
            ]
        )?;

        let token_account_data = TokenAccount::unpack(&voter_token_account_info.data.borrow())?;
        let voting_power = token_account_data.amount;
        
        if vote == 1 {
            proposal_data.votes_yes += voting_power;
        } else {
            proposal_data.votes_no += voting_power;
        }

        proposal_data.serialize(&mut &mut proposal_account_info.data.borrow_mut()[..])?;

        let vote_data = VoteAccount {
            voter: *voter_account_info.key,
            vote,
        };

        vote_data.serialize(&mut &mut vote_account_info.data.borrow_mut()[..])?;
    } else {
        return Err(FundError::AlreadyVoted.into());
    }

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

    let (rent_pda, _rent_bump) = Pubkey::find_program_address(&[b"rent"], program_id);

    if *rent_account_info.key != rent_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let rent = Rent::get()?;
    let data_len = 0 as usize;
    let rent_exemption_amount = rent.minimum_balance(data_len);

    invoke(
        &system_instruction::create_account(
            god_father_info.key,
            rent_account_info.key,
            rent_exemption_amount,
            data_len as u64,
            program_id
        ),
        &[god_father_info.clone(), rent_account_info.clone(), system_program_info.clone()],
    )?;

    Ok(())
}

/* fn process_execute_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: Vec<u8>,
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let proposal_account_info = next_account_info(accounts_iter)?;
    let fund_account_info = next_account_info(accounts_iter)?;
    let vault_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    let token_program_info = next_account_info(accounts_iter)?;
}
*/
