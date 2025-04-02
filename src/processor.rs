use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    system_instruction,
    sysvar::rent::Rent
};
use crate::{
    errors::FundError, instruction::FundInstruction, state::{FundAccount, ProposalAccount, UserSpecificAccount}
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
    let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"fund", &member_pubkeys.as_ref().sort()], program_id);
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault", fund_pda.as_ref()], program_id);
    let (rent_pda, rent_bump) = Pubkey::find_program_address(&[b"rent_69"], program_id);
    if *fund_account_info.key != fund_pda || *vault_account_info.key != vault_pda || *rent_account_info.key != rent_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let rent = Rent::get()?;
    let fund_space = 73 + 32*(number_of_members as i32);
    let vault_space = 165;
    let mint_space = 82;
    let total_rent = rent.minimum_balance(fund_space) + rent.minimum_balance(vault_space) + rent.minimum_balance(mint_space);
    let rent_per_member = total_rent / u64::from(number_of_members);

    for member in &members {
        invoke(
            &system_instruction::transfer(member.key, rent_account_info.key, rent_per_member),
            &[*member.clone(), fund_account_info.clone(), system_program_info.clone()],
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
            6,
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
    if *fund_account_info.key != fund_pda || *vault_account_info.key != vault_pda {
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
