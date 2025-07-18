use std::io::Write;
use std::vec;
use sha2::{Digest, Sha256};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::{Instruction, AccountMeta};
use solana_program::pubkey;
use solana_program::{
    account_info::{next_account_info, AccountInfo}, clock::Clock, entrypoint::ProgramResult, msg, program::{invoke, invoke_signed}, program_pack::Pack, pubkey:: Pubkey, system_instruction, sysvar::{rent::Rent, Sysvar}
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token::state::Account as TokenAccount;
use spl_associated_token_account::instruction::create_associated_token_account;
// use spl_token_2022::extension::{
//     ExtensionType,
//     metadata_pointer,
// };
// use spl_token_metadata_interface;
// use spl_token_2022::state::Mint;
use crate::state::{IncrementProposalAccount, LightFundAccount, MerkleRoot, UserSpecific};
use crate::{
    errors::FundError,
    instruction::FundInstruction,
    state::{FundAccount, Proposal, ProposalAggregatorAccount, UserAccount, VaultAccount}
};

pub const TOKEN_METADATA_PROGRAM_ID: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a[AccountInfo<'a>],
    data: &[u8]
) -> ProgramResult {
    let instruction = FundInstruction::unpack(data)?;
    match instruction {

        // FundInstruction::InitFundAccount { privacy, expected_members, fund_name, symbol_str} => {
        //     msg!("Instruction: Init Fund Account");
        //     process_init_fund_account(program_id, accounts, fund_name, privacy, expected_members, symbol_str)
        // }

        FundInstruction::InitUserAccount { cid } => {
            msg!("Instruction: Init User Account");
            process_init_user_account(program_id, accounts, cid)
        }

        // FundInstruction::AddFundMember { fund_name, vec_index } => {
        //     msg!("Instruction: Add Fund Member");
        //     process_add_member(program_id, accounts, fund_name, vec_index)
        // }

        FundInstruction::InitDepositToken { is_unwrapped_sol, amount, mint_amount, fund_name, fund_type } => {
            msg!("Instruction: Init Deposit Token");
            process_init_deposit_token(program_id, accounts,is_unwrapped_sol, amount, mint_amount, fund_name, fund_type)
        }

        FundInstruction::InitProposalInvestment { cid, deadline, fund_name, merkel_bytes} => {
            msg!("Instruction: Init Proposal");
            process_init_investment_proposal(program_id, accounts, fund_name, cid, deadline, merkel_bytes)
        }

        FundInstruction::Vote {vote, proposal_index, vec_index, fund_name} => {
            msg!("Instruction: Voting on Proposal");
            process_vote_on_proposal(program_id, accounts, vote, proposal_index, vec_index, fund_name)
        }

        // FundInstruction::InitRentAccount {  } => {
        //     msg!("Instruction: Init Rent Account");
        //     process_init_rent_account(program_id, accounts)
        // }

        FundInstruction::ExecuteProposalInvestment { fund_name, proposal_index, vec_index, swap_index, no_of_swaps, merkel_proof, amount, slippage} => {
            msg!("Instruction: Execute Proposal");
            process_execute_proposal(program_id, accounts, fund_name, proposal_index, vec_index, swap_index, no_of_swaps, merkel_proof, amount, slippage)
        }

        // FundInstruction::InitJoinProposal { fund_name } => {
        //     msg!("Instruction: Init Join Proposal");
        //     process_init_join_proposal(program_id, accounts, fund_name)
        // }

        // FundInstruction::JoinVote { vote, fund_name, proposal_index } => {
        //     msg!("Instruction: Voting on Join Proposal");
        //     process_vote_on_join_proposal(program_id, accounts, vote, fund_name, proposal_index)
        // }

        // FundInstruction::CancelJoinProposal { fund_name, proposal_index } => {
        //     msg!("Instruction: Cancel Join Proposal");
        //     process_cancel_join_proposal(program_id, accounts, fund_name, proposal_index)
        // }

        FundInstruction::CancelInvestmentProposal { fund_name, proposal_index, vec_index } => {
            msg!("Instruction: Cancel Investment Proposal");
            process_cancel_investment_proposal(program_id, accounts, fund_name, proposal_index, vec_index)
        }

        FundInstruction::InitIncrementProposal { fund_name, new_size, refund_type } => {
            msg!("Instruction: Init Increment Proposal");
            process_init_increment_proposal(program_id, accounts, fund_name, new_size, refund_type)
        }

        FundInstruction::VoteOnIncrement { fund_name, vote } => {
            msg!("Instruction: Vote On Increment Proposal");
            process_vote_increment_proposal(program_id, accounts, fund_name, vote)
        }

        FundInstruction::CancelIncrementProposal { fund_name } => {
            msg!("Instruction: Cancel Increment Proposal");
            process_cancel_increment_proposal(program_id, accounts, fund_name)
        }

        FundInstruction::ToggleRefundType { fund_name, refund_type } => {
            msg!("Instruction: Toggle Refund Type");
            process_toggle_refund_type(program_id, accounts, fund_name, refund_type)
        }

        FundInstruction::InitLightFundAccount { fund_name, num_of_members, max_num_members, tags, add_members_later } => {
            msg!("Instruction: Init Light Fund Account");
            process_init_light_fund(program_id, accounts, fund_name, num_of_members, tags, add_members_later, max_num_members)
        }

        FundInstruction::HandleInvition { fund_name, response, inviter_exists } => {
            msg!("Instruction: Handle Invitations");
            process_handle_invitation(program_id, accounts, fund_name, response, inviter_exists)
        }

        FundInstruction::InviteToFund { fund_name } => {
            msg!("Instruction: Invite To Light Fund");
            process_invite_to_fund(program_id, accounts, fund_name)
        }

        FundInstruction::WithdrawOrLeaveFromLightFund { fund_name, task, stake_percent, num_of_tokens } => {
            msg!("Instruction: Withdraw from Light Fund");
            process_withdraw_or_leave_from_light_fund(program_id, accounts, fund_name, task, stake_percent, num_of_tokens)
        }

        FundInstruction::SetExecutingOrExecuted { proposal_index, vec_index, fund_name, set } => {
            msg!("Instruction: Set Proposal Executing");
            process_set_executing_or_executed(program_id, accounts, proposal_index, vec_index, fund_name, set)
        }

        _ => Err(FundError::InvalidInstruction.into()),
    }
}

fn process_init_light_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: String,
    num_of_members: u8,
    tags: u32,
    add_members_later: u8,
    max_num_members: u8,
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let creator_wallet_info = next_account_info(accounts_iter)?; // Creator Wallet Address ...................
    let creator_account_info = next_account_info(accounts_iter)?; // Global User Account .....................
    let vault_account_info = next_account_info(accounts_iter)?; // Vault PDA Account .........................
    let system_program_info = next_account_info(accounts_iter)?; // System Program ...........................
    let fund_account_info = next_account_info(accounts_iter)?; // Fund PDA Account ...........................
    let proposal_aggregator_info = next_account_info(accounts_iter)?; // first proposal aggregator ...........

    if !creator_wallet_info.is_signer {
        msg!("[FUND-ERROR] {} {} Wrong signer!(must be your wallet)", fund_account_info.key.to_string(), creator_wallet_info.key.to_string());
        return Err(FundError::MissingRequiredSignature.into());
    }

    msg!("{}", fund_name);

    let current_index = 0 as u8;
    let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"light-fund", fund_name.as_bytes()], program_id);
    let (creator_pda, _creator_bump) = Pubkey::find_program_address(&[b"user", creator_wallet_info.key.as_ref()], program_id);
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault", fund_account_info.key.as_ref()], program_id);
    let (proposal_aggregator_pda, proposal_aggregator_bump) = Pubkey::find_program_address(&[b"proposal-aggregator", &[current_index], fund_account_info.key.as_ref()], program_id);

    if *fund_account_info.key != fund_pda {
        msg!("{}", fund_account_info.key.to_string());
        msg!("{}", fund_pda.to_string());
        return Err(FundError::InvalidFundDetails.into());
    }
    if *creator_account_info.key != creator_pda {
        return Err(FundError::InvalidFundCreator.into());
    }
    if *vault_account_info.key != vault_pda {
        return Err(FundError::InvaildVaultAccount.into());
    }
    if *proposal_aggregator_info.key != proposal_aggregator_pda {
        return Err(FundError::InvalidProposalAccount.into());
    }
    if !fund_account_info.data_is_empty() {
        return Err(FundError::InvalidFundDetails.into());
    }

    let rent = Rent::get()?;
    let fund_space = 128 as usize;
    let vault_space = 8 as usize;
    let aggregator_space = 5 as usize;
    
    invoke_signed(
        &system_instruction::create_account(
            creator_wallet_info.key,
            fund_account_info.key,
            rent.minimum_balance(fund_space),
            fund_space as u64,
            program_id
        ),
        &[creator_wallet_info.clone(), fund_account_info.clone(), system_program_info.clone()],
        &[&[b"light-fund", fund_name.as_bytes(), &[fund_bump]]]
    )?;

    invoke_signed(
        &system_instruction::create_account(
            creator_wallet_info.key,
            vault_account_info.key,
            rent.minimum_balance(vault_space),
            vault_space as u64,
            program_id
        ),
        &[creator_wallet_info.clone(), vault_account_info.clone(), system_program_info.clone()],
        &[&[b"vault", fund_account_info.key.as_ref(), &[vault_bump]]]
    )?;

    invoke_signed(
        &system_instruction::create_account(
            creator_wallet_info.key,
            proposal_aggregator_info.key,
            rent.minimum_balance(aggregator_space),
            aggregator_space as u64,
            program_id
        ),
        &[creator_wallet_info.clone(), proposal_aggregator_info.clone(), system_program_info.clone()],
        &[&[b"proposal-aggregator", &[current_index], fund_account_info.key.as_ref(), &[proposal_aggregator_bump]]]
    )?;

    // Converting the fund_name to an array of u8 of fixed size 32
    let bytes = fund_name.as_bytes();
    let mut array = [0u8; 32];
    let len = bytes.len().min(32);
    array[..len].copy_from_slice(&bytes[..len]);

    let members: Vec<(Pubkey, u32)> = vec![(*creator_wallet_info.key, 0 as u32)];
    if num_of_members == 0 && max_num_members < 1 {
        return Err(FundError::InvalidMemberInfo.into());
    }
    if num_of_members != 0 && max_num_members != 20 {
        return Err(FundError::InvalidMemberInfo.into());
    }

    let fund_data = LightFundAccount {
        name: array,
        fund_type: 0 as u8,
        creator_exists: true,
        total_deposit: 0 as u64,
        vault: *vault_account_info.key,
        current_proposal_index: current_index,
        created_at: current_time,
        tags,
        max_members: max_num_members,
        members
    };

    fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

    let vault_data = VaultAccount {
        last_deposit_time: current_time
    };

    vault_data.serialize(&mut &mut vault_account_info.data.borrow_mut()[..])?;

    let proposals: Vec<Proposal> = vec![];

    let proposal_data = ProposalAggregatorAccount {
        index: current_index,
        proposals
    };

    proposal_data.serialize(&mut &mut proposal_aggregator_info.data.borrow_mut()[..])?;

    let mut creator_data = UserAccount::try_from_slice(&creator_account_info.data.borrow())?;
    creator_data.funds.push(UserSpecific {
        fund: *fund_account_info.key,
        fund_type: 0 as u8,
        governance_token_balance: 0 as u64,
        is_pending: false,
        is_eligible: 0 as u8,
        inviter_index: 0 as u32,
        join_time: current_time
    });

    let current_creator_space = creator_account_info.data_len();
    let new_creator_space = current_creator_space + 55;
    let current_creator_rent = creator_account_info.lamports();
    let new_creator_rent = rent.minimum_balance(new_creator_space);

    if new_creator_rent > current_creator_rent {
        invoke(
            &system_instruction::transfer(
                creator_wallet_info.key,
                creator_account_info.key,
                new_creator_rent - current_creator_rent
            ),
            &[creator_wallet_info.clone(), creator_account_info.clone(), system_program_info.clone()]
        )?;
    }

    creator_account_info.realloc(new_creator_space, false)?;
    creator_data.serialize(&mut &mut creator_account_info.data.borrow_mut()[..])?;

    if add_members_later == 1 && max_num_members >= 1 {
        return Ok(());
    } else if add_members_later == 1 && max_num_members < 1 {
        return Err(FundError::InvalidMemberInfo.into());
    } else if add_members_later == 0 && num_of_members == 0 {
        return Err(FundError::InvalidMemberInfo.into());
    } else if add_members_later == 0 && max_num_members != 20 {
        return Err(FundError::InvalidMemberInfo.into());
    }

    let members_info: Vec<&AccountInfo> = accounts_iter
        .take(num_of_members as usize)
        .collect();
    if members_info.len() != num_of_members as usize {
        return Err(FundError::InvalidAccountData.into());
    }
    let members_pubkey: Vec<Pubkey> = members_info.iter().map(|m| *m.key).collect();

    let members_pda_info: Vec<&AccountInfo> = accounts_iter
        .take(num_of_members as usize)
        .collect();
    if members_pda_info.len() != num_of_members as usize {
        return Err(FundError::InvalidAccountData.into());
    }
    let members_pda_pubkey: Vec<Pubkey> = members_pda_info.iter().map(|m| *m.key).collect();

    for i in 0..num_of_members {
        let (pda, _bump) = Pubkey::find_program_address(&[b"user", members_pubkey[i as usize].as_ref()], program_id);
        if members_pda_pubkey[i as usize] != pda || members_pda_info[i as usize].data_is_empty() {
            return Err(FundError::InvalidMemberInfo.into());
        }
    }

    for i in 0..num_of_members {
        let member_pda_info = members_pda_info[i as usize];
        let mut pda_data = UserAccount::try_from_slice(&member_pda_info.data.borrow())?;
        pda_data.funds.push(UserSpecific {
            fund: *fund_account_info.key,
            fund_type: 0 as u8,
            governance_token_balance: 0 as u64,
            is_pending: true,
            is_eligible: 1 as u8,
            inviter_index: 0 as u32,
            join_time: current_time
        });
        
        let current_user_size = member_pda_info.data_len();
        let new_user_size = current_user_size + 55;
        let current_user_rent = member_pda_info.lamports();
        let new_user_rent = rent.minimum_balance(new_user_size);

        if new_user_rent > current_user_rent {
            invoke(
                &system_instruction::transfer(
                    creator_wallet_info.key,
                    member_pda_info.key,
                    new_user_rent - current_user_rent
                ),
                &[creator_wallet_info.clone(), member_pda_info.clone(), system_program_info.clone()]
            )?;
        }

        member_pda_info.realloc(new_user_size, false)?;

        pda_data.serialize(&mut &mut member_pda_info.data.borrow_mut()[..])?;
    }

    Ok(())
}

fn process_invite_to_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: String,
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let inviter_wallet_info = next_account_info(accounts_iter)?;
    let joiner_wallet_info = next_account_info(accounts_iter)?;
    let joiner_account_info = next_account_info(accounts_iter)?;
    let fund_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;

    if !inviter_wallet_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"light-fund", fund_name.as_bytes()], program_id);
    let (joiner_pda, _joiner_bump) = Pubkey::find_program_address(&[b"user", joiner_wallet_info.key.as_ref()], program_id);

    if *fund_account_info.key != fund_pda || *joiner_account_info.key != joiner_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let fund_data = LightFundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    let inviter_exists = fund_data
        .members
        .iter()
        .any(|member| member.0 == *inviter_wallet_info.key);

    if !inviter_exists {
        return Err(FundError::NotAFundMember.into());
    }

    let inviter_info = fund_data
        .members
        .iter()
        .find(|member| member.0 == *inviter_wallet_info.key)
        .ok_or(FundError::InvalidAccountData)?;

    let inviter_index = inviter_info.1;

    let mut joiner_data = UserAccount::try_from_slice(&joiner_account_info.data.borrow())?;
    let already_invited = joiner_data
        .funds
        .iter()
        .any(|user_specific| user_specific.fund == *fund_account_info.key);

    if already_invited {
        return Err(FundError::AlreadyInvited.into());
    }

    joiner_data.funds.push(UserSpecific {
        fund: *fund_account_info.key,
        fund_type: 0 as u8,
        governance_token_balance: 0 as u64,
        is_pending: true,
        is_eligible: 1 as u8,
        inviter_index,
        join_time: current_time
    });

    let rent = Rent::get()?;
    let current_joiner_space = joiner_account_info.data_len();
    let new_joiner_space = current_joiner_space + 55;
    let current_joiner_rent = joiner_account_info.lamports();
    let new_joiner_rent = rent.minimum_balance(new_joiner_space);

    if new_joiner_rent > current_joiner_rent {
        invoke(
            &system_instruction::transfer(
                inviter_wallet_info.key,
                joiner_account_info.key,
                new_joiner_rent - current_joiner_rent
            ),
            &[inviter_wallet_info.clone(), joiner_account_info.clone(), system_program_info.clone()]
        )?;
    }

    joiner_account_info.realloc(new_joiner_space, false)?;
    joiner_data.serialize(&mut &mut joiner_account_info.data.borrow_mut()[..])?;

    Ok(())
}

fn process_handle_invitation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: String,
    response: u8,
    inviter_exists: u8,
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let joiner_wallet_info = next_account_info(accounts_iter)?;
    let joiner_account_info = next_account_info(accounts_iter)?;
    let fund_account_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    let rent_reserve_info = next_account_info(accounts_iter)?;

    if !joiner_wallet_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"light-fund", fund_name.as_bytes()], program_id);
    let (joiner_pda, _joiner_bump) = Pubkey::find_program_address(&[b"user", joiner_wallet_info.key.as_ref()], program_id);
    let (rent_pda, _rent_bump) = Pubkey::find_program_address(&[b"rent"], program_id);

    if *fund_account_info.key != fund_pda || *joiner_account_info.key != joiner_pda || *rent_reserve_info.key != rent_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let mut fund_data = LightFundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    let mut joiner_data = UserAccount::try_from_slice(&joiner_account_info.data.borrow())?;

    if fund_data.members.iter().any(|member| member.0 == *joiner_wallet_info.key) {
        return Err(FundError::AlreadyMember.into());
    }

    let is_invited =  joiner_data.funds.iter().any(|fund| fund.fund == *fund_account_info.key);
    if !is_invited {
        return Err(FundError::NotInvited.into());
    }

    let user_specific = joiner_data
        .funds
        .iter()
        .find(|user_specific| user_specific.fund == *fund_account_info.key)
        .ok_or(FundError::InvalidAccountData)?;

    if !user_specific.is_pending {
        return Err(FundError::AlreadyMember.into());
    }

    let rent = Rent::get()?;

    if inviter_exists == 1 {
        // Inviter Verification
        let inviter_wallet_info = next_account_info(accounts_iter)?;
        let inviter_correct = fund_data
            .members
            .iter()
            .any(|member| member.0 == *inviter_wallet_info.key && member.1 == user_specific.inviter_index);

        if !inviter_correct {
            return Err(FundError::InvalidInviter.into());
        }

        if response == 0 {
            joiner_data.funds.retain(|fund| fund.fund != *fund_account_info.key);
            let current_joiner_space = joiner_account_info.data_len();
            let new_joiner_space = current_joiner_space - 55;
            let current_joiner_rent = joiner_account_info.lamports();
            let new_joiner_rent = rent.minimum_balance(new_joiner_space);

            if new_joiner_rent < current_joiner_rent {
                **joiner_account_info.lamports.borrow_mut() -= current_joiner_rent - new_joiner_rent;
                **inviter_wallet_info.lamports.borrow_mut() += current_joiner_rent - new_joiner_rent;
            }

            joiner_account_info.realloc(new_joiner_space, false)?;
            joiner_data.serialize(&mut &mut joiner_account_info.data.borrow_mut()[..])?;
        } else {
            if let Some(user_specific) = joiner_data
                .funds
                .iter_mut()
                .find(|entry| entry.fund == *fund_account_info.key) {
                    user_specific.is_pending = false;
                    user_specific.join_time = current_time;
                } else {
                    msg!("User is not a member in this fund");
                    return Err(FundError::InvalidAccountData.into());
                }

            joiner_data.serialize(&mut &mut joiner_account_info.data.borrow_mut()[..])?;

            let x = rent.minimum_balance(0 as usize);
            let y = rent.minimum_balance(55 as usize);
            invoke(
                &system_instruction::transfer(
                    joiner_wallet_info.key,
                    inviter_wallet_info.key,
                    y - x
                ),
                &[joiner_wallet_info.clone(), inviter_wallet_info.clone(), system_program_info.clone()]
            )?;
        }
    } else {
        if response == 0 {
            joiner_data.funds.retain(|fund| fund.fund != *fund_account_info.key);
            let current_joiner_space = joiner_account_info.data_len();
            let new_joiner_space = current_joiner_space - 55;
            let current_joiner_rent = joiner_account_info.lamports();
            let new_joiner_rent = rent.minimum_balance(new_joiner_space);

            if new_joiner_rent < current_joiner_rent {
                **joiner_account_info.lamports.borrow_mut() -= current_joiner_rent - new_joiner_rent;
                **rent_reserve_info.lamports.borrow_mut() += current_joiner_rent - new_joiner_rent;
            }

            joiner_account_info.realloc(new_joiner_space, false)?;
            joiner_data.serialize(&mut &mut joiner_account_info.data.borrow_mut()[..])?;
        } else {
            if let Some(user_specific) = joiner_data
                .funds
                .iter_mut()
                .find(|entry| entry.fund == *fund_account_info.key) {
                    user_specific.is_pending = false;
                    user_specific.join_time = current_time;
                } else {
                    msg!("User is not a member in this fund");
                    return Err(FundError::InvalidAccountData.into());
                }

            joiner_data.serialize(&mut &mut joiner_account_info.data.borrow_mut()[..])?;

            let x = rent.minimum_balance(0 as usize);
            let y = rent.minimum_balance(55 as usize);
            invoke(
                &system_instruction::transfer(
                    joiner_wallet_info.key,
                    rent_reserve_info.key,
                    y - x
                ),
                &[joiner_wallet_info.clone(), rent_reserve_info.clone(), system_program_info.clone()]
            )?;
        }
    }

    if response == 1 {
        fund_data.members.push((*joiner_wallet_info.key, fund_data.members[fund_data.members.len() - 1].1 + 1));
        let current_fund_size = fund_account_info.data_len();
        let new_fund_size = current_fund_size + 36;
        let current_fund_rent = fund_account_info.lamports();
        let new_fund_rent = rent.minimum_balance(new_fund_size);

        if new_fund_rent > current_fund_rent {
            invoke(
                &system_instruction::transfer(
                    joiner_wallet_info.key,
                    fund_account_info.key,
                    new_fund_rent - current_fund_rent
                ),
                &[joiner_wallet_info.clone(), fund_account_info.clone(), system_program_info.clone()]
            )?;
        }

        fund_account_info.realloc(new_fund_size, false)?;
        fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;
    }

    Ok(())
}

fn process_withdraw_or_leave_from_light_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: String,
    task: u8,
    stake_percent: u64,
    num_of_tokens: u8,
) -> ProgramResult {
    // let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let member_wallet_info = next_account_info(accounts_iter)?;
    let member_account_info = next_account_info(accounts_iter)?;
    let fund_account_info = next_account_info(accounts_iter)?;
    let vault_account_info = next_account_info(accounts_iter)?;
    let token_program_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    let ata_program_info = next_account_info(accounts_iter)?;
    let rent_sysvar_info = next_account_info(accounts_iter)?;

    msg!("Stake Percent: {}", stake_percent);

    if stake_percent > 100_000_000 {
        return Err(FundError::InvalidStakePercent.into());
    }

    if !member_wallet_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    if task == 1 && stake_percent != 100_000_000 {
        msg!("Task does not match the data provided.");
        return Err(FundError::InvalidInstruction.into());
    }

    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"light-fund", fund_name.as_bytes()], program_id);
    let (member_pda, _joiner_bump) = Pubkey::find_program_address(&[b"user", member_wallet_info.key.as_ref()], program_id);
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault", fund_account_info.key.as_ref()], program_id);

    if *fund_account_info.key != fund_pda || *member_account_info.key != member_pda || *vault_account_info.key != vault_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let mut mint_account_infos: Vec<&AccountInfo> = vec![];
    for _i in 0..num_of_tokens {
        let mint_account_info = next_account_info(accounts_iter)?;
        mint_account_infos.push(mint_account_info);
    }

    let mut member_ata_infos: Vec<&AccountInfo> = vec![];
    for _i in 0..num_of_tokens {
        let member_ata_info = next_account_info(accounts_iter)?;
        member_ata_infos.push(member_ata_info);
    }

    let mut vault_ata_infos: Vec<&AccountInfo> = vec![];
    for _i in 0..num_of_tokens {
        let vault_ata_info = next_account_info(accounts_iter)?;
        vault_ata_infos.push(vault_ata_info);
    }

    let mut member_data = UserAccount::try_from_slice(&member_account_info.data.borrow())?;
    let mut fund_data = LightFundAccount::try_from_slice(&fund_account_info.data.borrow())?;

    let (matched_index, user_specific) = member_data
        .funds
        .iter()
        .enumerate()
        .find(|(_i, user_specific)| user_specific.fund == *fund_account_info.key)
        .ok_or(FundError::InvalidAccountData)?;

    let member_deposit = user_specific.governance_token_balance;
    let total_deposit = fund_data.total_deposit;
    msg!("Fund's Total Deposit before Withdrawal: {}", total_deposit);
    msg!("Member Deposit before Withdrawal: {}", member_deposit);
    if member_deposit > total_deposit {
        return Err(FundError::InvalidAccountData.into());
    }

    let wsol_mint = pubkey!("So11111111111111111111111111111111111111112");

    if total_deposit != 0 {
        let withdraw_percent_u128: u128 = ((stake_percent as u128) * (member_deposit as u128))/(total_deposit as u128);
        let withdraw_percent = withdraw_percent_u128 as u64;
        msg!("Withdraw percent overall: {}", withdraw_percent);

        if withdraw_percent != 0 {
            for i in 0..num_of_tokens {
                let token_account = spl_token::state::Account::unpack(&vault_ata_infos[i as usize].data.borrow())?;
                let vault_balance = token_account.amount;
                msg!("Vault Balance: {}", vault_balance);
                let amount_to_transfer_u128: u128 = ((vault_balance as u128) * (withdraw_percent as u128))/(100_000_000 as u128);
                let amount_to_transfer = amount_to_transfer_u128 as u64;
                msg!("Amount to Transfer: {}", amount_to_transfer);

                if *mint_account_infos[i as usize].key == wsol_mint {
                    msg!("Withdrawing SOL");
                    if member_ata_infos[i as usize].data_is_empty() {
                        invoke(
                            &spl_associated_token_account::instruction::create_associated_token_account(
                                member_wallet_info.key,
                                member_wallet_info.key,
                                mint_account_infos[i as usize].key,
                                token_program_info.key,
                            ),
                            &[
                                member_wallet_info.clone(),
                                member_ata_infos[i as usize].clone(),
                                token_program_info.clone(),
                                mint_account_infos[i as usize].clone(),
                                system_program_info.clone(),
                                ata_program_info.clone(),
                                rent_sysvar_info.clone(),
                            ]
                        )?;
                    }

                    invoke_signed(
                        &spl_token::instruction::transfer(
                            token_program_info.key,
                            vault_ata_infos[i as usize].key,
                            member_ata_infos[i as usize].key,
                            vault_account_info.key,
                            &[],
                            amount_to_transfer
                        )?,
                        &[
                            token_program_info.clone(),
                            vault_ata_infos[i as usize].clone(),
                            member_ata_infos[i as usize].clone(),
                            vault_account_info.clone()
                        ],
                        &[&[b"vault", fund_account_info.key.as_ref(), &[vault_bump]]]
                    )?;

                    invoke(
                        &spl_token::instruction::close_account(
                            token_program_info.key,
                            member_ata_infos[i as usize].key,
                            member_wallet_info.key,
                            member_wallet_info.key,
                            &[]
                        )?,
                        &[
                            token_program_info.clone(),
                            member_wallet_info.clone(),
                            member_ata_infos[i as usize].clone(),
                            system_program_info.clone(),
                            ata_program_info.clone(),
                            rent_sysvar_info.clone()
                        ]
                    )?;
                } else {
                    msg!("Withdrawing SPL Token");
                    if member_ata_infos[i as usize].data_is_empty() {
                        invoke(
                            &spl_associated_token_account::instruction::create_associated_token_account(
                                member_wallet_info.key,
                                member_wallet_info.key,
                                mint_account_infos[i as usize].key,
                                token_program_info.key,
                            ),
                            &[
                                member_wallet_info.clone(),
                                member_ata_infos[i as usize].clone(),
                                token_program_info.clone(),
                                mint_account_infos[i as usize].clone(),
                                system_program_info.clone(),
                                ata_program_info.clone(),
                                rent_sysvar_info.clone(),
                            ]
                        )?;
                    }

                    invoke_signed(
                        &spl_token::instruction::transfer(
                            token_program_info.key,
                            vault_ata_infos[i as usize].key,
                            member_ata_infos[i as usize].key,
                            vault_account_info.key,
                            &[],
                            amount_to_transfer
                        )?,
                        &[
                            token_program_info.clone(),
                            vault_ata_infos[i as usize].clone(),
                            member_ata_infos[i as usize].clone(),
                            vault_account_info.clone()
                        ],
                        &[&[b"vault", fund_account_info.key.as_ref(), &[vault_bump]]]
                    )?;
                }
            }
        }
    }

    let rent = Rent::get()?;
    
    fund_data.total_deposit -= (((member_data.funds[matched_index].governance_token_balance as u128) * (stake_percent as u128))/(100_000_000 as u128)) as u64;
    msg!("Fund's Total Deposit after Withdrawal: {}", fund_data.total_deposit);

    if task == 1 {
        fund_data.members.retain(|member| member.0 != *member_wallet_info.key);
        member_data.funds.retain(|user_specific| user_specific.fund != *fund_account_info.key);

        if fund_data.creator_exists && fund_data.members[0 as usize].0 == *member_wallet_info.key {
            fund_data.creator_exists = false;
        }

        let current_fund_size = fund_account_info.data_len();
        let new_fund_size = current_fund_size - 36;
        let current_fund_rent = fund_account_info.lamports();
        let new_fund_rent = rent.minimum_balance(new_fund_size);
        

        let current_user_size = member_account_info.data_len();
        let new_user_size = current_user_size - 55;
        let current_user_rent = member_account_info.lamports();
        let new_user_rent = rent.minimum_balance(new_user_size);

        fund_account_info.realloc(new_fund_size, false)?;
        member_account_info.realloc(new_user_size, false)?;

        if current_fund_rent > new_fund_rent  {
            **fund_account_info.lamports.borrow_mut() -= current_fund_rent - new_fund_rent;
            **member_wallet_info.lamports.borrow_mut() += current_fund_rent - new_fund_rent;
        }

        if current_user_rent > new_user_rent  {
            **member_account_info.lamports.borrow_mut() -= current_user_rent - new_user_rent;
            **member_wallet_info.lamports.borrow_mut() += current_user_rent - new_user_rent;
        }
    } else {
        if total_deposit != 0 && stake_percent != 0 {
            member_data.funds[matched_index].governance_token_balance -= (((member_data.funds[matched_index].governance_token_balance as u128) * (stake_percent as u128))/(100_000_000 as u128)) as u64;
            msg!("Member Deposit After Withdrawal: {}", member_data.funds[matched_index].governance_token_balance);
        }
    }
    fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;
    member_data.serialize(&mut &mut member_account_info.data.borrow_mut()[..])?;

    msg!("Transferred holdings successfully");

    Ok(())
}

// fn process_init_fund_account<'a>(
//     program_id: &Pubkey,
//     accounts: &'a [AccountInfo<'a>],
//     fund_name: String,
//     privacy: u8,
//     expected_members: u32,
//     symbol_str: String,
// ) -> ProgramResult {
//     let current_time = Clock::get()?.unix_timestamp;

//     let accounts_iter = &mut accounts.iter();
//     let governance_mint_info = next_account_info(accounts_iter)?; // Governance Mint .........................
//     let vault_account_info = next_account_info(accounts_iter)?; // Vault PDA Account .........................
//     let system_program_info = next_account_info(accounts_iter)?; // System Program ...........................
//     let token_program_2022_info = next_account_info(accounts_iter)?; // Token Program (2022) .................
//     let fund_account_info = next_account_info(accounts_iter)?; // Fund PDA Account ...........................
//     let creator_wallet_info = next_account_info(accounts_iter)?; // Creator Wallet Address ...................
//     let rent_sysvar_info = next_account_info(accounts_iter)?; // Rent Sysvar .................................
//     let user_account_info = next_account_info(accounts_iter)?; // Global User Account ........................
//     let proposal_aggregator_info = next_account_info(accounts_iter)?; // first proposal aggregator ...........
//     let join_proposal_aggregator_info = next_account_info(accounts_iter)?; // join proposal aggregator .......

//     // Creator should be signer
//     if !creator_wallet_info.is_signer {
//         msg!("[FUND-ERROR] {} {} Wrong signer!(must be your wallet)", fund_account_info.key.to_string(), creator_wallet_info.key.to_string());
//         return Err(FundError::MissingRequiredSignature.into());
//     }

//     let index: u8 = 0;

//     // Deriving required PDAs
//     let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
//     let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault", fund_pda.as_ref()], program_id);
//     let (governance_mint, governance_bump) = Pubkey::find_program_address(&[b"governance", fund_pda.as_ref()], program_id);
//     let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", creator_wallet_info.key.as_ref()], program_id);
//     let (proposal_pda, proposal_bump) = Pubkey::find_program_address(&[b"proposal-aggregator", &[index], fund_pda.as_ref()], program_id);
//     let (join_aggregator_pda, join_aggregator_bump) = Pubkey::find_program_address(&[b"join-proposal-aggregator", &[index], fund_pda.as_ref()], program_id);

//     // Check if any of the provided PDA differes from the derived
//     if *fund_account_info.key != fund_pda ||
//        *vault_account_info.key != vault_pda ||
//        *governance_mint_info.key != governance_mint ||
//        *user_account_info.key != user_pda ||
//        *proposal_aggregator_info.key != proposal_pda ||
//        *join_proposal_aggregator_info.key != join_aggregator_pda {
//         msg!("[FUND-ERROR] {} {} Given PDAs doesn't match with the derived ones(Wrong accounts provided).", fund_account_info.key.to_string(), creator_wallet_info.key.to_string());
//         return Err(FundError::InvalidAccountData.into());
//     }

//     // Check if the an account already exists on that PDA
//     if fund_account_info.lamports() > 0 {
//         msg!("[FUND-ERROR] {} {} Fund already exists!!", fund_account_info.key.to_string(), creator_wallet_info.key.to_string());
//         return Err(FundError::InvalidAccountData.into());
//     }

//     // Calculate Rent
//     let rent = &Rent::from_account_info(rent_sysvar_info)?;
//     let fund_space = 150 as usize;
//     let vault_space = 40 as usize;
//     let extensions = vec![ExtensionType::NonTransferable, ExtensionType::MetadataPointer];
//     let base_mint_space = ExtensionType::try_calculate_account_len::<Mint>(&extensions)?;
//     let token_name = fund_name.clone();
//     let token_symbol = String::from(symbol_str.clone());
//     let token_uri = "".to_string();
//     let mint_space = base_mint_space;
//     let proposal_space = 37 as usize;
//     let join_proposal_space = 37 as usize;

//     // Creating the Fund Account PDA
//     invoke_signed(
//         &system_instruction::create_account(
//             creator_wallet_info.key,
//             fund_account_info.key,
//             rent.minimum_balance(fund_space),
//             fund_space as u64,
//             program_id,
//         ),
//         &[creator_wallet_info.clone(), fund_account_info.clone(), system_program_info.clone()],
//         &[&[b"fund", fund_name.as_bytes(), &[fund_bump]]]
//     )?;

//     // Creating the Vault Account PDA
//     invoke_signed(
//         &system_instruction::create_account(
//             creator_wallet_info.key,
//             vault_account_info.key,
//             rent.minimum_balance(vault_space),
//             vault_space as u64,
//             program_id,
//         ),
//         &[creator_wallet_info.clone(), vault_account_info.clone(), system_program_info.clone()],
//         &[&[b"vault", fund_pda.as_ref(), &[vault_bump]]],
//     )?;

//     // Creating the Governance Mint account
//     invoke_signed(
//         &system_instruction::create_account(
//             creator_wallet_info.key,
//             governance_mint_info.key,
//             rent.minimum_balance(mint_space),
//             mint_space as u64,
//             token_program_2022_info.key,
//         ),
//         &[creator_wallet_info.clone(), governance_mint_info.clone(), system_program_info.clone(), token_program_2022_info.clone()],
//         &[&[b"governance", fund_pda.as_ref(), &[governance_bump]]],
//     )?;

//     invoke_signed(
//         &spl_token_2022::instruction::initialize_non_transferable_mint(
//             &spl_token_2022::id(),
//             governance_mint_info.key,
//         )?,
//         &[governance_mint_info.clone(), token_program_2022_info.clone()],
//         &[&[b"governance", fund_pda.as_ref(), &[governance_bump]]]
//     )?;
//     invoke_signed(
//         &metadata_pointer::instruction::initialize(
//             &spl_token_2022::id(),
//             governance_mint_info.key,
//             Some(*fund_account_info.key),
//             Some(*governance_mint_info.key),
//         )?,
//         &[governance_mint_info.clone(), rent_sysvar_info.clone(), token_program_2022_info.clone()],
//         &[&[b"governance", fund_pda.as_ref(), &[governance_bump]]],
//     )?;
//     invoke_signed(
//         &spl_token_2022::instruction::initialize_mint2(
//             &spl_token_2022::id(),
//             governance_mint_info.key,
//             fund_account_info.key,
//             Some(fund_account_info.key),
//             0,
//         )?,
//         &[
//             governance_mint_info.clone(),
//             rent_sysvar_info.clone(),
//             // token_program_2022_info.clone(),
//             // fund_account_info.clone()
//         ],
//         &[&[b"governance", fund_pda.as_ref(), &[governance_bump]]],
//     )?;


//     // creating the proposal aggregator account
//     invoke_signed(
//         &system_instruction::create_account(
//             creator_wallet_info.key,
//             proposal_aggregator_info.key,
//             rent.minimum_balance(proposal_space),
//             proposal_space as u64,
//             program_id
//         ),
//         &[creator_wallet_info.clone(), system_program_info.clone(), proposal_aggregator_info.clone()],
//         &[&[b"proposal-aggregator", &[index], fund_pda.as_ref(), &[proposal_bump]]]
//     )?;

//     // creating the joining proposal aggregator account
//     if privacy == 1 {
//         invoke_signed(
//             &system_instruction::create_account(
//                 creator_wallet_info.key,
//                 join_proposal_aggregator_info.key,
//                 rent.minimum_balance(join_proposal_space),
//                 join_proposal_space as u64,
//                 program_id
//             ),
//             &[creator_wallet_info.clone(), system_program_info.clone(), join_proposal_aggregator_info.clone()],
//             &[&[b"join-proposal-aggregator", &[index], fund_pda.as_ref(), &[join_aggregator_bump]]]
//         )?;
//     }

//     let current_governance_lamports = governance_mint_info.lamports();
//     let new_governance_rent = rent.minimum_balance(322 + fund_name.len() + symbol_str.len());
//     let transfer_amount = new_governance_rent - current_governance_lamports;
//     invoke(
//         &system_instruction::transfer(
//             creator_wallet_info.key,
//             governance_mint_info.key,
//             transfer_amount + 1
//         ),
//         &[creator_wallet_info.clone(), governance_mint_info.clone(), system_program_info.clone()]
//     )?;

//     invoke_signed(
//         &spl_token_metadata_interface::instruction::initialize(
//             &spl_token_2022::id(),
//             governance_mint_info.key,
//             fund_account_info.key,
//             governance_mint_info.key,
//             fund_account_info.key,
//             token_name,
//             token_symbol,
//             token_uri
//         ),
//         &[
//             governance_mint_info.clone(),
//             fund_account_info.clone(),
//             fund_account_info.clone(),
//             token_program_2022_info.clone()
//         ],
//         &[&[b"fund", fund_name.as_bytes(), &[fund_bump]]]
//     )?;

//     // Converting the fund_name to an array of u8 of fixed size 32
//     let bytes = fund_name.as_bytes();
//     let mut array = [0u8; 26];
//     let len = bytes.len().min(26);
//     array[..len].copy_from_slice(&bytes[..len]);

//     let members: Vec<Pubkey> = vec![*creator_wallet_info.key];
//     let mut is_refunded = false;
//     if expected_members == 1 {
//         is_refunded = true;
//     }

//     // Deserialization and Serialization of Fund data
//     let fund_data = FundAccount {
//         name: array,
//         is_refunded,
//         expected_members,
//         creator_exists: true,
//         total_deposit: 0 as u64,
//         governance_mint: *governance_mint_info.key,
//         vault: *vault_account_info.key,
//         current_proposal_index: 0 as u8,
//         created_at: current_time,
//         is_private: privacy,
//         members,
//     };
//     fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

//     // Deserializing the User Global PDA
//     let mut user_data = UserAccount::try_from_slice(&user_account_info.data.borrow())?;

//     // Calculate current size and new size of User PDA
//     let current_size = user_account_info.data_len();
//     let new_size = current_size + 50;

//     // Calculate current min rent-exempt and new rent-exempt
//     let new_min_balance = rent.minimum_balance(new_size);
//     let current_balance = user_account_info.lamports();

//     // Deposit lamports if required
//     if new_min_balance > current_balance {
//         invoke(
//             &system_instruction::transfer(
//                 creator_wallet_info.key,
//                 user_account_info.key,
//                 new_min_balance - current_balance,
//             ),
//             &[creator_wallet_info.clone(), user_account_info.clone(), system_program_info.clone()],
//         )?;
//     }

//     // Reallocation for new bytes
//     user_account_info.realloc(new_size, false)?;
//     // Add the new fund details to user funds
//     user_data.funds.push(UserSpecific {
//         fund: *fund_account_info.key,
//         governance_token_balance: 0 as u64,
//         is_pending: false,
//         is_eligible: true,
//         join_time: current_time
//     });
//     user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;

//     // Deserialization and Serialization of Vault Account Data
//     let vault_data = VaultAccount {
//         last_deposit_time: 0,
//     };
//     vault_data.serialize(&mut &mut vault_account_info.data.borrow_mut()[..])?;

//     let proposals: Vec<Proposal> = vec![];

//     let proposal_aggregator_data = ProposalAggregatorAccount {
//         index: 0 as u8,
//         proposals
//     };

//     proposal_aggregator_data.serialize(&mut &mut proposal_aggregator_info.data.borrow_mut()[..])?;

//     if privacy == 1 {
//         let join_proposals: Vec<JoinProposal> = vec![];

//         let join_proposal_aggreagtor_data = JoinProposalAggregator {
//             fund: *fund_account_info.key,
//             index: 0 as u8,
//             join_proposals,
//         };

//         join_proposal_aggreagtor_data.serialize(&mut &mut join_proposal_aggregator_info.data.borrow_mut()[..])?;
//     }

//     msg!("[FUND-ACTIVITY] {} {} {} Fund created by: {}", fund_account_info.key.to_string(), current_time, fund_name, creator_wallet_info.key.to_string());

//     Ok(())
// }


fn process_init_user_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    cid: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let creator_account_info = next_account_info(accounts_iter)?; // User Wallet
    let user_account_info = next_account_info(accounts_iter)?; // User PDA Account to be created
    let system_program_info = next_account_info(accounts_iter)?; // System Program

    // User should be the signer
    if !creator_account_info.is_signer {
        msg!("[USER-ERROR] {} Wrong signer!(must be your wallet)", creator_account_info.key.to_string());
        return Err(FundError::MissingRequiredSignature.into());
    }

    // Derive User PDA and check if provided is same as expected
    let (user_pda, user_bump) = Pubkey::find_program_address(&[b"user", creator_account_info.key.as_ref()], program_id);
    if *user_account_info.key != user_pda {
        msg!("[USER-ERROR] {} Given PDAs doesn't match with the derived ones(Wrong accounts provided).", creator_account_info.key.to_string());
        return Err(FundError::InvalidAccountData.into());
    }

    // If user PDA already exists
    if !user_account_info.data_is_empty() {
        msg!("[USER-ERROR] {} User already exists", creator_account_info.key.to_string());
        return Ok(());
    }

    // Calculate rent-exempt
    let rent = Rent::get()?;
    let user_space = 59 + 4 as usize;
    
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

    let bytes = cid.as_bytes();
    let mut array = [0u8; 59];
    let len = bytes.len().min(59);
    array[..len].copy_from_slice(&bytes[..len]);
    // Deserialization and Serialization of User Account Data
    let user_data = UserAccount {
        user_cid: array,
        funds,
    };
    user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;

    Ok(())

}

// fn process_init_join_proposal(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     fund_name: String,
// ) -> ProgramResult {
//     let creation_time = Clock::get()?.unix_timestamp;

//     let accounts_iter = &mut accounts.iter();
//     let joiner_account_info = next_account_info(accounts_iter)?; // joiner wallet ............................
//     let join_proposal_aggregator_info = next_account_info(accounts_iter)?; // join proposal aggregator .......
//     let fund_account_info = next_account_info(accounts_iter)?; // fund account ...............................
//     let vote_account_info = next_account_info(accounts_iter)?; // join votes aggregator ......................
//     let system_program_info = next_account_info(accounts_iter)?; // system program ...........................
//     let user_account_info = next_account_info(accounts_iter)?; // user's global account ......................

//     if !joiner_account_info.is_signer {
//         msg!("[FUND-ERROR] {} {} Wrong signer!(must be your wallet)", fund_account_info.key.to_string(), joiner_account_info.key.to_string());
//         return Err(FundError::MissingRequiredSignature.into());
//     }
//     let mut proposal_index = 0;
//     let index = 0 as u8;

//     let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
//     let (join_proposal_pda, _join_proposal_bump) = Pubkey::find_program_address(&[b"join-proposal-aggregator", &[index], fund_pda.as_ref()], program_id);
//     let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", joiner_account_info.key.as_ref()], program_id);
//     let (join_vote_pda, join_vote_bump) = Pubkey::find_program_address(&[b"join-vote", &[proposal_index], fund_pda.as_ref()], program_id);

//     if *fund_account_info.key != fund_pda || *join_proposal_aggregator_info.key != join_proposal_pda || *user_account_info.key != user_pda || *vote_account_info.key != join_vote_pda{
//         msg!("[FUND-ERROR] {} {} Given PDAs doesn't match with the derived ones(Wrong accounts provided).", fund_account_info.key.to_string(), joiner_account_info.key.to_string());
//         return Err(FundError::InvalidAccountData.into());
//     }

//     if !vote_account_info.data_is_empty() {
//         return Err(FundError::InvalidVoteAccount.into());
//     }

//     let mut join_proposal_data = JoinProposalAggregator::try_from_slice(&join_proposal_aggregator_info.data.borrow())?;

//     // Deserialize User Data and check if User is already a member of provided Fund
//     let mut user_data = UserAccount::try_from_slice(&user_account_info.data.borrow())?;
//     if user_data.funds.iter().any(|entry| entry.fund == *fund_account_info.key) {
//         msg!("[FUND-ERROR] {} {} Either a member already, or has a pending join proposal for this fund!", fund_account_info.key.to_string(), joiner_account_info.key.to_string());
//         return Err(FundError::AlreadyMember.into());
//     }

//     // check if already applied for entry in fund
//     let joiner_in_queue = join_proposal_data
//         .join_proposals
//         .iter()
//         .any(|proposal| proposal.joiner == *joiner_account_info.key);

//     if joiner_in_queue {
//         msg!("[FUND-ERROR] {} {} You have already applied for this fund! If want to apply again, delete your existing proposal in the 'Pending Funds' section.", fund_account_info.key.to_string(), joiner_account_info.key.to_string());
//         return Err(FundError::AlreadyAppliedForEntry.into());
//     }

//     let fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
//     if fund_data.members.len() >= fund_data.expected_members as usize {
//         msg!("[FUND-ERROR] {} {} Fund already exists!!", fund_account_info.key.to_string(), joiner_account_info.key.to_string());
//         return Err(FundError::FundAlreadyFull.into());
//     }

//     let rent = Rent::get()?;
//     let extra_space = 57 as usize;
//     let current_space = join_proposal_aggregator_info.data_len();
//     let current_rent = join_proposal_aggregator_info.lamports();
//     let new_space = current_space + extra_space;
//     let new_rent = rent.minimum_balance(new_space);

//     if new_rent > current_rent {
//         invoke(
//             &system_instruction::transfer(
//                 joiner_account_info.key,
//                 join_proposal_aggregator_info.key,
//                 new_rent - current_rent,
//             ),
//             &[joiner_account_info.clone(), join_proposal_aggregator_info.clone(), system_program_info.clone()]
//         )?;
//     }

//     join_proposal_aggregator_info.realloc(new_space, false)?;
//     let num_of_join_proposals = join_proposal_data.join_proposals.len();
//     if num_of_join_proposals != 0 {
//         proposal_index = join_proposal_data.join_proposals[join_proposal_data.join_proposals.len() - 1].proposal_index + 1;
//     }
//     join_proposal_data.join_proposals.push(JoinProposal {
//         joiner: *joiner_account_info.key,
//         votes_yes: 0 as u64,
//         votes_no: 0 as u64,
//         creation_time,
//         proposal_index,
//     });

//     join_proposal_data.serialize(&mut &mut join_proposal_aggregator_info.data.borrow_mut()[..])?;

//     let vote_space = 5 as usize;
    
//     invoke_signed(
//         &system_instruction::create_account(
//             joiner_account_info.key,
//             vote_account_info.key,
//             rent.minimum_balance(vote_space),
//             vote_space as u64,
//             program_id
//         ),
//         &[joiner_account_info.clone(), vote_account_info.clone(), system_program_info.clone()],
//         &[&[b"join-vote", &[proposal_index], fund_pda.as_ref(), &[join_vote_bump]]]
//     )?;

//     let voters: Vec<(Pubkey, u8)> = vec![];

//     let join_vote_data = JoinVoteAccount {
//         proposal_index,
//         voters,
//     };

//     join_vote_data.serialize(&mut &mut vote_account_info.data.borrow_mut()[..])?;

//     let current_user_size = user_account_info.data_len();
//     let new_user_size = current_user_size + 50;
//     let current_user_rent = user_account_info.lamports();
//     let new_user_rent = rent.minimum_balance(new_user_size);

//     if new_user_rent > current_user_rent {
//         invoke(
//             &system_instruction::transfer(
//                 joiner_account_info.key,
//                 user_account_info.key,
//                 new_user_rent - current_user_rent
//             ),
//             &[joiner_account_info.clone(), user_account_info.clone(), system_program_info.clone()]
//         )?;
//     }

//     user_account_info.realloc(new_user_size, false)?;

//     user_data.funds.push(UserSpecific {
//         fund: *fund_account_info.key,
//         governance_token_balance: 0 as u64,
//         is_pending: true,
//         is_eligible: false,
//         join_time: creation_time
//     });

//     user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;

//     msg!("[FUND-ACTIVITY] {} {} {} {} created proposal to join the fund", fund_account_info.key.to_string(), creation_time, fund_name, joiner_account_info.key.to_string());

//     Ok(())
// }

// fn process_vote_on_join_proposal(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     vote: u8,
//     fund_name: String,
//     proposal_index: u8
// ) -> ProgramResult {
//     let current_time = Clock::get()?.unix_timestamp;

//     let accounts_iter = &mut accounts.iter();
//     let voter_account_info = next_account_info(accounts_iter)?; // Voter Wallet ..................................
//     let vote_account_info = next_account_info(accounts_iter)?; // Join Votes aggregator ..........................
//     let join_proposal_aggregator_info = next_account_info(accounts_iter)?; // Join proposal aggregator ...........
//     let system_program_info = next_account_info(accounts_iter)?; // System Program ...............................
//     let fund_account_info = next_account_info(accounts_iter)?; // fund Account ...................................
//     let governance_token_mint_info = next_account_info(accounts_iter)?; // Governance Mint account ...............
//     let voter_token_account_info = next_account_info(accounts_iter)?; // Voter's governance token account ........
//     let token_program_2022_info = next_account_info(accounts_iter)?; // token extension program ..................
//     let joiner_account_info = next_account_info(accounts_iter)?; // joiner wallet ................................
//     let joiner_pda_info = next_account_info(accounts_iter)?; // joiner global account ............................

//     if !voter_account_info.is_signer {
//         msg!("[FUND-ERROR] {} {} Wrong signer!(must be your wallet)", fund_account_info.key.to_string(), voter_account_info.key.to_string());
//         return Err(FundError::MissingRequiredSignature.into());
//     }

//     let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
//     let index = 0 as u8;
//     let (join_proposal_pda, _join_proposal_bump) = Pubkey::find_program_address(&[b"join-proposal-aggregator", &[index], fund_pda.as_ref()], program_id);
//     let (join_vote_pda, _join_vote_bump) = Pubkey::find_program_address(&[b"join-vote", &[proposal_index], fund_pda.as_ref()], program_id);
//     let (governance_mint, _governance_bump) = Pubkey::find_program_address(&[b"governance", fund_pda.as_ref()], program_id);
//     let (joiner_pda, _joiner_bump) = Pubkey::find_program_address(&[b"user", joiner_account_info.key.as_ref()], program_id);

//     if *fund_account_info.key != fund_pda ||
//        *join_proposal_aggregator_info.key != join_proposal_pda ||
//        *vote_account_info.key != join_vote_pda ||
//        *governance_token_mint_info.key != governance_mint ||
//        *joiner_pda_info.key != joiner_pda {
//         msg!("[FUND-ERROR] {} {} Given PDAs doesn't match with the derived ones(Wrong accounts provided).", fund_account_info.key.to_string(), voter_account_info.key.to_string());
//         return Err(FundError::InvalidAccountData.into());
//     }

//     let token_account = spl_associated_token_account::get_associated_token_address_with_program_id(
//         voter_account_info.key,
//         governance_token_mint_info.key,
//         token_program_2022_info.key
//     );

//     if token_account != *voter_token_account_info.key {
//         msg!("[FUND-ERROR] {} {} Your token account doesn's match the derived one.", fund_account_info.key.to_string(), voter_account_info.key.to_string());
//         return Err(FundError::InvalidTokenAccount.into());
//     }

//     if voter_token_account_info.data_is_empty() {
//         msg!("[FUND-ERROR] {} {} Your token account does not exist.", fund_account_info.key.to_string(), voter_account_info.key.to_string());
//         return  Err(FundError::NoVotingPower.into());
//     }

//     if vote_account_info.data_is_empty() {
//         msg!("[FUND-ERROR] {} {} The vote account for this proposal doesn't exist. Maybe the proposal is cancelled.", fund_account_info.key.to_string(), voter_account_info.key.to_string());
//         return Err(FundError::InvalidVoteAccount.into());
//     }

//     let mut join_proposal_data = JoinProposalAggregator::try_from_slice(&join_proposal_aggregator_info.data.borrow())?;
//     let (matched_index, proposal) = join_proposal_data
//         .join_proposals
//         .iter()
//         .enumerate()
//         .find(|(_, p)| p.proposal_index == proposal_index)
//         .ok_or(FundError::InvalidAccountData)?;

//     if proposal.joiner != *joiner_account_info.key {
//         msg!("[FUND-ERROR] {} {} The proposal's joiner wallet differs from what is provided.", fund_account_info.key.to_string(), voter_account_info.key.to_string());
//         return Err(FundError::InvalidAccountData.into());
//     }

//     let mut joiner_data = UserAccount::try_from_slice(&joiner_pda_info.data.borrow())?;
//     let fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
//     let is_voter_member = fund_data
//         .members
//         .iter()
//         .any(|member| *member == *voter_account_info.key);

//     if !is_voter_member {
//         msg!("[FUND-ERROR] {} {} You are not a member of the fund and so cannot vote.", fund_account_info.key.to_string(), voter_account_info.key.to_string());
//         return Err(FundError::NotAFundMember.into());
//     }

//     let mut vote_data = JoinVoteAccount::try_from_slice(&mut &mut vote_account_info.data.borrow())?;

//     // check if already voted
//     let voter_exists = vote_data
//         .voters
//         .iter()
//         .any(|(key, _)| *key == *voter_account_info.key);

//     if voter_exists {
//         msg!("[FUND-ERROR] {} {} You have voted for this proposal already.", fund_account_info.key.to_string(), voter_account_info.key.to_string());
//         return Err(FundError::AlreadyVoted.into());
//     }

//     let rent = Rent::get()?;
//     let current_space = vote_account_info.data_len();
//     let new_space = current_space + 33 as usize;
//     let current_rent = vote_account_info.lamports();
//     let new_rent = rent.minimum_balance(new_space);

//     // transfer reallocation lamports
//     if new_rent > current_rent {
//         invoke(
//             &system_instruction::transfer(
//                 voter_account_info.key,
//                 vote_account_info.key,
//                 new_rent - current_rent
//             ),
//             &[voter_account_info.clone(), vote_account_info.clone(), system_program_info.clone()]
//         )?;
//     }

//     vote_account_info.realloc(new_space, false)?;
//     vote_data.voters.push((*voter_account_info.key, vote));
//     vote_data.serialize(&mut &mut vote_account_info.data.borrow_mut()[..])?;

    
//     if *voter_token_account_info.owner != *token_program_2022_info.key {
//         msg!("[FUND-ERROR] {} {} The owner of your token account is not Token Program 2022.", fund_account_info.key.to_string(), voter_account_info.key.to_string());
//         return Err(FundError::InvalidAccountData.into());
//     }

//     let token_account_data = voter_token_account_info.try_borrow_data()?;
//     let token_account = spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(&token_account_data)?;
//     let base_account = token_account.base;
//     let balance = base_account.amount;
//     if vote == 0 {
//         join_proposal_data.join_proposals[matched_index as usize].votes_no += balance;
//     } else {
//         join_proposal_data.join_proposals[matched_index as usize].votes_yes += balance;
//     }

//     join_proposal_data.serialize(&mut &mut join_proposal_aggregator_info.data.borrow_mut()[..])?;

//     if 2 * join_proposal_data.join_proposals[matched_index as usize].votes_yes >= fund_data.total_deposit {
//         if let Some(user_specific) = joiner_data
//             .funds
//             .iter_mut()
//             .find(|entry| entry.fund == *fund_account_info.key) {
//                 user_specific.is_pending = true;
//                 // user_specific.is_eligible = true;
//             } else {
//                 msg!("[FUND-ERROR] {} {} User is not a member in this fund.", fund_account_info.key.to_string(), voter_account_info.key.to_string());
//                 return Err(FundError::InvalidAccountData.into());
//             }
//     }

//     joiner_data.serialize(&mut &mut joiner_pda_info.data.borrow_mut()[..])?;

//     msg!("[FUND-ACTIVITY] {} {} {} {} voted for addition of {}", fund_account_info.key.to_string(), current_time, fund_name, voter_account_info.key.to_string(), joiner_account_info.key.to_string());

//     Ok(())
// }


// fn process_cancel_join_proposal(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     fund_name: String,
//     proposal_index: u8,
// ) -> ProgramResult {
//     let accounts_iter = &mut accounts.iter();
//     let joiner_account_info = next_account_info(accounts_iter)?; //  joiner wallet ...........................
//     let joiner_pda_info = next_account_info(accounts_iter)?; // joiner global pda ............................
//     let fund_account_info = next_account_info(accounts_iter)?; // fund account ...............................
//     let join_proposal_aggregator_info = next_account_info(accounts_iter)?; // join proposal aggregator .......
//     let vote_account_info = next_account_info(accounts_iter)?; // vote account ...............................
//     let rent_reserve_info = next_account_info(accounts_iter)?; // rent reserve ...............................

//     if !joiner_account_info.is_signer {
//         msg!("[FUND-ERROR] {} {} Wrong signer!(must be your wallet)", fund_account_info.key.to_string(), joiner_account_info.key.to_string());
//         return Err(FundError::MissingRequiredSignature.into());
//     }

//     let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
//     let (joiner_pda, _joiner_bump) = Pubkey::find_program_address(&[b"user", joiner_account_info.key.as_ref()], program_id);
//     let index = 0 as u8;
//     let (join_aggregator_pda, _join_aggregator_bump) = Pubkey::find_program_address(&[b"join-proposal-aggregator", &[index], fund_account_info.key.as_ref()], program_id);
//     let (vote_pda, _vote_bump) = Pubkey::find_program_address(&[b"join-vote", &[proposal_index], fund_account_info.key.as_ref()], program_id);
//     let (rent_pda, _rent_bump) = Pubkey::find_program_address(&[b"rent"], program_id);

//     if *fund_account_info.key != fund_pda ||
//        *joiner_pda_info.key != joiner_pda ||
//        *join_proposal_aggregator_info.key != join_aggregator_pda ||
//        *vote_account_info.key != vote_pda ||
//        *rent_reserve_info.key != rent_pda {
//         msg!("[FUND-ERROR] {} {} Given PDAs doesn't match with the derived ones(Wrong accounts provided).", fund_account_info.key.to_string(), joiner_account_info.key.to_string());
//         return Err(FundError::InvalidAccountData.into());
//        }

//     let fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
//     if fund_data.is_private == 0 {
//         msg!("[FUND-ERROR] {} {} This fund is not private.", fund_account_info.key.to_string(), joiner_account_info.key.to_string());
//         return Err(FundError::InvalidFundDetails.into());
//     }

//     let is_joiner_member = fund_data
//        .members
//        .iter()
//        .any(|member| *member == *joiner_account_info.key);

//     if is_joiner_member {
//         msg!("[FUND-ERROR] {} {} You are already a member of this fund.", fund_account_info.key.to_string(), joiner_account_info.key.to_string());
//         return Err(FundError::AlreadyMember.into());
//     }

//     let mut joiner_data = UserAccount::try_from_slice(&joiner_pda_info.data.borrow())?;
//     let user_specific = joiner_data
//         .funds
//         .iter()
//         .find(|user_specific| user_specific.fund == *fund_account_info.key)
//         .ok_or(FundError::InvalidAccountData)?;

//     if !user_specific.is_pending {
//         msg!("[FUND-ERROR] {} {} You are already a member of this fund.", fund_account_info.key.to_string(), joiner_account_info.key.to_string());
//         return Err(FundError::AlreadyMember.into());
//     }

//     let mut join_proposal_data = JoinProposalAggregator::try_from_slice(&join_proposal_aggregator_info.data.borrow())?;
//     let (_matched_index, proposal) = join_proposal_data
//         .join_proposals
//         .iter()
//         .enumerate()
//         .find(|(_, proposal)| proposal.proposal_index == proposal_index)
//         .ok_or(FundError::InvalidAccountData)?;

//     if proposal.joiner != *joiner_account_info.key {
//         msg!("[FUND-ERROR] {} {} Wrong joiner/proposer wallet address.", fund_account_info.key.to_string(), joiner_account_info.key.to_string());
//         return Err(FundError::InvalidProposerInfo.into());
//     }

//     join_proposal_data.join_proposals.retain(|proposal| proposal.proposal_index != proposal_index);

//     let rent = Rent::get()?;
//     let current_aggregator_size = join_proposal_aggregator_info.data_len();
//     let new_aggregator_size = current_aggregator_size - 57;
//     let current_aggregator_rent = join_proposal_aggregator_info.lamports();
//     let new_aggregator_rent = rent.minimum_balance(new_aggregator_size);

//     join_proposal_aggregator_info.realloc(new_aggregator_size, false)?;

//     if new_aggregator_rent < current_aggregator_rent {
//         **join_proposal_aggregator_info.lamports.borrow_mut() -= current_aggregator_rent - new_aggregator_rent;
//         **joiner_account_info.lamports.borrow_mut() += current_aggregator_rent - new_aggregator_rent;
//     }

//     join_proposal_data.serialize(&mut &mut join_proposal_aggregator_info.data.borrow_mut()[..])?;

//     // Delete Vote account
//     let lamports = vote_account_info.lamports();
//     **rent_reserve_info.lamports.borrow_mut() += lamports;
//     **vote_account_info.lamports.borrow_mut() -= lamports;

//     let mut data = vote_account_info.data.borrow_mut();
//     for byte in data.iter_mut() {
//         *byte = 0;
//     }

//     // remove fund data from joiner pda
//     joiner_data.funds.retain(|user_specific| user_specific.fund != *fund_account_info.key);
//     let current_joiner_size = joiner_pda_info.data_len();
//     let new_joiner_size = current_joiner_size - 50;
//     let current_joiner_rent = joiner_pda_info.lamports();
//     let new_joiner_rent = rent.minimum_balance(new_joiner_size);

//     joiner_pda_info.realloc(new_joiner_size, false)?;

//     if new_joiner_rent < current_joiner_rent {
//         **joiner_pda_info.lamports.borrow_mut() -= current_joiner_rent - new_joiner_rent;
//         **joiner_account_info.lamports.borrow_mut() += current_joiner_rent - new_joiner_rent;
//     }

//     joiner_data.serialize(&mut &mut joiner_pda_info.data.borrow_mut()[..])?;

//     Ok(())
// }


// fn process_add_member<'a>(
//     program_id: &Pubkey,
//     accounts: &'a [AccountInfo<'a>],
//     fund_name: String,
//     proposal_index: u8,
// ) -> ProgramResult {
//     let current_time = Clock::get()?.unix_timestamp;

//     let accounts_iter = &mut accounts.iter();
//     let fund_account_info = next_account_info(accounts_iter)?; // Fund Account ..................................
//     let member_account_info = next_account_info(accounts_iter)?; // User to be added ............................
//     let system_program_info = next_account_info(accounts_iter)?; // System Program ..............................
//     let user_account_info = next_account_info(accounts_iter)?; // User Global identity account ..................
//     let rent_reserve_info = next_account_info(accounts_iter)?; // peerfund's rent reserve .......................

//     // User should be signer
//     if !member_account_info.is_signer {
//         msg!("[FUND-ERROR] {} {} Wrong signer!(must be your wallet)", fund_account_info.key.to_string(), member_account_info.key.to_string());
//         return Err(FundError::MissingRequiredSignature.into());
//     }

//     // Derive PDAs and check if it is same as provided in accounts
//     let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
//     let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", member_account_info.key.as_ref()], program_id);
//     let (rent_pda, _rent_bump) = Pubkey::find_program_address(&[b"rent"], program_id);
//     if *fund_account_info.key != fund_pda || *user_account_info.key != user_pda || *rent_reserve_info.key != rent_pda {
//         msg!("[FUND-ERROR] {} {} Given PDAs doesn't match with the derived ones(Wrong accounts provided).", fund_account_info.key.to_string(), member_account_info.key.to_string());
//         return Err(FundError::InvalidAccountData.into());
//     }

//     // Check if Fund exist or not!
//     if fund_account_info.data_is_empty() {
//         msg!("[FUND-ERROR] {} {} This fund does not exists..", fund_account_info.key.to_string(), member_account_info.key.to_string());
//         return Err(FundError::InvalidAccountData.into());
//     }

//     // Deserialize User Data and check if User is already a member of provided Fund
//     let mut user_data = UserAccount::try_from_slice(&user_account_info.data.borrow())?;
//     if user_data.funds.iter().any(|entry| entry.fund == *fund_account_info.key && !entry.is_pending) {
//         msg!("[FUND-ERROR] {} {} User is already a member.", fund_account_info.key.to_string(), member_account_info.key.to_string());
//         return Err(FundError::AlreadyMember.into());
//     }

//     // Deserialize the fund data
//     let mut fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
//     let total_voting_power = fund_data.total_deposit;

//     if fund_data.expected_members <= fund_data.members.len() as u32 {
//         msg!("[FUND-ERROR] {} {} Fund already has more member than expected members.", fund_account_info.key.to_string(), member_account_info.key.to_string());
//         return Err(FundError::FundAlreadyFull.into());
//     }

//     let rent = Rent::get()?;

//     if fund_data.is_private == 0 {
//         // Calculate if more lamports are needed for reallocation of User Global Account
//         let user_current_size = user_account_info.data_len();
//         let user_new_size = user_current_size + 50;
//         let user_new_min_balance = rent.minimum_balance(user_new_size);
//         let user_current_balance = user_account_info.lamports();
//         if user_new_min_balance > user_current_balance {
//             invoke(
//                 &system_instruction::transfer(
//                     member_account_info.key,
//                     user_account_info.key,
//                     user_new_min_balance - user_current_balance,
//                 ),
//                 &[member_account_info.clone(), user_account_info.clone(), system_program_info.clone()],
//             )?;
//         }

//         // Reallocate new bytes ofr storage of new Fund details
//         user_account_info.realloc(user_new_size, false)?;
//         user_data.funds.push(UserSpecific {
//             fund: *fund_account_info.key,
//             governance_token_balance: 0 as u64,
//             is_pending: false,
//             is_eligible: true,
//             join_time: current_time
//         });
//         user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;
//     }


//     // Check if user already exists
//     if fund_data.members.contains(member_account_info.key) {
//         msg!("[FUND-ERROR] {} {} User is already in the fund.", fund_account_info.key.to_string(), member_account_info.key.to_string());
//         return Err(FundError::InvalidAccountData.into());
//     }

//     // Calculate if more lamports are needed for reallocation of Fund Account
//     let fund_current_size = fund_account_info.data_len();
//     let fund_new_size = fund_current_size + 32;
//     let fund_new_min_balance = rent.minimum_balance(fund_new_size);
//     let fund_current_balance = fund_account_info.lamports();

//     if fund_new_min_balance > fund_current_balance {
//         invoke(
//             &system_instruction::transfer(
//                 member_account_info.key,
//                 fund_account_info.key,
//                 fund_new_min_balance - fund_current_balance
//             ),
//             &[member_account_info.clone(), fund_account_info.clone(), system_program_info.clone()]
//         )?;
//     }

//     // Reallocate new bytes for storage of new member pubkey
//     fund_account_info.realloc(fund_new_size, false)?;
//     fund_data.members.push(*member_account_info.key);

//     // Refund logic
//     let mut refund_per_member: u64 = (rent.minimum_balance(327 + fund_name.len()) + 5760000 as u64) / (fund_data.expected_members as u64);
//     if fund_data.is_refunded {
//         refund_per_member = 1_500_000;
//     }

//     invoke(
//         &system_instruction::transfer(
//             member_account_info.key,
//             rent_reserve_info.key,
//             refund_per_member
//         ),
//         &[member_account_info.clone(), rent_reserve_info.clone(), system_program_info.clone()]
//     )?;

//     // If fund is private
//     if fund_data.is_private == 1 as u8 {
//         let join_proposal_aggregator_info = next_account_info(accounts_iter)?;
//         let vote_account_info = next_account_info(accounts_iter)?;
//         let index = 0 as u8;
//         let (join_proposal_pda, _join_proposal_bump) = Pubkey::find_program_address(&[b"join-proposal-aggregator", &[index], fund_pda.as_ref()], program_id);
//         let (vote_pda, _vote_bump) = Pubkey::find_program_address(&[b"join-vote", &[proposal_index], fund_pda.as_ref()], program_id);
//         if *join_proposal_aggregator_info.key != join_proposal_pda {
//             msg!("[FUND-ERROR] {} {} Wrong join proposal information.", fund_account_info.key.to_string(), member_account_info.key.to_string());
//             return Err(FundError::InvalidProposalAccount.into());
//         }
//         if *vote_account_info.key != vote_pda {
//             msg!("[FUND-ERROR] {} {} Wrong vote account information.", fund_account_info.key.to_string(), member_account_info.key.to_string());
//             return Err(FundError::InvalidVoteAccount.into());
//         }

//         let mut join_proposal_data = JoinProposalAggregator::try_from_slice(&join_proposal_aggregator_info.data.borrow())?;
//         let (_matched_index, proposal) = join_proposal_data
//             .join_proposals
//             .iter()
//             .enumerate()
//             .find(|(_, p)| p.proposal_index == proposal_index)
//             .ok_or(FundError::InvalidAccountData)?;

//         if proposal.joiner != *member_account_info.key {
//             msg!("[FUND-ERROR] {} {} Wrong proposer/joiner wallet.", fund_account_info.key.to_string(), member_account_info.key.to_string());
//             return Err(FundError::InvalidAccountData.into());
//         }

//         if 2*(proposal.votes_yes) < total_voting_power {
//             msg!("[FUND-ERROR] {} {} Not enough votes to join the fund.", fund_account_info.key.to_string(), member_account_info.key.to_string());
//             return Err(FundError::NotEnoughVotes.into());
//         }

//         join_proposal_data.join_proposals.retain(|proposal| proposal.proposal_index != proposal_index);

//         let rent = Rent::get()?;
//         let current_proposal_aggregator_size = join_proposal_aggregator_info.data_len();
//         let new_proposal_aggregator_size = current_proposal_aggregator_size - 57;
//         let current_aggregator_rent = join_proposal_aggregator_info.lamports();
//         let new_aggregator_rent = rent.minimum_balance(new_proposal_aggregator_size);

//         join_proposal_aggregator_info.realloc(new_proposal_aggregator_size, false)?;
//         if new_aggregator_rent < current_aggregator_rent {
//             **join_proposal_aggregator_info.lamports.borrow_mut() -= current_aggregator_rent - new_aggregator_rent;
//             **member_account_info.lamports.borrow_mut() += current_aggregator_rent - new_aggregator_rent;
//         }

//         join_proposal_data.serialize(&mut &mut join_proposal_aggregator_info.data.borrow_mut()[..])?;

//         let lamports = vote_account_info.lamports();
//         **rent_reserve_info.lamports.borrow_mut() += lamports;
//         **vote_account_info.lamports.borrow_mut() -= lamports;

//         let mut data = vote_account_info.data.borrow_mut();
//         for byte in data.iter_mut() {
//             *byte = 0;
//         }

//         if let Some(user_specific) = user_data
//             .funds
//             .iter_mut()
//             .find(|entry| entry.fund == *fund_account_info.key) {
//                 user_specific.is_pending = false;
//                 user_specific.is_eligible = true;
//             } else {
//                 msg!("User is not a member in this fund");
//                 return Err(FundError::InvalidAccountData.into());
//             }

//         user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;
//     }

//     // If expected number of members are achieved, refund back to creator
//     if fund_data.expected_members == fund_data.members.len() as u32 && !fund_data.is_refunded {
//         let fund_creator_info = next_account_info(accounts_iter)?;

//         if *fund_creator_info.key != fund_data.members[0] {
//             msg!("[FUND-ERROR] {} {} Wrong fund creator.", fund_account_info.key.to_string(), member_account_info.key.to_string());
//             return Err(FundError::InvalidFundCreator.into());
//         }

//         let rent_paid_by_creator = rent.minimum_balance(327 + fund_name.len()) + 5760000 as u64;

//         let refund_to_creator: u64 = ((fund_data.expected_members - 1) as u64 * rent_paid_by_creator) / (fund_data.expected_members as u64);

//         **rent_reserve_info.lamports.borrow_mut() -= refund_to_creator;
//         **fund_creator_info.lamports.borrow_mut() += refund_to_creator;
//         fund_data.is_refunded = true;
//     }

//     fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

//     msg!("[FUND-ACTIVITY] {} {} {} Member joined: {}", fund_account_info.key.to_string(), current_time, fund_name, member_account_info.key.to_string());

//     Ok(())

// }

fn process_init_deposit_token(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    is_unwrapped_sol: u8,
    amount: u64,
    mint_amount: u64,
    fund_name: String,
    fund_type: u8,
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
    // let governance_token_account_info = next_account_info(accounts_iter)?; // Governance Token Account of depositor ..
    // let governance_mint_info = next_account_info(accounts_iter)?; // Governance Mint Account .........................
    // let token_program_2022_info = next_account_info(accounts_iter)?; // token program 2022 ...........................

    // Depositor should be signer
    if !member_account_info.is_signer {
        msg!("[FUND-ERROR] {} {} Wrong signer!(must be your wallet)", fund_account_info.key.to_string(), member_account_info.key.to_string());
        return Err(FundError::MissingRequiredSignature.into());
    }

    // Derive the PDAs and check for equality with provided ones
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault", fund_account_info.key.as_ref()], program_id);
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[b"user", member_account_info.key.as_ref()], program_id);
    let mut seed = String::from("light-fund");
    if fund_type == 1 {
        seed = String::from("standard-fund");
    } else if fund_type == 2 {
        seed = String::from("dao-fund");
    }
    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[seed.as_bytes(), fund_name.as_bytes()], program_id);
    if *vault_account_info.key != vault_pda || *fund_account_info.key != fund_pda || *user_account_info.key != user_pda {
        msg!("[FUND-ERROR] {} {} Given PDAs doesn't match with the derived ones(Wrong accounts provided).", fund_account_info.key.to_string(), member_account_info.key.to_string());
        return Err(FundError::InvalidAccountData.into());
    }

    // let expected_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
    //     member_account_info.key,
    //     governance_mint_info.key,
    //     token_program_2022_info.key
    // );
    // if *governance_token_account_info.key != expected_ata {
    //     msg!("[FUND-ERROR] {} {} Wrong governance token account information.", fund_account_info.key.to_string(), member_account_info.key.to_string());
    //     return Err(FundError::InvalidTokenAccount.into());
    // }

    let mut fund_data = LightFundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    let is_depositer_member = fund_data
        .members
        .iter()
        .any(|member| member.0 == *member_account_info.key);

    if !is_depositer_member {
        msg!("[FUND-ERROR] {} {} You are not a member of this fund and so cannot deposit in it.", fund_account_info.key.to_string(), member_account_info.key.to_string());
        return Err(FundError::NotAFundMember.into());
    }

    // If user doesn't exist in the fund, it can't deposit
    let mut user_data = UserAccount::try_from_slice(&user_account_info.data.borrow())?;
    if !user_data.funds.iter().any(|entry| entry.fund == *fund_account_info.key) {
        msg!("[FUND-ERROR] {} {} User is not a member of the fund.", fund_account_info.key.to_string(), member_account_info.key.to_string());
        return Err(FundError::InvalidAccountData.into());
    }

    // If depositor's governance token account doesn't exist, create one
    // if governance_token_account_info.data_is_empty() {
    //     invoke(
    //         &spl_associated_token_account::instruction::create_associated_token_account(
    //             member_account_info.key,
    //             member_account_info.key,
    //             governance_mint_info.key,
    //             token_program_2022_info.key,
    //         ),
    //         &[
    //             member_account_info.clone(),
    //             governance_token_account_info.clone(),
    //             token_program_2022_info.clone(),
    //             governance_mint_info.clone(),
    //             rent_sysvar_info.clone(),
    //         ]
    //     )?;
    // }

    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let rent_req = rent.minimum_balance(TokenAccount::LEN);

    // If vault's token account account for the depositing mint doesn't exist, create it
    if vault_ata_info.data_is_empty() {

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

    // let mint: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

    if is_unwrapped_sol == 1 {
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
    // invoke_signed(
    //     &spl_token_2022::instruction::mint_to(
    //         token_program_2022_info.key,
    //         governance_mint_info.key,
    //         governance_token_account_info.key,
    //         fund_account_info.key,
    //         &[],
    //         mint_amount,
    //     )?,
    //     &[
    //         governance_mint_info.clone(),
    //         governance_token_account_info.clone(),
    //         fund_account_info.clone(),
    //         token_program_2022_info.clone(),
    //     ],
    //     &[&[b"fund", fund_name.as_bytes(), &[fund_bump]]],
    // )?;

    // In vault account, set the last deposit time
    let mut vault_data = VaultAccount::try_from_slice(&vault_account_info.data.borrow())?;
    vault_data.last_deposit_time = current_time;
    vault_data.serialize(&mut &mut vault_account_info.data.borrow_mut()[..])?;

    // In fund account increase the deposited amount (unit lamports)
    fund_data.total_deposit += mint_amount;
    fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

    if let Some(entry) = user_data.funds.iter_mut().find(|entry| entry.fund == *fund_account_info.key) {
        entry.governance_token_balance += mint_amount;
    } else {
        msg!("[FUND-ERROR] {} {} Fund entry not found for user.", fund_account_info.key.to_string(), member_account_info.key.to_string());
        return Err(FundError::InvalidAccountData.into());
    }

    user_data.serialize(&mut &mut user_account_info.data.borrow_mut()[..])?;

    msg!("[FUND-ACTIVITY] {} {} {} Token deposit: {} of {} by {}", fund_account_info.key.to_string(), current_time, fund_name, amount, mint_account_info.key.to_string(), member_account_info.key.to_string());

    Ok(())
}


fn process_init_investment_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: String,
    cid: String,
    deadline: i64,
    merkel_bytes: MerkleRoot,
) -> ProgramResult {
    let creation_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let proposer_account_info = next_account_info(accounts_iter)?;
    let fund_account_info = next_account_info(accounts_iter)?;
    let proposal_aggregator_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    let new_proposal_aggregator_info = next_account_info(accounts_iter)?;

    msg!("{} {} {}", cid, deadline, fund_name);

    if !proposer_account_info.is_signer {
        msg!("[FUND-ERROR] {} {} Wrong signer!(must be your wallet)", fund_account_info.key.to_string(), proposer_account_info.key.to_string());
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"light-fund", fund_name.as_bytes()], program_id);
    msg!("Fund PDA derived");
    let mut fund_data = LightFundAccount::try_from_slice(&fund_account_info.data.borrow())?;

    let is_proposer_member = fund_data
        .members
        .iter()
        .any(|member| member.0 == *proposer_account_info.key);

    if !is_proposer_member {
        msg!("[FUND-ERROR] {} {} You are not a member of this fund and so cannot create a proposal.", fund_account_info.key.to_string(), proposer_account_info.key.to_string());
        return Err(FundError::NotAFundMember.into());
    }

    if *fund_account_info.key != fund_pda {
        msg!("[FUND-ERROR] {} {} Wrong fund account information.", fund_account_info.key.to_string(), proposer_account_info.key.to_string());
        return Err(FundError::InvalidAccountData.into());
    }

    let current_index = fund_data.current_proposal_index;

    let (proposal_aggregator_pda, _proposal_aggregator_bump) = Pubkey::find_program_address(
        &[
            b"proposal-aggregator",
            &[current_index],
            fund_pda.as_ref()
        ],
        program_id
    );

    msg!("Current Derived");

    let (new_proposal_aggregator_pda, new_proposal_aggregator_bump) = Pubkey::find_program_address(
        &[
            b"proposal-aggregator",
            &[current_index + 1],
            fund_pda.as_ref()
        ],
        program_id
    );

    msg!("New Derived");

    if *proposal_aggregator_info.key != proposal_aggregator_pda || *new_proposal_aggregator_info.key != new_proposal_aggregator_pda {
        msg!("[FUND-ERROR] {} {} Given PDAs doesn't match with the derived ones(Wrong accounts provided).", fund_account_info.key.to_string(), proposer_account_info.key.to_string());
        return Err(FundError::InvalidProposalAccount.into());
    }

    // Rent Calculation
    let rent = Rent::get()?;
    let current_proposal_space = proposal_aggregator_info.data_len();
    let extra_proposal_space = 169 as usize;

    let mut proposal_aggregator_data = ProposalAggregatorAccount::try_from_slice(&proposal_aggregator_info.data.borrow())?;

    let mut voters_bitmap: Vec<(u32, u8)> = vec![];
    let proposer_info = fund_data
        .members
        .iter()
        .find(|member| member.0 == *proposer_account_info.key)
        .ok_or(FundError::InvalidAccountData)?;

    let proposer_vec_index = proposer_info.1;

    voters_bitmap.push((proposer_vec_index, 1));

    let bytes = cid.as_bytes();
    let mut array = [0u8; 59];
    let len = bytes.len().min(59);
    array[..len].copy_from_slice(&bytes[..len]);

    // let merkel_bytes = merkel_root.as_bytes();
    // let mut merkel_array = [0u8; 32];
    // let merkel_len = merkel_bytes.len().min(32);
    // merkel_array[..merkel_len].copy_from_slice(&merkel_bytes[..merkel_len]);

    // check if new proposal aggregator is required
    if (current_proposal_space + extra_proposal_space) > 10240 as usize {
        msg!("New is creating");
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
                &[new_proposal_aggregator_bump]
            ]]
        )?;

        let proposals_vec: Vec<Proposal> = vec![ Proposal {
            proposer: *proposer_account_info.key,
            cid: array,
            merkel_root: merkel_bytes.0,
            votes_yes: 1 as u64,
            votes_no: 0 as u64,
            creation_time,
            deadline,
            executed: 0 as u8,
            vec_index: 0 as u16,
            swaps_status: 0 as u16,
            voters_bitmap,
        }];

        let new_proposal_data = ProposalAggregatorAccount {
            index: current_index + 1,
            proposals: proposals_vec
        };

        new_proposal_data.serialize(&mut &mut new_proposal_aggregator_info.data.borrow_mut()[..])?;

        fund_data.current_proposal_index += 1;
        fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

        msg!("[FUND-ACTIVITY] {} {} {} Proposal created: ({}, 0) by {}", fund_account_info.key.to_string(), creation_time, fund_name, (current_index + 1), proposer_account_info.key.to_string());
    } else {
        msg!("Current is enough");
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

        let num_of_proposals = proposal_aggregator_data.proposals.len();
        let mut vec_index = 0 as u16;
        if num_of_proposals != 0 {
            vec_index = proposal_aggregator_data.proposals[num_of_proposals - 1].vec_index + 1;
        }
        msg!("Vec Index: {}", vec_index);

        proposal_aggregator_info.realloc(new_aggregator_size, false)?;
        
        proposal_aggregator_data.proposals.push( Proposal {
            proposer: *proposer_account_info.key,
            cid: array,
            merkel_root: merkel_bytes.0,
            votes_yes: 1 as u64,
            votes_no: 0 as u64,
            creation_time,
            deadline,
            executed: 0 as u8,
            vec_index,
            swaps_status: 0 as u16,
            voters_bitmap,
        });

        proposal_aggregator_data.serialize(&mut &mut proposal_aggregator_info.data.borrow_mut()[..])?;

        msg!("[FUND-ACTIVITY] {} {} {} Proposal created: ({}, {}) by {}", fund_account_info.key.to_string(), creation_time, fund_name, current_index, vec_index, proposer_account_info.key.to_string());
    }

    Ok(())
}


fn process_vote_on_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vote: u8,
    proposal_index: u8,
    vec_index: u16,
    fund_name: String,
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let voter_account_info = next_account_info(accounts_iter)?; // Voter's Wallet ................................
    let proposal_aggregator_info = next_account_info(accounts_iter)?; // Proposal account ........................
    let fund_account_info = next_account_info(accounts_iter)?; // fund Account ...................................
    let proposer_account_info = next_account_info(accounts_iter)?; // Proposer Wallet ............................
    let system_program_info = next_account_info(accounts_iter)?; // System Program ...............................

    msg!("Vec Index: {}", vec_index);

    // Voter needs to be signer
    if !voter_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    // Pdas derivation
    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"light-fund", fund_name.as_bytes()], program_id);
    let (proposal_aggregator_pda, _proposal_aggregator_bump) = Pubkey::find_program_address(&[b"proposal-aggregator", &[proposal_index], fund_account_info.key.as_ref()], program_id);

    // Pdas verification
    if *fund_account_info.key != fund_pda ||
       *proposal_aggregator_info.key != proposal_aggregator_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let fund_data = LightFundAccount::try_from_slice(&fund_account_info.data.borrow())?;

    let is_voter_member = fund_data
        .members
        .iter()
        .any(|member| member.0 == *voter_account_info.key);

    if !is_voter_member {
        return Err(FundError::NotAFundMember.into());
    }

    let voter_info = fund_data
        .members
        .iter()
        .find(|member| member.0 == *voter_account_info.key)
        .ok_or(FundError::InvalidAccountData)?;

    let voter_vec_index = voter_info.1;    

    let mut proposal_aggregator_data = ProposalAggregatorAccount::try_from_slice(&proposal_aggregator_info.data.borrow())?;
    let (matched_index, proposal) = proposal_aggregator_data
        .proposals
        .iter()
        .enumerate()
        .find(|(_, proposal)| proposal.vec_index == vec_index)
        .ok_or(FundError::InvalidProposalAccount)?;

    if proposal.proposer != *proposer_account_info.key {
        return Err(FundError::InvalidProposerInfo.into());
    }

    if proposal.deadline < current_time {
        return Err(FundError::VotingCeased.into());
    }

    if proposal_aggregator_data.proposals[matched_index].voters_bitmap.iter().any(|voter| voter.0 == voter_vec_index) {
        return Err(FundError::AlreadyVoted.into());
    }

    proposal_aggregator_data.proposals[matched_index].voters_bitmap.push((voter_vec_index, vote));

    if vote == 0 {
        proposal_aggregator_data.proposals[matched_index].votes_no += 1;
    } else {
        proposal_aggregator_data.proposals[matched_index].votes_yes += 1;
    }

    let rent = Rent::get()?;
    let current_aggregator_space = proposal_aggregator_info.data_len();
    let new_proposal_space = current_aggregator_space + 5;
    let current_proposal_rent = proposal_aggregator_info.lamports();
    let new_proposal_rent = rent.minimum_balance(new_proposal_space);

    if new_proposal_rent > current_proposal_rent {
        invoke(
            &system_instruction::transfer(
                voter_account_info.key,
                proposal_aggregator_info.key,
                new_proposal_rent - current_proposal_rent
            ),
            &[voter_account_info.clone(), proposal_aggregator_info.clone(), system_program_info.clone()]
        )?;
    }

    proposal_aggregator_info.realloc(new_proposal_space, false)?;
    proposal_aggregator_data.serialize(&mut &mut proposal_aggregator_info.data.borrow_mut()[..])?;

    msg!("[FUND-ACTIVITY] {} {} {} Vote: {} on proposal ({}, {})", fund_account_info.key.to_string(), current_time, fund_name, voter_account_info.key.to_string(), proposal_index, vec_index);

    Ok(())
}


// fn process_init_investment_proposal(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     amounts: Vec<u64>,
//     slippages: Vec<u16>,
//     deadline: i64,
//     fund_name: String,
// ) -> ProgramResult {
//     let creation_time = Clock::get()?.unix_timestamp;

//     let accounts_iter = &mut accounts.iter();
//     let proposer_account_info = next_account_info(accounts_iter)?; // Proposer Wallet ............................
//     let fund_account_info = next_account_info(accounts_iter)?; // Fund Account ...................................
//     let proposal_aggregator_info = next_account_info(accounts_iter)?; // Proposal Account ........................
//     let system_program_info = next_account_info(accounts_iter)?; // System Program ...............................
//     let new_proposal_aggregator_info = next_account_info(accounts_iter)?; // new proposal aggregator account .....
//     let vote_account_info = next_account_info(accounts_iter)?; // vote account 1 .................................
//     let new_vote_account_info = next_account_info(accounts_iter)?; // vote account 2 .............................
//     let proposer_token_account_info = next_account_info(accounts_iter)?;// 
//     let governance_mint_info = next_account_info(accounts_iter)?;
//     let token_program_2022_info = next_account_info(accounts_iter)?;

//     // Proposer needs to be signer
//     if !proposer_account_info.is_signer {
//         msg!("[FUND-ERROR] {} {} Wrong signer!(must be your wallet)", fund_account_info.key.to_string(), proposer_account_info.key.to_string());
//         return Err(FundError::MissingRequiredSignature.into());
//     }

//     // Derive PDAs and check for equality with the provided ones
//     let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
//     let mut fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;

//     let token_account = get_associated_token_address_with_program_id(
//         proposer_account_info.key,
//         governance_mint_info.key,
//         token_program_2022_info.key
//     );

//     if token_account != *proposer_token_account_info.key {
//         msg!("[FUND-ERROR] {} {} Wrong governance token account information.", fund_account_info.key.to_string(), proposer_account_info.key.to_string());
//         return Err(FundError::InvalidTokenAccount.into());
//     }

//     if proposer_token_account_info.data_is_empty() {
//         msg!("[FUND-ERROR] {} {} Governance token account does not exists.", fund_account_info.key.to_string(), proposal_aggregator_info.key.to_string());
//         return Err(FundError::InvalidAccountData.into());
//     }

//     let token_account_data = proposer_token_account_info.try_borrow_data()?;
//     let token_account = spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(&token_account_data)?;
//     let base_token_account = token_account.base;
//     let balance = base_token_account.amount;

//     if balance == 0 {
//         msg!("[FUND-ERROR] {} {} You have no voting power(contribution) in this fund.", fund_account_info.key.to_string(), proposal_aggregator_info.key.to_string());
//         return Err(FundError::NoVotingPower.into());
//     }

//     let is_proposer_member = fund_data
//         .members
//         .iter()
//         .any(|member| *member == *proposer_account_info.key);

//     if !is_proposer_member {
//         msg!("[FUND-ERROR] {} {} You are not a member of this fund and so cannot create a proposal.", fund_account_info.key.to_string(), proposer_account_info.key.to_string());
//         return Err(FundError::NotAFundMember.into());
//     }

//     let current_index = fund_data.current_proposal_index;

//     if *fund_account_info.key != fund_pda {
//         msg!("[FUND-ERROR] {} {} Wrong fund account information.", fund_account_info.key.to_string(), proposer_account_info.key.to_string());
//         return Err(FundError::InvalidAccountData.into());
//     }

//     let (proposal_aggregator_pda, _proposal_aggregator_bump) = Pubkey::find_program_address(
//         &[
//             b"proposal-aggregator",
//             &[current_index],
//             fund_pda.as_ref()
//         ],
//         program_id
//     );

//     let (new_proposal_aggregator_pda, new_proposal_aggregator_bump) = Pubkey::find_program_address(
//         &[
//             b"proposal-aggregator",
//             &[current_index + 1],
//             fund_pda.as_ref()
//         ],
//         program_id
//     );

//     if *proposal_aggregator_info.key != proposal_aggregator_pda || *new_proposal_aggregator_info.key != new_proposal_aggregator_pda {
//         msg!("[FUND-ERROR] {} {} Given PDAs doesn't match with the derived ones(Wrong accounts provided).", fund_account_info.key.to_string(), proposer_account_info.key.to_string());
//         return Err(FundError::InvalidProposalAccount.into());
//     }

//     // Extract From Assets Mint
//     let from_assets_info : Vec<&AccountInfo> = accounts_iter
//         .take(amounts.len())
//         .collect();
//     if from_assets_info.len() != amounts.len() {
//         msg!("[FUND-ERROR] {} {} Wrong information of number of assets to trade of.", fund_account_info.key.to_string(), proposer_account_info.key.to_string());
//         return Err(FundError::InvalidAccountData.into());
//     }
//     let from_assets_mints: Vec<Pubkey> = from_assets_info.iter().map(|m| *m.key).collect();

//     // Extract To Assets Mint
//     let to_assets_info: Vec<&AccountInfo> = accounts_iter
//         .take(amounts.len())
//         .collect();
//     if to_assets_info.len() != amounts.len() {
//         msg!("[FUND-ERROR] {} {} Wrong information of number of assets to trade with.", fund_account_info.key.to_string(), proposer_account_info.key.to_string());
//         return Err(FundError::InvalidAccountData.into());
//     }
//     let to_assets_mints: Vec<Pubkey> = to_assets_info.iter().map(|m| *m.key).collect();

//     // Rent Calculation
//     let rent = Rent::get()?;
//     let extra_proposal_space = (83 + to_assets_info.len()*74) as usize;
//     let current_proposal_space = proposal_aggregator_info.data_len();

//     let mut proposal_aggregator_data = ProposalAggregatorAccount::try_from_slice(&proposal_aggregator_info.data.borrow())?;

//     // check if new proposal aggregator is required
//     if (current_proposal_space + extra_proposal_space) > 10240 as usize {
//         // Create Proposal Account
//         invoke_signed(
//             &system_instruction::create_account(
//                 proposer_account_info.key,
//                 new_proposal_aggregator_info.key,
//                 rent.minimum_balance(37 + extra_proposal_space),
//                 (37 + extra_proposal_space) as u64,
//                 program_id
//             ),
//             &[
//                 new_proposal_aggregator_info.clone(),
//                 proposer_account_info.clone(),
//                 system_program_info.clone(),
//             ],
//             &[&[
//                 b"proposal-aggregator",
//                 &[current_index + 1],
//                 fund_account_info.key.as_ref(),
//                 &[new_proposal_aggregator_bump]
//             ]]
//         )?;

//         let proposals_vec: Vec<Proposal> = vec![ Proposal {
//             proposer: *proposer_account_info.key,
//             from_assets: from_assets_mints,
//             to_assets: to_assets_mints,
//             amounts,
//             slippages,
//             votes_yes: balance,
//             votes_no: 0 as u64,
//             creation_time,
//             deadline,
//             executed: false,
//             vec_index: 0 as u16
//         }];

//         let new_proposal_data = ProposalAggregatorAccount {
//             fund: *fund_account_info.key,
//             index: current_index + 1,
//             proposals: proposals_vec
//         };

//         new_proposal_data.serialize(&mut &mut new_proposal_aggregator_info.data.borrow_mut()[..])?;

//         fund_data.current_proposal_index += 1;
//         fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;
        
//         let new_vec_index = 0 as u16;
//         let (new_vote_pda, new_vote_bump) = Pubkey::find_program_address(&[b"vote", &[current_index + 1], &new_vec_index.to_le_bytes(), fund_account_info.key.as_ref()], program_id);
//         if *new_vote_account_info.key != new_vote_pda {
//             msg!("[FUND-ERROR] {} {} Invalid new vote account data.", fund_account_info.key.to_string(), proposer_account_info.key.to_string());
//             return Err(FundError::InvalidVoteAccount.into());
//         }
//         if !new_vote_account_info.data_is_empty() {
//             msg!("Wrong Vote Account");
//             return Err(FundError::InvalidVoteAccount.into());
//         }

//         let vote_space = 40 as usize;
//         let vote_rent = rent.minimum_balance(vote_space);

//         invoke_signed(
//             &system_instruction::create_account(
//                 proposer_account_info.key,
//                 new_vote_account_info.key,
//                 vote_rent,
//                 vote_space as u64,
//                 program_id
//             ),
//             &[proposer_account_info.clone(), new_vote_account_info.clone(), system_program_info.clone()],
//             &[&[b"vote", &[current_index + 1], &new_vec_index.to_le_bytes(), fund_account_info.key.as_ref(), &[new_vote_bump]]]
//         )?;

//         let new_voters_vec: Vec<(Pubkey, u8)> = vec![(*proposer_account_info.key, 1)];
//         let new_vote_data = VoteAccount {
//             proposal_index: current_index + 1,
//             vec_index: new_vec_index,
//             voters: new_voters_vec
//         };

//         new_vote_data.serialize(&mut &mut new_vote_account_info.data.borrow_mut()[..])?;

//         msg!("[FUND-ACTIVITY] {} {} {} Proposal created: ({}, {}) by {}", fund_account_info.key.to_string(), creation_time, fund_name, (current_index + 1), new_vec_index, proposer_account_info.key.to_string());
//     } else {
//         let new_aggregator_size = extra_proposal_space + current_proposal_space as usize;
//         let current_rent_exempt = proposal_aggregator_info.lamports();
//         let new_rent_exempt = rent.minimum_balance(new_aggregator_size);

//         if new_rent_exempt > current_rent_exempt {
//             invoke(
//                 &system_instruction::transfer(
//                     proposer_account_info.key,
//                     proposal_aggregator_info.key,
//                     new_rent_exempt-current_rent_exempt
//                 ),
//                 &[proposal_aggregator_info.clone(), proposer_account_info.clone(), system_program_info.clone()]
//             )?;
//         }

//         let num_of_proposals = proposal_aggregator_data.proposals.len();
//         let mut vec_index = 0 as u16;
//         if num_of_proposals != 0 {
//             vec_index = proposal_aggregator_data.proposals[num_of_proposals - 1].vec_index + 1;
//         }
//         msg!("Vec Index: {}", vec_index);

//         proposal_aggregator_info.realloc(new_aggregator_size, false)?;
        
//         proposal_aggregator_data.proposals.push( Proposal {
//             proposer: *proposer_account_info.key,
//             from_assets: from_assets_mints,
//             to_assets: to_assets_mints,
//             amounts,
//             slippages,
//             votes_yes: balance,
//             votes_no: 0 as u64,
//             creation_time,
//             deadline,
//             executed: false,
//             vec_index
//         });

//         proposal_aggregator_data.serialize(&mut &mut proposal_aggregator_info.data.borrow_mut()[..])?;

//         let (vote_pda, vote_bump) = Pubkey::find_program_address(&[b"vote", &[current_index], &vec_index.to_le_bytes(), fund_account_info.key.as_ref()], program_id);
//         if *vote_account_info.key != vote_pda {
//             return Err(FundError::InvalidVoteAccount.into());
//         }
//         if !vote_account_info.data_is_empty() {
//             msg!("Wrong Vote Account");
//             return Err(FundError::InvalidVoteAccount.into());
//         }

//         let vote_space = 40 as usize;
//         let vote_rent = rent.minimum_balance(vote_space);
        
//         invoke_signed(
//             &system_instruction::create_account(
//                 proposer_account_info.key,
//                 vote_account_info.key,
//                 vote_rent,
//                 vote_space as u64,
//                 program_id
//             ),
//             &[vote_account_info.clone(), proposer_account_info.clone(), system_program_info.clone()],
//             &[&[b"vote", &[current_index], &vec_index.to_le_bytes(), fund_account_info.key.as_ref(), &[vote_bump]]]
//         )?;

//         let voters: Vec<(Pubkey, u8)> = vec![(*proposer_account_info.key, 1)];

//         let vote_data = VoteAccount {
//             proposal_index: current_index,
//             vec_index,
//             voters,
//         };

//         vote_data.serialize(&mut &mut vote_account_info.data.borrow_mut()[..])?;

//         msg!("[FUND-ACTIVITY] {} {} {} Proposal created: ({}, {}) by {}", fund_account_info.key.to_string(), creation_time, fund_name, current_index, vec_index, proposer_account_info.key.to_string());
//     }

//     Ok(())
// }


// fn process_vote_on_proposal(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     vote: u8,
//     proposal_index: u8,
//     vec_index: u16,
//     fund_name: String,
// ) -> ProgramResult {
//     let current_time = Clock::get()?.unix_timestamp;

//     let accounts_iter = &mut accounts.iter();
//     let voter_account_info = next_account_info(accounts_iter)?; // Voter's Wallet ................................
//     let vote_account_info = next_account_info(accounts_iter)?; // Voter's vote account to be created .............
//     let proposal_aggregator_info = next_account_info(accounts_iter)?; // Proposal account ........................
//     let system_program_info = next_account_info(accounts_iter)?; // System Program ...............................
//     let fund_account_info = next_account_info(accounts_iter)?; // fund Account ...................................
//     let governance_token_mint_info = next_account_info(accounts_iter)?; // Governance Mint account ...............
//     let voter_token_account_info = next_account_info(accounts_iter)?; // Voter's governance token account ........
//     let token_program_2022_info = next_account_info(accounts_iter)?; // Token program extensions .................
//     let proposer_account_info = next_account_info(accounts_iter)?; // Proposer Wallet ............................

//     msg!("Vec Index: {}", vec_index);

//     // Voter needs to be signer
//     if !voter_account_info.is_signer {
//         return Err(FundError::MissingRequiredSignature.into());
//     }

//     // Pdas derivation
//     let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
//     let (vote_pda, _vote_bump) = Pubkey::find_program_address(&[b"vote", &[proposal_index], &vec_index.to_le_bytes(), fund_account_info.key.as_ref()], program_id);
//     let token_account = spl_associated_token_account::get_associated_token_address_with_program_id(
//         voter_account_info.key,
//         governance_token_mint_info.key,
//         token_program_2022_info.key
//     );

//     let (proposal_aggregator_pda, _proposal_aggregator_bump) = Pubkey::find_program_address(&[b"proposal-aggregator", &[proposal_index], fund_account_info.key.as_ref()], program_id);

//     // Pdas verification
//     if *fund_account_info.key != fund_pda ||
//        *vote_account_info.key != vote_pda ||
//        token_account != *voter_token_account_info.key ||
//        *proposal_aggregator_info.key != proposal_aggregator_pda {
//         return Err(FundError::InvalidAccountData.into());
//     }

//     let fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;

//     let is_voter_member = fund_data
//         .members
//         .iter()
//         .any(|member| *member == *voter_account_info.key);

//     if !is_voter_member {
//         return Err(FundError::NotAFundMember.into());
//     }

//     if fund_data.governance_mint != *governance_token_mint_info.key {
//         return Err(FundError::InvalidGovernanceMint.into());
//     }

//     let mut proposal_aggregator_data = ProposalAggregatorAccount::try_from_slice(&proposal_aggregator_info.data.borrow())?;
//     let (matched_index, proposal) = proposal_aggregator_data
//         .proposals
//         .iter()
//         .enumerate()
//         .find(|(_, proposal)| proposal.vec_index == vec_index)
//         .ok_or(FundError::InvalidProposalAccount)?;

//     if proposal.proposer != *proposer_account_info.key {
//         return Err(FundError::InvalidProposerInfo.into());
//     }

//     if proposal.deadline < current_time {
//         return Err(FundError::VotingCeased.into());
//     }

//     let mut vote_data = VoteAccount::try_from_slice(&vote_account_info.data.borrow())?;

//     let voter_exists = vote_data
//         .voters
//         .iter()
//         .any(|(key, _)| *key == *voter_account_info.key);

//     if voter_exists {
//         return Err(FundError::AlreadyVoted.into());
//     }

//     if voter_token_account_info.data_is_empty() {
//         msg!("No Voting Power");
//         return Err(FundError::NoVotingPower.into());
//     }

//     if vote_account_info.data_is_empty() {
//         msg!("Vote account should be already created");
//         return Err(FundError::InvalidVoteAccount.into());
//     } else {
//         let rent = Rent::get()?;
//         let extra_vote_space = 33 as usize;
//         let current_vote_space = vote_account_info.data_len();
//         let new_vote_space = current_vote_space + extra_vote_space;
//         let new_rent = rent.minimum_balance(new_vote_space);
//         let current_rent = vote_account_info.lamports();

//         if new_rent > current_rent {
//             invoke(
//                 &system_instruction::transfer(
//                     voter_account_info.key,
//                     vote_account_info.key,
//                     new_rent - current_rent
//                 ),
//                 &[system_program_info.clone(), voter_account_info.clone(), vote_account_info.clone()]
//             )?;
//         }

//         vote_account_info.realloc(new_vote_space, false)?;
        
//         vote_data.voters.push((*voter_account_info.key, vote));
//         vote_data.serialize(&mut &mut vote_account_info.data.borrow_mut()[..])?;

//         let token_account_data = voter_token_account_info.try_borrow_data()?;
//         let token_account = spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(&token_account_data)?;
//         let base_token_account = token_account.base;
//         let balance = base_token_account.amount;

//         if balance == 0 {
//             msg!("No Voting Power");
//             return Err(FundError::NoVotingPower.into());
//         }

//         if vote != 0 {
//             proposal_aggregator_data.proposals[matched_index as usize].votes_yes += balance;
//         } else {
//             proposal_aggregator_data.proposals[matched_index as usize].votes_no += balance;
//         }

//         proposal_aggregator_data.serialize(&mut &mut proposal_aggregator_info.data.borrow_mut()[..])?;
//     }

//     msg!("[FUND-ACTIVITY] {} {} {} Vote: {} on proposal ({}, {})", fund_account_info.key.to_string(), current_time, fund_name, voter_account_info.key.to_string(), proposal_index, vec_index);

//     Ok(())
// }


fn process_cancel_investment_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: String,
    proposal_index: u8,
    vec_index: u16
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let proposer_account_info = next_account_info(accounts_iter)?; // Proposer wallet ..........................
    let proposal_aggregator_info = next_account_info(accounts_iter)?; // proposal aggregator ...................
    let fund_account_info = next_account_info(accounts_iter)?; // fund account .................................
    let rent_reserve_info = next_account_info(accounts_iter)?; // peerfund's rent reserve ......................

    if !proposer_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"light-fund", fund_name.as_bytes()], program_id);
    let (proposal_aggregator_pda, _proposal_aggregator_bump) = Pubkey::find_program_address(&[b"proposal-aggregator", &[proposal_index], fund_account_info.key.as_ref()], program_id);
    let (rent_pda, _rent_bump) = Pubkey::find_program_address(&[b"rent"], program_id);

    if *fund_account_info.key != fund_pda || *proposal_aggregator_info.key != proposal_aggregator_pda || *rent_reserve_info.key != rent_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let fund_data = LightFundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    if proposal_index > fund_data.current_proposal_index {
        msg!("Yaha se aaya hai");
        return Err(FundError::InvalidProposalAccount.into());
    }

    msg!("Vec Index obtained = {}", vec_index);

    let mut proposal_aggregator_data = ProposalAggregatorAccount::try_from_slice(&proposal_aggregator_info.data.borrow())?;

    msg!("Number of proposals = {}", proposal_aggregator_data.proposals.len());
    msg!("Vec Index of that proposal = {}", proposal_aggregator_data.proposals[0].vec_index);
    let proposal = proposal_aggregator_data
        .proposals
        .iter()
        .find(|proposal| proposal.vec_index == vec_index)
        .ok_or(FundError::InvalidProposalAccount)?;

    if proposal.proposer != *proposer_account_info.key {
        return Err(FundError::InvalidProposerInfo.into());
    }

    // if proposal.deadline < current_time {
    //     return Err(FundError::DeadlineReached.into());
    // }

    // Remove proposal from aggregator
    let rent = Rent::get()?;
    let current_aggregator_size = proposal_aggregator_info.data_len();
    msg!("Current Aggregator Size = {}", current_aggregator_size);
    let new_aggregator_size = current_aggregator_size - (132 + proposal.voters_bitmap.len() * 5);
    let current_aggregator_rent = proposal_aggregator_info.lamports();
    let new_aggregator_rent = rent.minimum_balance(new_aggregator_size);
    let new_aggregator_rent_for_proposer = rent.minimum_balance(new_aggregator_size + proposal.voters_bitmap.len() * 5);
    msg!("New Aggregator Size = {}", new_aggregator_size);
    msg!("New Aggregator Rent = {}", new_aggregator_rent);
    msg!("New Aggregator Rent for Proposer = {}", new_aggregator_rent_for_proposer);

    proposal_aggregator_info.realloc(new_aggregator_size, false)?;

    if new_aggregator_rent < current_aggregator_rent {
        **proposal_aggregator_info.lamports.borrow_mut() -= current_aggregator_rent - new_aggregator_rent;
        **proposer_account_info.lamports.borrow_mut() += current_aggregator_rent - new_aggregator_rent_for_proposer;
        **rent_reserve_info.lamports.borrow_mut() += new_aggregator_rent_for_proposer - new_aggregator_rent;
    }

    proposal_aggregator_data.proposals.retain(|p| p.vec_index != vec_index);

    proposal_aggregator_data.serialize(&mut &mut proposal_aggregator_info.data.borrow_mut()[..])?;

    msg!("[FUND-ACTIVITY] {} {} Investment Proposal ({}, {}) deleted by proposer ({})", fund_account_info.key.to_string(), current_time, proposal_index, vec_index, proposer_account_info.key.to_string());

    Ok(())
}


// fn process_init_rent_account(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
// ) -> ProgramResult {
//     let accounts_iter = &mut accounts.iter();
//     let rent_account_info = next_account_info(accounts_iter)?;
//     let system_program_info = next_account_info(accounts_iter)?;
//     let god_father_info = next_account_info(accounts_iter)?;

//     if !god_father_info.is_signer {
//         return Err(FundError::MissingRequiredSignature.into());
//     }

//     let (rent_pda, rent_bump) = Pubkey::find_program_address(&[b"rent"], program_id);

//     if *rent_account_info.key != rent_pda {
//         return Err(FundError::InvalidAccountData.into());
//     }

//     let rent = Rent::get()?;
//     let data_len = 0 as usize;
//     let rent_exemption_amount = rent.minimum_balance(data_len);

//     invoke_signed(
//         &system_instruction::create_account(
//             god_father_info.key,
//             rent_account_info.key,
//             rent_exemption_amount,
//             data_len as u64,
//             program_id
//         ),
//         &[god_father_info.clone(), rent_account_info.clone(), system_program_info.clone()],
//         &[&[b"rent", &[rent_bump]]]
//     )?;

//     Ok(())
// }

fn process_set_executing_or_executed(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    proposal_index: u8,
    vec_index: u16,
    fund_name: String,
    set: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let peerfunds_wallet_info = next_account_info(accounts_iter)?;
    let fund_account_info = next_account_info(accounts_iter)?;
    let proposal_aggregator_info = next_account_info(accounts_iter)?;

    if !peerfunds_wallet_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"light-fund", fund_name.as_bytes()], program_id);
    let (aggregator_pda, _aggregator_bump) = Pubkey::find_program_address(&[b"proposal-aggregator", &[proposal_index]], program_id);

    if *fund_account_info.key != fund_pda || *proposal_aggregator_info.key != aggregator_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let mut proposal_aggregator_data = ProposalAggregatorAccount::try_from_slice(&proposal_aggregator_info.data.borrow())?;

    let (matched_index, proposal) = proposal_aggregator_data
        .proposals
        .iter()
        .enumerate()
        .find(|proposal| proposal.1.vec_index == vec_index)
        .ok_or(FundError::InvalidAccountData)?;

    if (proposal.executed == 0 && set != 1) || (proposal.executed == 1 && set != 2) || (proposal.executed == 2) {
        return Err(FundError::InvalidProposalAccount.into());
    }

    proposal_aggregator_data.proposals[matched_index].executed = set;

    proposal_aggregator_data.serialize(&mut &mut proposal_aggregator_info.data.borrow_mut()[..])?;

    Ok(())
}

fn process_execute_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: String,
    proposal_index: u8,
    vec_index: u16,
    swap_index: u8,
    no_of_swaps: u8,
    merkel_proof: Vec<[u8; 32]>,
    amount: u64,
    slippage: u16,
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let account_iter = &mut accounts.iter();
    let payer_info = next_account_info(account_iter)?; // payer ...............................................
    let fund_account_info = next_account_info(account_iter)?; // fund account .................................
    let vault_account_info = next_account_info(account_iter)?; // fund's vault account ........................
    let proposal_aggregator_info = next_account_info(account_iter)?; // proposal account ......................
    let token_program_2022_info = next_account_info(account_iter)?; // token program 2022 .....................
    let token_program_std_info = next_account_info(account_iter)?; // token program 2020 ......................
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

    if !payer_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    if payer_info.key.to_string() != String::from("BA19YT7ryTxJY14J2CaY7xzAZWyR9afwDRUXgB7fMXEh") {
        return Err(FundError::InvalidSigner.into());
    }

    let (proposal_pda, _proposal_bump) = Pubkey::find_program_address(&[b"proposal-aggregator", &[proposal_index], fund_account_info.key.as_ref()], program_id);
    if *proposal_aggregator_info.key != proposal_pda {
        msg!("Wrong proposal aggregator account");
        return Err(FundError::InvalidProposalAccount.into());
    }

    let proposal_aggregator_data= ProposalAggregatorAccount::try_from_slice(&proposal_aggregator_info.data.borrow())?;
    let is_executed = proposal_aggregator_data.proposals[vec_index as usize].executed;

    // if proposal is executed/not initiated then return
    if is_executed != 1 {
        msg!("Proposal execution isn't initiated yet or it is already executed");
        return Err(FundError::InvalidAccountData.into());
    }

    let deadline = proposal_aggregator_data.proposals[vec_index as usize].deadline;

    // if voting deadline hasn't reached yet, return
    if current_time <= deadline {
        msg!("The proposal is still under voting.");
        return Err(FundError::InvalidAccountData.into());
    }

    let mut swap_hasher = Sha256::new();

    swap_hasher.update(input_token_account.key.to_bytes());
    swap_hasher.update(output_token_account.key.to_bytes());
    swap_hasher.update(&amount.to_le_bytes());
    swap_hasher.update(&slippage.to_le_bytes());

    let swap_hash: [u8; 32] = swap_hasher.finalize().into();
    let mut index = swap_index as usize;

    // --- Merkle Root Verification ---

    let mut swap_hasher1 = Sha256::new();

    swap_hasher1.update(input_token_account.key.to_bytes());
    swap_hasher1.update(output_token_account.key.to_bytes());
    swap_hasher1.update(&amount.to_le_bytes());
    swap_hasher1.update(&slippage.to_le_bytes());

    let mut calculated_merkel_root = swap_hasher1.finalize().into();
    let mut merkle_hasher = Sha256::new();

    for node in merkel_proof.iter() {
        if index % 2 == 0 {
            // Current hash is left child
            merkle_hasher.update(&swap_hash);
            merkle_hasher.update(&node[..]);
        } else {
            // Current hash is right child
            merkle_hasher.update(&node[..]);
            merkle_hasher.update(&swap_hash);
        }
        let intermediate: [u8; 32] = merkle_hasher.finalize().into();
        calculated_merkel_root = intermediate;
        merkle_hasher = Sha256::new();
        index /= 2;
    }

    let merkel_root = proposal_aggregator_data.proposals[vec_index as usize].merkel_root;

    if calculated_merkel_root != merkel_root {
        msg!("The merkel root calculated don't match with the one in the proposal.");
        return Err(FundError::InvalidSwapDetails.into());
    }

    let vote_yes = proposal_aggregator_data.proposals[vec_index as usize].votes_yes;
    let vote_no = proposal_aggregator_data.proposals[vec_index as usize].votes_no;

    let fund_data = LightFundAccount::try_from_slice(&fund_account_info.data.borrow())?;
    let strength = fund_data.members.len();

    // if quorum not reached, error
    if (vote_yes + vote_no) < (strength as u64) * 1 / 2 {
        msg!("Quorum not reached");
        return Err(FundError::InvalidInstruction.into());
    }

    // if proposal not in majority, error
    if vote_yes <= vote_no {
        msg!("Not enough votes favouring the trades");
        return Err(FundError::InvalidInstruction.into());
    }

    // check fund account
    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    if *fund_account_info.key != fund_pda {
        msg!("Wrong Fund details");
        return Err(FundError::InvalidFundDetails.into());
    }

    // verify vault account
    let (vault_pda, vault_bump) = Pubkey::find_program_address(&[b"vault", fund_pda.as_ref()], program_id);
    if *vault_account_info.key != vault_pda {
        msg!("Wrong vault account");
        return Err(FundError::InvaildVaultAccount.into());
    }

    // verify the vault's token accounts
    let input_vault_token_account = spl_associated_token_account::get_associated_token_address(
        &vault_pda,
        input_token_mint.key
    );
    if *input_token_account.key != input_vault_token_account || input_token_account.data_is_empty() {
        return Err(FundError::InvalidTokenAccount.into());
    }

    let output_vault_token_account = spl_associated_token_account::get_associated_token_address(
        &vault_pda,
        output_token_mint.key
    );
    if *output_token_account.key != output_vault_token_account {
        return Err(FundError::InvalidTokenAccount.into());
    }

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

    // proposal_aggregator_data.proposals[vec_index as usize].executed = true;
    proposal_aggregator_data.serialize(&mut &mut proposal_aggregator_info.data.borrow_mut()[..])?;

    msg!("[FUND-ACTIVITY] {} {} {} Proposal executed: ({}, {})", fund_account_info.key.to_string(), current_time, fund_name, proposal_index, vec_index);

    Ok(())
}

fn process_init_increment_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: String,
    new_size: u32,
    refund_type: u8,
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let proposer_account_info = next_account_info(accounts_iter)?; // proposer wallet ...........................
    let increment_proposal_account_info = next_account_info(accounts_iter)?; // proposal account ................
    let system_program_info = next_account_info(accounts_iter)?; // system program ..............................
    let fund_account_info = next_account_info(accounts_iter)?; // fund account ..................................
    let governance_mint_info = next_account_info(accounts_iter)?; // governance mint ............................
    let proposer_token_account_info = next_account_info(accounts_iter)?; // token account .......................
    let token_program_2022_info = next_account_info(accounts_iter)?; // token program 2022 ......................
    let rent_reserve_info = next_account_info(accounts_iter)?; // rent reserve ..................................

    if !proposer_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let (increment_proposal_pda, increment_proposal_bump) = Pubkey::find_program_address(&[b"increment-proposal-account", fund_account_info.key.as_ref()], program_id);
    let (rent_pda, _rent_bump) = Pubkey::find_program_address(&[b"rent"], program_id);

    if *fund_account_info.key != fund_pda || *increment_proposal_account_info.key != increment_proposal_pda || *rent_reserve_info.key != rent_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let token_account = get_associated_token_address_with_program_id(
        proposer_account_info.key,
        governance_mint_info.key,
        token_program_2022_info.key
    );

    if token_account != *proposer_token_account_info.key {
        return Err(FundError::InvalidTokenAccount.into());
    }

    let token_account_data = proposer_token_account_info.try_borrow_data()?;
    let token_account = spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(&token_account_data)?;
    let base_token_account = token_account.base;
    let balance = base_token_account.amount;
    drop(token_account_data);

    if balance == 0 {
        return Err(FundError::NoVotingPower.into());
    }

    if !increment_proposal_account_info.data_is_empty() {
        return Err(FundError::IncrementProposalExists.into());
    }

    let total_deposit: u64 = {
        let fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
        if new_size <= fund_data.expected_members {
            return Err(FundError::InvalidNewSize.into());
        }

        if fund_data.expected_members as usize > fund_data.members.len() && !fund_data.is_refunded {
            return Err(FundError::InvalidInstruction.into());
        }

        fund_data.total_deposit
    };


    if 2*balance >= total_deposit {

        if refund_type == 0 {
            invoke(
                &system_instruction::transfer(
                    proposer_account_info.key,
                    rent_reserve_info.key,
                    1_510_000
                ),
                &[proposer_account_info.clone(), rent_reserve_info.clone(), system_program_info.clone()]
            )?;
        }
        let mut fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;
        fund_data.expected_members = new_size;
        fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;

        msg!("[FUND-ACTIVITY] {} {} Fund's max size increased to {}", fund_account_info.key.to_string(), current_time, new_size);
    } else {
        let rent = Rent::get()?;
        let proposal_space = 90 as usize;
        let proposal_rent = rent.minimum_balance(proposal_space);

        invoke_signed(
            &system_instruction::create_account(
                proposer_account_info.key,
                increment_proposal_account_info.key,
                proposal_rent,
                proposal_space as u64,
                program_id
            ),
            &[proposer_account_info.clone(), increment_proposal_account_info.clone(), system_program_info.clone()],
            &[&[b"increment-proposal-account", fund_account_info.key.as_ref(), &[increment_proposal_bump]]]
        )?;

        let voters: Vec<(Pubkey, u8)> = vec![(*proposer_account_info.key, 1 as u8)];
        let proposal_data = IncrementProposalAccount {
            proposer: *proposer_account_info.key,
            new_size,
            refund_type,
            votes_yes: balance,
            votes_no: 0 as u64,
            voters
        };

        proposal_data.serialize(&mut &mut increment_proposal_account_info.data.borrow_mut()[..])?;

        msg!("[FUND-ACTIVITY] {} {} Increment Proposal created with new size: {}", fund_account_info.key.to_string(), current_time, new_size);
    }

    if refund_type == 0 {
        invoke_signed(
            &spl_token_2022::instruction::mint_to(
                token_program_2022_info.key,
                governance_mint_info.key,
                proposer_token_account_info.key,
                fund_account_info.key,
                &[],
                1_510_000,
            )?,
            &[
                governance_mint_info.clone(),
                proposer_token_account_info.clone(),
                fund_account_info.clone(),
                token_program_2022_info.clone(),
            ],
            &[&[b"fund", fund_name.as_bytes(), &[fund_bump]]],
        )?;
    }

    Ok(())
}

fn process_toggle_refund_type(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: String,
    refund_type: u8
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let proposer_account_info = next_account_info(accounts_iter)?; // proposer wallet ........................
    let fund_account_info = next_account_info(accounts_iter)?; // fund account ...............................
    let increment_proposal_account_info = next_account_info(accounts_iter)?; // proposal account .............
    let token_program_2022_info = next_account_info(accounts_iter)?;
    let governance_mint_info = next_account_info(accounts_iter)?;
    let proposer_token_account_info = next_account_info(accounts_iter)?;

    if !proposer_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let (increment_proposal_pda, _increment_proposal_bump) = Pubkey::find_program_address(&[b"increment-proposal-account", fund_account_info.key.as_ref()], program_id);

    if *fund_account_info.key != fund_pda || *increment_proposal_account_info.key != increment_proposal_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let mut proposal_data = IncrementProposalAccount::try_from_slice(&increment_proposal_account_info.data.borrow())?;
    if proposal_data.proposer != *proposer_account_info.key {
        return Err(FundError::InvalidProposerInfo.into());
    }

    if proposal_data.refund_type == 0 && refund_type == 1 {
        invoke(
            &spl_token_2022::instruction::burn(
                &spl_token_2022::id(),
                proposer_token_account_info.key,
                governance_mint_info.key,
                proposer_account_info.key,
                &[], 1_510_000
            )?,
            &[proposer_token_account_info.clone(), governance_mint_info.clone(), proposer_account_info.clone(), token_program_2022_info.clone()],
            // &[&[b"fund", fund_name.as_bytes(), &[fund_bump]]]
        )?;
    } else if proposal_data.refund_type == 1 && refund_type == 0 {
        invoke_signed(
            &spl_token_2022::instruction::mint_to(
                &spl_token_2022::id(),
                governance_mint_info.key,
                proposer_token_account_info.key,
                fund_account_info.key,
                &[],
                1_510_000
            )?,
            &[governance_mint_info.clone(), proposer_token_account_info.clone(), fund_account_info.clone(), token_program_2022_info.clone()],
            &[&[b"fund", fund_name.as_bytes(), &[fund_bump]]]
        )?;
    }

    proposal_data.refund_type = refund_type;
    proposal_data.serialize(&mut &mut increment_proposal_account_info.data.borrow_mut()[..])?;

    Ok(())
}

fn process_vote_increment_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: String,
    vote: u8
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let voter_account_info = next_account_info(accounts_iter)?; // voter wallet ...................................
    let increment_proposal_account_info = next_account_info(accounts_iter)?; // proposal account ..................
    let system_program_info = next_account_info(accounts_iter)?; // system program ................................
    let fund_account_info = next_account_info(accounts_iter)?; // fund account ....................................
    let governance_mint_info = next_account_info(accounts_iter)?; // governance mint ..............................
    let voter_token_account_info = next_account_info(accounts_iter)?; // voter token account ......................
    let token_program_2022_info = next_account_info(accounts_iter)?; // token program 2022 ........................
    let proposer_account_info = next_account_info(accounts_iter)?; // proposer wallet .............................
    let rent_reserve_info = next_account_info(accounts_iter)?; // rent reserve ....................................

    if !voter_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (fund_pda, fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let (increment_proposal_pda, _increment_proposal_bump) = Pubkey::find_program_address(&[b"increment-proposal-account", fund_account_info.key.as_ref()], program_id);
    let (rent_pda, _rent_bump) = Pubkey::find_program_address(&[b"rent"], program_id);

    if *fund_account_info.key != fund_pda || *increment_proposal_account_info.key != increment_proposal_pda || *rent_reserve_info.key != rent_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let token_account = get_associated_token_address_with_program_id(
        voter_account_info.key,
        governance_mint_info.key,
        token_program_2022_info.key
    );

    if token_account != *voter_token_account_info.key {
        return Err(FundError::InvalidTokenAccount.into());
    }

    if voter_token_account_info.data_is_empty() {
        return Err(FundError::NoVotingPower.into());
    }

    let token_account_data = voter_token_account_info.try_borrow_data()?;
    let token_account = spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(&token_account_data)?;
    let base_token_account = token_account.base;
    let balance = base_token_account.amount;

    if balance == 0 {
        return Err(FundError::NoVotingPower.into());
    }

    let mut proposal_data = IncrementProposalAccount::try_from_slice(&increment_proposal_account_info.data.borrow())?;
    let voter_exists = proposal_data
        .voters
        .iter()
        .any(|voter| voter.0 == *voter_account_info.key);

    if voter_exists {
        return Err(FundError::AlreadyVoted.into());
    }

    if proposal_data.proposer != *proposer_account_info.key {
        return Err(FundError::InvalidProposerInfo.into());
    }

    if vote == 1 {
        proposal_data.votes_yes += balance;
    } else {
        proposal_data.votes_no += balance;
    }

    let mut fund_data = FundAccount::try_from_slice(&fund_account_info.data.borrow())?;

    if 2 * proposal_data.votes_yes >= fund_data.total_deposit {
        fund_data.expected_members = proposal_data.new_size;
        fund_data.serialize(&mut &mut fund_account_info.data.borrow_mut()[..])?;
        let lamports = increment_proposal_account_info.lamports();
        **increment_proposal_account_info.lamports.borrow_mut() = 0;
        if proposal_data.refund_type == 1 {
            **proposer_account_info.lamports.borrow_mut() += 1_510_000;
            if lamports - 1_510_000 > 0 {
                **rent_reserve_info.lamports.borrow_mut() += lamports - 1_510_000;
            }
        } else {
            let proposer_token_account_info = next_account_info(accounts_iter)?;
            let token_account = get_associated_token_address_with_program_id(
                proposer_account_info.key,
                governance_mint_info.key,
                token_program_2022_info.key
            );
            if token_account != *proposer_token_account_info.key {
                return Err(FundError::InvalidTokenAccount.into());
            }
            **rent_reserve_info.lamports.borrow_mut() += lamports;
            invoke_signed(
                &spl_token_2022::instruction::mint_to(
                    token_program_2022_info.key,
                    governance_mint_info.key,
                    proposer_token_account_info.key,
                    fund_account_info.key,
                    &[],
                    1_510_000,
                )?,
                &[
                    governance_mint_info.clone(),
                    proposer_token_account_info.clone(),
                    fund_account_info.clone(),
                    token_program_2022_info.clone(),
                ],
                &[&[b"fund", fund_name.as_bytes(), &[fund_bump]]],
            )?;
        }

        msg!("[FUND-ACTIVITY] {} {} Fund's max size increased to {}", fund_account_info.key.to_string(), current_time, proposal_data.new_size);
    } else {
        proposal_data.voters.push((*voter_account_info.key, vote));
        let current_proposal_size = increment_proposal_account_info.data_len();
        let new_proposal_size = current_proposal_size + 33;
        let current_proposal_rent = increment_proposal_account_info.lamports();
        let new_proposal_rent = Rent::get()?.minimum_balance(new_proposal_size);

        if new_proposal_rent > current_proposal_rent {
            invoke(
                &system_instruction::transfer(
                    voter_account_info.key,
                    increment_proposal_account_info.key,
                    new_proposal_rent - current_proposal_rent
                ),
                &[voter_account_info.clone(), increment_proposal_account_info.clone(), system_program_info.clone()]
            )?;
        }

        increment_proposal_account_info.realloc(new_proposal_size, false)?;

        proposal_data.serialize(&mut &mut increment_proposal_account_info.data.borrow_mut()[..])?;
    }

    msg!("[FUND-ACTIVITY] {} {} {} voted on increment proposal", fund_account_info.key.to_string(), current_time, voter_account_info.key.to_string());

    Ok(())
}

fn process_cancel_increment_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: String
) -> ProgramResult {
    let current_time = Clock::get()?.unix_timestamp;

    let accounts_iter = &mut accounts.iter();
    let proposer_account_info = next_account_info(accounts_iter)?; // proposer wallet ............................
    let increment_proposal_account_info = next_account_info(accounts_iter)?; // proposal account .................
    let fund_account_info = next_account_info(accounts_iter)?; // fund account ...................................
    let rent_reserve_info = next_account_info(accounts_iter)?; // rent reserve ...................................

    if !proposer_account_info.is_signer {
        return Err(FundError::MissingRequiredSignature.into());
    }

    let (fund_pda, _fund_bump) = Pubkey::find_program_address(&[b"fund", fund_name.as_bytes()], program_id);
    let (increment_proposal_pda, _increment_proposal_bump) = Pubkey::find_program_address(&[b"increment-proposal-account", fund_account_info.key.as_ref()], program_id);
    let (rent_pda, _rent_bump) = Pubkey::find_program_address(&[b"rent"], program_id);

    if *fund_account_info.key != fund_pda || *increment_proposal_account_info.key != increment_proposal_pda || *rent_reserve_info.key != rent_pda {
        return Err(FundError::InvalidAccountData.into());
    }

    let proposal_data = IncrementProposalAccount::try_from_slice(&increment_proposal_account_info.data.borrow())?;
    if proposal_data.proposer != *proposer_account_info.key {
        return Err(FundError::InvalidProposerInfo.into());
    }

    let lamports = increment_proposal_account_info.lamports();
    **increment_proposal_account_info.lamports.borrow_mut() = 0;
    if proposal_data.refund_type == 1 {
        **proposer_account_info.lamports.borrow_mut() += 1_510_000;
        if lamports - 1_510_000 > 0 {
            **rent_reserve_info.lamports.borrow_mut() += lamports - 1_510_000;
        }
    } else {
        **rent_reserve_info.lamports.borrow_mut() += lamports;
    }

    msg!("[FUND-ACTIVITY] {} {} Increment proposal for new size {} deleted", fund_account_info.key.to_string(), current_time, proposal_data.new_size);

    Ok(())
}