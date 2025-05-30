use borsh::{BorshDeserialize, BorshSerialize};
use solana_program:: pubkey;
use solana_program::{
    account_info::{next_account_info, AccountInfo}, clock::Clock, entrypoint::ProgramResult, msg, program::{invoke, invoke_signed}, program_pack::Pack, pubkey:: Pubkey, system_instruction, sysvar::{rent::Rent, Sysvar}
    // instruction::{Instruction},
};
use spl_token::state::Account as TokenAccount;
use spl_associated_token_account::instruction::create_associated_token_account;
use crate::{
    errors::FundError,
    instruction::FundInstruction,
    state::{FundAccount, InvestmentProposalAccount, UserAccount, UserSpecificAccount, VaultAccount, VoteAccount}
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

        FundInstruction::InitFundAccount { privacy, fund_name} => {
            msg!("Instruction: Init Fund Account");
            process_init_fund_account(program_id, accounts, fund_name, privacy)
        }

        FundInstruction::InitUserAccount {  } => {
            msg!("Instruction: Init User Account");
            process_init_user_account(program_id, accounts)
        }

        FundInstruction::AddFundMember { fund_name } => {
            msg!("Instruction: Add Fund Member");
            process_add_member(program_id, accounts, fund_name)
        }

        // FundInstruction::InitDepositSol { amount , fund_name} => {
        //     msg!("Instruction: Init Deposit");
        //     process_init_deposit_sol(program_id, accounts, amount, fund_name)
        // }

        FundInstruction::InitDepositToken { amount, mint_amount, fund_name } => {
            msg!("Instruction: Init Deposit Token");
            process_init_deposit_token(program_id, accounts, amount, mint_amount, fund_name)
        }

        FundInstruction::InitProposalInvestment { 
            amounts,
            // dex_tags,
            deadline,
            fund_name,
        } => {
            msg!("Instruction: Init Proposal");
            process_init_investment_proposal(program_id, accounts, amounts, deadline, fund_name)
        }

        FundInstruction::Vote {vote, fund_name} => {
            msg!("Instruction: Voting on Proposal");
            process_vote_on_proposal(program_id, accounts, vote, fund_name)
        }

        FundInstruction::InitRentAccount {  } => {
            msg!("Instruction: Init Rent Account");
            process_init_rent_account(program_id, accounts)
        }

        FundInstruction::LeaveFund { fund_name } => {
            msg!("Instruction: Leave Fund");
            process_leave_fund(program_id, fund_name, accounts)
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
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let governance_mint_info = next_account_info(accounts_iter)?; // Governance Mint
    let vault_account_info = next_account_info(accounts_iter)?; // Vault PDA Account
    let system_program_info = next_account_info(accounts_iter)?; // System Program
    let token_program_info = next_account_info(accounts_iter)?; // Token Program (2020)
    let fund_account_info = next_account_info(accounts_iter)?; // Fund PDA Account
    let creator_wallet_info = next_account_info(accounts_iter)?; // Creator Wallet Address
    let metadata_account_info = next_account_info(accounts_iter)?; // Metadat PDA for Governance Mint
    let rent_sysvar_info = next_account_info(accounts_iter)?; // Rent Sysvar
    let token_metadata_program_info = next_account_info(accounts_iter)?; // Token Metadata Program
    let user_account_info = next_account_info(accounts_iter)?; // Global User Account
    let user_specific_info = next_account_info(accounts_iter)?;

    // Creator should be signer
    if !creator_wallet_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    // Deriving required PDAs
    let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault", fund_pda.as_ref()], program_id);
    let (governance_mint, governance_bump) = Pubkey::find_program_address(&[b"governance", fund_pda.as_ref()], program_id);
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", creator_wallet_info.key.as_ref()], program_id);

    // Check if any of the provided PDA differes from the derived
    if *fund_account_info.key != fund_pda ||
       *vault_account_info.key != vault_pda ||
       *governance_mint_info.key != governance_mint ||
       *user_account_info.key != user_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    // Check if the an account already exists on that PDA
    if fund_account_info.lamports() > 0 {
        return Err(FundError::InvalidAccountData.into());
    }

    // Calculate Rent
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let fund_space = 154 as usize;
    let vault_space = 40 as usize;
    let mint_space = spl_token::state::Mint::LEN;

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
            token_program_info.key,
        ),
        &[creator_wallet_info.clone(), governance_mint_info.clone(), system_program_info.clone()],
        &[&[b"governance", fund_pda.as_ref(), &[governance_bump]]],
    )?;
    invoke_signed(
        &spl_token::instruction::initialize_mint(
            token_program_info.key,
            governance_mint_info.key,
            fund_account_info.key,
            None,
            0,
        )?,
        &[governance_mint_info.clone(), token_program_info.clone(), rent_sysvar_info.clone()],
        &[&[b"governance", fund_pda.as_ref(), &[governance_bump]]],
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
    let mut array = [0u8; 32];
    let len = bytes.len().min(32);
    array[..len].copy_from_slice(&bytes[..len]);

    // Deserialization and Serialization of Fund data
    let fund_data = FundAccount {
        name: array,
        creator: *creator_wallet_info.key,
        members: 1 as u64,
        total_deposit: 0 as u64,
        governance_mint: *governance_mint_info.key,
        vault: *vault_account_info.key,
        is_initialized: true,
        created_at: current_time,
        is_private: privacy,
    };
    fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

    // Deserializing the User Global PDA
    let mut user_data = UserAccount::try_from_slice(&user_account_info.data.borrow())?;

    // Check is User is already in the Fund (vague since fund account is just created)
    if user_data.funds.contains(fund_account_info.key) {
        msg!("User is already a member");
        return Ok(())
    }

    // Calculate current size and new size of User PDA
    let current_size = user_account_info.data_len();
    let new_size = current_size + 32;

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
    // Add the new fund address to user funds
    user_data.funds.push(*fund_account_info.key);
    user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;

    // Deserialization and Serialization of Vault Account Data
    let vault_data = VaultAccount {
        fund: *fund_account_info.key,
        last_deposit_time: 0,
    };
    vault_data.serialize(&mut &mut vault_account_info.data.borrow_mut()[..])?;

    msg!("Fund Initialization successful");

    create_user_specific_pda(
        program_id,
        creator_wallet_info,
        system_program_info,
        fund_account_info,
        user_specific_info
    )
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

    // Initiallt user is joined in no Funds
    let funds:Vec<Pubkey> = vec![];

    // Deserialization and Serialization of User Account Data
    let user_data = UserAccount {
        user: *creator_account_info.key,
        funds,
    };
    user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;
    msg!("User Account created successfully");

    Ok(())

}

fn process_add_member<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    fund_name: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let fund_account_info = next_account_info(accounts_iter)?; // Fund Account
    let member_account_info = next_account_info(accounts_iter)?; // USer to be added 
    let system_program_info = next_account_info(accounts_iter)?; // System Program
    let user_account_info = next_account_info(accounts_iter)?; // User Global identity account
    let user_specific_info = next_account_info(accounts_iter)?; // User Specific pda

    // User should be signer
    if !member_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    // Derive PDAs and check if it is same as provided in accounts
    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", member_account_info.key.as_ref()], program_id);
    if *fund_account_info.key != fund_pda || *user_account_info.key != user_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    // Check if Fund exist or not!
    if fund_account_info.data_is_empty() {
        return Err(FundError::InvalidAccountData.into());
    }

    // Deserialize User Data and check if User is already a member of provided Fund
    let mut user_data = UserAccount::try_from_slice(&user_account_info.data.borrow())?;
    if user_data.funds.contains(fund_account_info.key) {
        msg!("User is already a member");
        return Ok(());
    }

    // Calculate if more lamports are needed for reallocation of User Global Account
    let current_size = user_account_info.data_len();
    let new_size = current_size + 32;
    let rent = Rent::get()?;
    let new_min_balance = rent.minimum_balance(new_size);
    let current_balance = user_account_info.lamports();
    if new_min_balance > current_balance {
        invoke(
            &system_instruction::transfer(
                member_account_info.key,
                user_account_info.key,
                new_min_balance - current_balance,
            ),
            &[member_account_info.clone(), user_account_info.clone(), system_program_info.clone()],
        )?;
    }

    // invoke(
    //     &system_instruction::transfer(
    //         member_account_info.key,
    //         user_account_info.key,
    //         400_000_000,
    //     ),
    //     &[member_account_info.clone(), user_account_info.clone(), system_program_info.clone()]
    // )?;

    // Reallocate new bytes ofr storage of new Fund addresss
    user_account_info.realloc(new_size, false)?;
    user_data.funds.push(*fund_account_info.key);
    user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;

    // Deserialize the fund data
    let mut fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    let n = fund_data.members;
    let refund = (4593600 as u64)/(n*(n+1));
    // Refund to existing members to equally distribute the rent fee
    for _i in 0..n {
        let receiver_account_info = next_account_info(accounts_iter)?;
        invoke(
            &system_instruction::transfer(
                member_account_info.key,
                receiver_account_info.key,
                refund,
            ),
            &[member_account_info.clone(), receiver_account_info.clone(), system_program_info.clone()],
        )?;
    }

    fund_data.members += 1;
    fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

    create_user_specific_pda(
        program_id,
        member_account_info,
        system_program_info,
        fund_account_info,
        user_specific_info
    )
    // Ok(())

}

// fn process_init_deposit_sol(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     amount: u64,
//     fund_name: String,
// ) -> ProgramResult {
//     let current_time = Clock::get()?.unix_timestamp;

//     let accounts_iter = &mut accounts.iter();
//     let governance_mint_info = next_account_info(accounts_iter)?; // Governance Mint Account
//     let vault_account_info = next_account_info(accounts_iter)?; // Vault PDA
//     let fund_account_info = next_account_info(accounts_iter)?; // Fund PDA
//     let system_program_info = next_account_info(accounts_iter)?; // System Program
//     let token_program_info = next_account_info(accounts_iter)?; // Token Program
//     let governance_token_account_info = next_account_info(accounts_iter)?; // Governance token Account of depositor
//     let member_account_info = next_account_info(accounts_iter)?; // Depositor Wallet
//     let user_specific_pda_info = next_account_info(accounts_iter)?; // Depositor's Fund Specific Account
//     let rent_sysvar_info = next_account_info(accounts_iter)?; // Rent Sysvar Account
//     let associated_token_program_info = next_account_info(accounts_iter)?;

//     // Depositor needs to be signer
//     if !member_account_info.is_signer {
//         msg!("Required Signer not found");
//         return Err(FundError::MissingRequiredSignature.into());
//     }

//     // Derive PDAs and check for equality with provided ones
//     let (governance_mint, _governance_bump) = Pubkey::find_program_address(&[b"governance", fund_account_info.key.as_ref()], program_id);
//     let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
//     let (vault_pda, _vault_bump) = Pubkey::find_program_address(&[b"vault", fund_account_info.key.as_ref()], program_id);
//     let (user_specific_pda, user_specific_bump) = Pubkey::find_program_address(&[b"user", fund_account_info.key.as_ref(), member_account_info.key.as_ref()], program_id);
//     if *fund_account_info.key != fund_pda ||
//        *vault_account_info.key != vault_pda ||
//        *user_specific_pda_info.key != user_specific_pda ||
//        *governance_mint_info.key != governance_mint {
//         return Err(FundError::InvalidAccountData.into());
//     }
//     let expected_ata = spl_associated_token_account::get_associated_token_address(
//         member_account_info.key,
//         governance_mint_info.key,
//     );
//     if *governance_token_account_info.key != expected_ata {
//         return Err(FundError::InvalidTokenAccount.into());
//     }

//     // If depositor's token account doesn't exist for governance mint, then create it
//     if governance_token_account_info.data_is_empty() {
//         invoke(
//             &spl_associated_token_account::instruction::create_associated_token_account(
//                 member_account_info.key,
//                 member_account_info.key,
//                 governance_mint_info.key,
//                 token_program_info.key,
//             ),
//             &[
//                 member_account_info.clone(),
//                 governance_token_account_info.clone(),
//                 member_account_info.clone(),
//                 governance_mint_info.clone(),
//                 token_program_info.clone(),
//                 system_program_info.clone(),
//                 associated_token_program_info.clone(),
//                 rent_sysvar_info.clone(),
//             ]
//         )?;
//     }

//     // Deposit required SOL from depositor to vault account
//     invoke(
//         &system_instruction::transfer(
//             member_account_info.key,
//             vault_account_info.key,
//             amount,
//         ),
//         &[
//             member_account_info.clone(),
//             vault_account_info.clone(),
//             system_program_info.clone(),
//         ]
//     )?;

//     // Now mint equal amount of governance tokens to depositor's governance token account
//     invoke_signed(
//         &spl_token::instruction::mint_to(
//             token_program_info.key,
//             governance_mint_info.key,
//             governance_token_account_info.key,
//             fund_account_info.key,
//             &[],
//             amount,
//         )?,
//         &[
//             governance_mint_info.clone(),
//             governance_token_account_info.clone(),
//             fund_account_info.clone(),
//             token_program_info.clone(),
//             // rent_sysvar_info.clone(),
//         ],
//         &[&[b"fund", fund_name.as_bytes(), &[fund_bump]]],
//     )?;

//     // In vault account, set the last deposit time
//     let mut vault_data = VaultAccount::try_from_slice(&vault_account_info.data.borrow())?;
//     vault_data.last_deposit_time = current_time;
//     vault_data.serialize(&mut &mut vault_account_info.data.borrow_mut()[..])?;


//     // In fund account increase the deposited amount (unit lamports)
//     let mut fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
//     fund_data.total_deposit += amount;
//     fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

//     Ok(())
// }

fn process_init_deposit_token(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    mint_amount: u64,
    fund_name: String,
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let member_account_info = next_account_info(accounts_iter)?; // Depositor wallet
    let member_ata_info = next_account_info(accounts_iter)?; // Depositor's ATA for the depositing mint
    let vault_account_info = next_account_info(accounts_iter)?; // Vault PDA
    let vault_ata_info = next_account_info(accounts_iter)?; // Vault PDA's ATA for the depositing mint
    let mint_account_info = next_account_info(accounts_iter)?; // Mint account of token to be deposited
    let token_program_info = next_account_info(accounts_iter)?; // Token Progarm
    let ata_program_info = next_account_info(accounts_iter)?; // Associated Token Program
    let fund_account_info = next_account_info(accounts_iter)?; // Fund PDA
    let user_specific_pda_info = next_account_info(accounts_iter)?; // USER Specific PDA
    let system_program_info = next_account_info(accounts_iter)?; // System program
    let rent_sysvar_info = next_account_info(accounts_iter)?; // Rent Sysvar Account
    let governance_token_account_info = next_account_info(accounts_iter)?; // Governance Token Account of depositor
    let governance_mint_info = next_account_info(accounts_iter)?; // Governance Mint Account
    // let temp_wsol_account_info = next_account_info(accounts_iter)?;

    // Depositor should be signer
    if !member_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    // Derive the PDAs and check for equality with provided ones
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault", fund_account_info.key.as_ref()], program_id);
    let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let (user_specific_pda, _user_specific_bump) = Pubkey::find_program_address(&[b"user", fund_pda.as_ref(), member_account_info.key.as_ref()], program_id);
    if *vault_account_info.key != vault_pda || *fund_account_info.key != fund_pda || *user_specific_pda_info.key != user_specific_pda {
        return Err(FundError::InvalidAccountData.into());
    }
    let expected_ata = spl_associated_token_account::get_associated_token_address(
        member_account_info.key,
        governance_mint_info.key,
    );
    if *governance_token_account_info.key != expected_ata {
        return Err(FundError::InvalidTokenAccount.into());
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

    let rent =Rent::get()?;
    let rent_req = rent.minimum_balance(TokenAccount::LEN);

    // If vault's token account account for the depositing mint doesn't exist, create it
    if vault_ata_info.data_is_empty() {
        msg!("Creating Vault ATA...");

        invoke_signed(
            &create_associated_token_account(
                &vault_pda,
                member_account_info.key,
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

    Ok(())
}

fn process_init_investment_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amounts: Vec<u64>,
    // dex_tags: Vec<u8>,
    deadline: i64,
    fund_name: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let proposer_account_info = next_account_info(accounts_iter)?; // Proposer Wallet
    let user_specific_pda_info = next_account_info(accounts_iter)?; // Proposer's Fund-specific Account
    let fund_account_info = next_account_info(accounts_iter)?; // Fund Account
    let proposal_account_info = next_account_info(accounts_iter)?; // Proposal Account
    let system_program_info = next_account_info(accounts_iter)?; // System Program

    // Proposer needs to be signer
    if !proposer_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    // Derive PDAs and check for equality with the provided ones
    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", fund_pda.as_ref(), proposer_account_info.key.as_ref()], program_id);
    let mut user_data = UserSpecificAccount::try_from_slice(&user_specific_pda_info.data.borrow())?;
    let (proposal_pda, proposal_bump) = Pubkey::find_program_address(
        &[
            b"proposal-investment",
            proposer_account_info.key.as_ref(),
            &[user_data.num_proposals],
            fund_account_info.key.as_ref()
        ],
        program_id
    );
    if *fund_account_info.key != fund_pda || *user_specific_pda_info.key != user_pda || *proposal_account_info.key != proposal_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    // Rent Calculation
    let rent = Rent::get()?;
    let proposal_space = 43 + 32 + 4 + amounts.len()*73;
    let total_rent = rent.minimum_balance(proposal_space);

    // Create Proposal Account
    invoke_signed(
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
        &[&[
            b"proposal-investment",
            proposer_account_info.key.as_ref(),
            &[user_data.num_proposals],
            fund_account_info.key.as_ref(),
            &[proposal_bump]
        ]]
    )?;

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
    let proposal_data = InvestmentProposalAccount {
        fund: *fund_account_info.key,
        proposer: *proposer_account_info.key,
        from_assets: from_assets_mints,
        to_assets: to_assets_mints,
        amounts,
        // dex_tags,
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

fn process_leave_fund(
    program_id: &Pubkey,
    fund_name: String,
    accounts: &[AccountInfo],
    
) -> ProgramResult {

    // let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let member_wallet_info =next_account_info(accounts_iter)?;
    let user_specific_info = next_account_info(accounts_iter)?; // to be deleted
    let fund_account_info= next_account_info(accounts_iter)?;  // to change number of members
    let user_account_info=next_account_info(accounts_iter)?;   // to change fund details
    // let system_program_info = next_account_info(accounts_iter)?;  // to delete fund specific account
    // let voter_account_info = next_account_info(accounts_iter)?;
    // let proposal_account_info = next_account_info(accounts_iter)?;

    if !member_wallet_info.is_signer {
        return Err(FundError::InvalidAccountData.into());
    }


    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let (user_specific_pda, _user_specific_bump) = Pubkey::find_program_address(&[b"user", fund_pda.as_ref(), member_wallet_info.key.as_ref()], program_id);
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", member_wallet_info.key.as_ref()], program_id);
    let mut user_data = UserAccount::try_from_slice(&user_account_info.data.borrow())?;
    let mut fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    // let (proposal_pda, _proposal_bump) = Pubkey::find_program_address(&[b"proposal-investment", user_account_info.key.as_ref(), &[user_data.num_proposals]], program_id);
    // let proposal_data = InvestmentProposalAccount::try_from_slice(&proposal_account_info.data.borrow())?;
    if *fund_account_info.key != fund_pda ||
    *user_specific_info.key != user_specific_pda ||
    *user_account_info.key != user_pda {
        return Err(FundError::InvalidAccountData.into());
    }



    if !user_specific_info.data_is_empty(){
        let lamports = **user_specific_info.try_borrow_lamports()?;
        **member_wallet_info.try_borrow_mut_lamports()? += lamports;
        **user_specific_info.try_borrow_mut_lamports()? = 0;
    
        // let mut data = user_specific_info.try_borrow_mut_data()?;
        // for byte in data.iter_mut() {
        //     *byte = 0;
        // }
    
        msg!("User-specific fund account closed and lamports sent to user");
        
    }
    
    let current_rent = user_account_info.lamports();

    let mut flag = false;

    user_data.funds.retain(|key| {
        let keep = key != fund_account_info.key;
        if !keep {
            flag = true;
        }
        keep
    });

    if flag {

        fund_data.members-=1;

        let rent = Rent::get()?;

        let current_size = user_account_info.data_len();
        let new_size= current_size-32;
        let new_rent = rent.minimum_balance(new_size);
    
        if new_rent < current_rent {
            // let lamports = **user_account_info.try_borrow_lamports()?;
            **user_account_info.try_borrow_mut_lamports()? -= current_rent-new_rent;
            **member_wallet_info.try_borrow_mut_lamports()? += current_rent-new_rent;
        }

        user_account_info.realloc(new_size, false)?;
    }

    user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;
    fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

    Ok(())
}


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

fn create_user_specific_pda<'a>(
    program_id: &Pubkey,
    member_wallet_info: &'a AccountInfo<'a>,
    system_program_info: &'a AccountInfo<'a>,
    fund_account_info: &'a AccountInfo<'a>,
    user_specific_info: &'a AccountInfo<'a>
) -> ProgramResult {

    let current_time = Clock::get()?.unix_timestamp;

    let (user_specific_pda, user_specific_bump) = Pubkey::find_program_address(&[b"user", fund_account_info.key.as_ref(), member_wallet_info.key.as_ref()], program_id);

    if *user_specific_info.key != user_specific_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    if !user_specific_info.data_is_empty() {
        msg!("User already exists");
        return Ok(());
    }

    let rent = Rent::get()?;
    let size = 90 as usize;

    invoke_signed(
        &system_instruction::create_account(
            member_wallet_info.key,
            user_specific_info.key,
            rent.minimum_balance(size),
            size as u64,
            program_id
        ),
        &[member_wallet_info.clone(),user_specific_info.clone(),system_program_info.clone()],
        &[&[b"user", fund_account_info.key.as_ref(), member_wallet_info.key.as_ref(),&[user_specific_bump]]]
    )?;

    let mut user_data= UserSpecificAccount::try_from_slice(&user_specific_info.data.borrow())?;

        user_data.fund = *fund_account_info.key;
        user_data.is_active = true;
        user_data.join_time = current_time;
        user_data.pubkey = *member_wallet_info.key;

    user_data.serialize(&mut &mut user_specific_info.data.borrow_mut()[..])?;

    Ok(())
}