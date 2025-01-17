use crate::instruction::{AgentInstruction, CounterInstruction};
use crate::state::{AgentSettings, CounterAccount, MessagePayload};
use crate::utils::{create_related_account, write_related_account};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = AgentInstruction::unpack(instruction_data)?;
    match instruction {
        AgentInstruction::Initialize => process_initialize(program_id, accounts)?,
        AgentInstruction::CreateAgent => process_create_agent(program_id, accounts)?,
        AgentInstruction::RegisterAgent { agent_settings } => {
            process_register_agent(program_id, accounts, agent_settings)?
        }
        AgentInstruction::CreateAndRegisterAgent { agent_settings } => {
            process_create_and_register_agent(program_id, accounts, agent_settings)?
        }
        AgentInstruction::ChangeAgentSettingProposal {
            agent_id,
            agent_settings,
        } => process_change_agent_setting_proposal(program_id, accounts, agent_id, agent_settings)?,
        AgentInstruction::Verify {
            settings_digest,
            payload,
        } => process_verify(program_id, accounts, settings_digest, payload)?,
        AgentInstruction::AcceptAgent { agent_id } => {
            process_accept_agent(program_id, accounts, agent_id)?
        }
        AgentInstruction::AcceptAgentSettingProposal { agent_id } => {
            process_accept_agent_setting_proposal(program_id, accounts, agent_id)?
        }
        AgentInstruction::RemoveAgent { agent_id } => {
            process_remove_agent(program_id, accounts, agent_id)?
        }
    };
    Ok(())
}

/// This function is only for reference. Will be removed in the future.
#[deprecated]
pub fn process_instruction_counter(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Unpack instruction data
    let instruction = CounterInstruction::unpack(instruction_data)?;

    // Match instruction type
    match instruction {
        CounterInstruction::InitializeCounter { initial_value } => {
            process_initialize_counter(program_id, accounts, initial_value)?
        }
        CounterInstruction::IncrementCounter => process_increment_counter(program_id, accounts)?,
        CounterInstruction::AddAnyValue { amount } => {
            process_add_any_value(program_id, accounts, amount)?
        }
    };
    Ok(())
}

// Initialize a new counter account
fn process_initialize_counter(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    initial_value: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let counter_account = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // // Size of our counter account
    // let account_space = 8; // Size in bytes to store a u64

    // // Calculate minimum balance for rent exemption
    // let rent = Rent::get()?;
    // let required_lamports = rent.minimum_balance(account_space);

    // // Create the counter account
    // invoke(
    //     &system_instruction::create_account(
    //         payer_account.key,    // Account paying for the new account
    //         counter_account.key,  // Account to be created
    //         required_lamports,    // Amount of lamports to transfer to the new account
    //         account_space as u64, // Size in bytes to allocate for the data field
    //         program_id,           // Set program owner to our program
    //     ),
    //     &[
    //         payer_account.clone(),
    //         counter_account.clone(),
    //         system_program.clone(),
    //     ],
    // )?;

    create_related_account(
        program_id,
        payer_account,
        counter_account,
        system_program,
        b"counter",
        b"-1",
        8,
    )?;

    // // Create a new CounterAccount struct with the initial value
    // let counter_data = CounterAccount {
    //     count: initial_value,
    // };

    // // Get a mutable reference to the counter account's data
    // let mut account_data = &mut counter_account.data.borrow_mut()[..];

    // // Serialize the CounterAccount struct into the account's data
    // counter_data.serialize(&mut account_data)?;

    write_related_account(counter_account, &initial_value.to_le_bytes())?;

    msg!("Counter initialized with value: {}", initial_value);

    Ok(())
}

// Update an existing counter's value
fn process_increment_counter(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    // Verify account ownership
    if counter_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Mutable borrow the account data
    let mut data = counter_account.data.borrow_mut();

    // Deserialize the account data into our CounterAccount struct
    let mut counter_data: CounterAccount = CounterAccount::try_from_slice(&data)?;

    // Increment the counter value
    counter_data.count = counter_data
        .count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    // Serialize the updated counter data back into the account
    counter_data.serialize(&mut &mut data[..])?;

    msg!("Counter incremented to: {}", counter_data.count);
    Ok(())
}

fn process_add_any_value(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    // Verify account ownership
    if counter_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Mutable borrow the account data
    let mut data = counter_account.data.borrow_mut();

    // Deserialize the account data into our CounterAccount struct
    let mut counter_data: CounterAccount = CounterAccount::try_from_slice(&data)?;

    // Add the specified amount to the counter value
    counter_data.count = counter_data
        .count
        .checked_add(amount)
        .ok_or(ProgramError::InvalidAccountData)?;

    // Serialize the updated counter data back into the account
    counter_data.serialize(&mut &mut data[..])?;

    msg!("Counter increased by {} to {}", amount, counter_data.count);
    Ok(())
}

// Agent instruction processing functions

fn process_initialize(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    // TODO: Initialize contract state
    Ok(())
}

fn process_create_agent(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    // TODO: Create new agent
    Ok(())
}

fn process_register_agent(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _agent_settings: AgentSettings,
) -> ProgramResult {
    // TODO: Register agent with provided settings
    Ok(())
}

fn process_create_and_register_agent(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _agent_settings: AgentSettings,
) -> ProgramResult {
    // TODO: Create and register agent in one transaction
    Ok(())
}

fn process_change_agent_setting_proposal(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _agent_id: u128,
    _agent_settings: AgentSettings,
) -> ProgramResult {
    // TODO: Process agent setting change proposal
    Ok(())
}

fn process_verify(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _settings_digest: [u8; 32],
    _payload: MessagePayload,
) -> ProgramResult {
    // TODO: Verify message payload
    Ok(())
}

fn process_accept_agent(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _agent_id: u128,
) -> ProgramResult {
    // TODO: Accept agent registration
    Ok(())
}

fn process_accept_agent_setting_proposal(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _agent_id: u128,
) -> ProgramResult {
    // TODO: Accept agent setting change proposal
    Ok(())
}

fn process_remove_agent(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _agent_id: u128,
) -> ProgramResult {
    // TODO: Remove agent
    Ok(())
}
