use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use crate::{
    errors::FundError, instruction::FundInstruction, state::{FundAccount, InvestmentProposalAccount, UserSpecificAccount}
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8]
) -> ProgramResult {
    let instruction = FundInstruction::unpack(data)?;
    match instruction {

        FundInstruction::InitFundAccount { number_of_members } => {
            msg!("Instruction: Init Fund Account");
            process_init_fund_account(program_id, accounts, number_of_members)
        }

        FundInstruction::InitDepositSol { amount } => {
            msg!("Instruction: Init Deposit");
            process_init_deposit_sol(program_id, accounts, amount)
        }

        FundInstruction::InitProposalInvestment { 
            amounts,
            dex_tags,
            deadline,
        } => {
            msg!("Instruction: Init Proposal");
            process_init_investment_proposal(program_id, accounts, amounts, dex_tags, deadline)
        }

        _ => Err(FundError::InvalidInstruction.into()),
    }
}

fn process_init_fund_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    number_of_members: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let governance_mint_info = next_account_info(accounts_iter)?; // Governance Mint
    let vault_account_info = next_account_info(accounts_iter)?; // Vault PDA Account
    let system_program_info = next_account_info(accounts_iter)?; // System Program
    let token_program_info = next_account_info(accounts_iter)?; // Token Program (2020)
    let fund_account_info = next_account_info(accounts_iter)?; // Fund PDA Account
    let rent_account_info = next_account_info(accounts_iter)?; // Rent PDA Account

    let members : Vec<&AccountInfo> = accounts_iter
        .take(number_of_members as usize)
        .collect();

    if members.len() != number_of_members as usize || !members.iter().all(|m| m.is_signer) {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let member_pubkeys: Vec<Pubkey> = members.iter().map(|m| *m.key).collect();
    let mut seeds: Vec<u8> = Vec::new();
    seeds.extend_from_slice(b"fund");
    // for pubkey in member_pubkeys {
    //     seeds.extend_from_slice(pubkey.as_ref());
    // }
    let seeds_array: &[u8] = &seeds;
    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[seeds_array], program_id);
    let (vault_pda, _vault_bump) = Pubkey::find_program_address(&[b"vault", fund_pda.as_ref()], program_id);
    let (rent_pda, rent_bump) = Pubkey::find_program_address(&[b"rent_69"], program_id);
    if *fund_account_info.key != fund_pda || *vault_account_info.key != vault_pda || *rent_account_info.key != rent_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let rent = Rent::get()?;
    let fund_space = 73 + 32*(number_of_members as usize);
    let vault_space = 165;
    let mint_space = 82;
    let total_rent = rent.minimum_balance(fund_space) + rent.minimum_balance(vault_space) + rent.minimum_balance(mint_space);
    let rent_per_member = total_rent / u64::from(number_of_members);

    for member in &members {
        invoke(
            &system_instruction::transfer(member.key, rent_account_info.key, rent_per_member),
            &[(*member).clone(), fund_account_info.clone(), system_program_info.clone()],
        )?;
    }

    invoke_signed(
        &system_instruction::create_account(
            rent_account_info.key,
            fund_account_info.key,
            rent.minimum_balance(fund_space),
            fund_space as u64,
            program_id
        ),
        &[rent_account_info.clone(), fund_account_info.clone(), system_program_info.clone()],
        &[&[b"rent_69", &[rent_bump]]],
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
        &[&[b"rent_69", &[rent_bump]]],
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
        &[&[b"rent_69", &[rent_bump]]],
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

    let fund_data = FundAccount {
        members: member_pubkeys,
        total_deposit: 0,
        governance_mint: *governance_mint_info.key,
        vault: *vault_account_info.key,
        is_initialized: true,
    };
    fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

    Ok(())


}

fn process_init_deposit_sol(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64
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
    let mut member_pubkeys = fund_data.members.clone();
    let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"fund", &member_pubkeys.as_ref().sort()], program_id);
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault", fund_pda.as_ref()], program_id);
    let (user_pda, user_bump) = Pubkey::find_program_address(&[b"user", fund_pda.as_ref(), member_account_info.key.as_ref()], program_id);
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
        &[&[b"fund", &member_pubkeys.sort(), &[fund_bump]]],
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
    deadline: i64
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

    let fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    let mut member_pubkeys = fund_data.members.clone();
    let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"fund", &member_pubkeys.as_ref().sort()], program_id);
    let (user_pda, user_bump) = Pubkey::find_program_address(&[b"user", fund_pda.as_ref(), proposer_account_info.key.as_ref()], program_id);
    let user_data = UserSpecificAccount::try_from_slice(&user_specific_pda_info.data.borrow())?;
    let (proposal_pda, proposal_bump) = Pubkey::find_program_address(&[b"proposal-investment", proposal_account_info.key.as_ref(), &[user_data.num_proposals]], program_id);
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

    Ok(())

}
