use crate::instruction::{AgentInstruction, CounterInstruction};
use crate::state::{AgentSettings, CounterAccount, MessagePayload};
use crate::utils::{create_related_account, read_account_data, write_account_data};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
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

    // Create a new data account for the counter
    create_related_account(
        program_id,
        payer_account,
        counter_account,
        system_program,
        b"counter",
        b"-1",
        8, // data size
    )?;

    // Write the initial value to the counter account
    write_account_data(
        counter_account,
        CounterAccount {
            count: initial_value,
        },
    )?;

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

    // Read the counter data
    let mut counter_data: CounterAccount = read_account_data(counter_account)?;

    // Increment the counter value and write back
    counter_data.count = counter_data
        .count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;
    msg!("Counter incremented to: {}", counter_data.count);
    write_account_data(counter_account, counter_data)?;

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
