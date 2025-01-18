use crate::constants::Constants;
use crate::error::AttpsAccountError;
use crate::instruction::{AgentInstruction, CounterInstruction};
use crate::state::{
    AgentConfig, AgentCounter, AgentInfo, AgentSettings, CounterAccount, MessagePayload,
};
use crate::utils::DataAccountUtils;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub struct Processor;

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = AgentInstruction::unpack(instruction_data)?;
        match instruction {
            AgentInstruction::Initialize => Self::process_initialize(program_id, accounts)?,
            AgentInstruction::CreateAgent => Self::process_create_agent(program_id, accounts)?,
            AgentInstruction::RegisterAgent { agent_settings } => {
                Self::process_register_agent(program_id, accounts, agent_settings)?
            }
            AgentInstruction::CreateAndRegisterAgent { agent_settings } => {
                Self::process_create_and_register_agent(program_id, accounts, agent_settings)?
            }
            AgentInstruction::ChangeAgentSettingProposal {
                agent_id,
                agent_settings,
            } => Self::process_change_agent_setting_proposal(
                program_id,
                accounts,
                agent_id,
                agent_settings,
            )?,
            AgentInstruction::Verify {
                settings_digest,
                payload,
            } => Self::process_verify(program_id, accounts, settings_digest, payload)?,
            AgentInstruction::AcceptAgent { agent_id } => {
                Self::process_accept_agent(program_id, accounts, agent_id)?
            }
            AgentInstruction::AcceptAgentSettingProposal { agent_id } => {
                Self::process_accept_agent_setting_proposal(program_id, accounts, agent_id)?
            }
            AgentInstruction::RemoveAgent { agent_id } => {
                Self::process_remove_agent(program_id, accounts, agent_id)?
            }
        };
        Ok(())
    }

    fn process_initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let counter_account = next_account_info(accounts_iter)?;
        let payer_account = next_account_info(accounts_iter)?;

        // Create AgentCounter PDA if it doesn't exist
        let (counter_pda, _) = Pubkey::find_program_address(
            &[
                Constants::PREFIX_AGENT_COUNTER,
                Constants::AGENT_COUNTER_SEED,
            ],
            program_id,
        );

        if counter_pda != *counter_account.key {
            return Err(AttpsAccountError::PdaAccountMismatch.into());
        }

        if !counter_account.data_is_empty() {
            return Err(AttpsAccountError::PdaAccountAlreadyCreated.into());
        }

        // Create and initialize the counter account
        DataAccountUtils::create_related_account(
            program_id,
            payer_account,
            counter_account,
            Constants::PREFIX_AGENT_COUNTER,
            Constants::AGENT_COUNTER_SEED,
            Constants::AGENT_COUNTER_SIZE,
        )?;

        // Initialize counter with starting ID of 0
        DataAccountUtils::write_account_data(counter_account, AgentCounter { current_id: 0 })?;

        msg!("Agent counter initialized");
        Ok(())
    }

    fn process_create_agent(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
        // TODO: Create new agent
        Ok(())
    }

    fn process_register_agent(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        agent_settings: AgentSettings,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let agent_account = next_account_info(accounts_iter)?;
        let payer_account = next_account_info(accounts_iter)?;
        let counter_account = next_account_info(accounts_iter)?;

        // Get or create counter PDA
        let (counter_pda, _) = Pubkey::find_program_address(
            &[
                Constants::PREFIX_AGENT_COUNTER,
                Constants::AGENT_COUNTER_SEED,
            ],
            program_id,
        );
        if counter_pda != *counter_account.key {
            DataAccountUtils::create_related_account(
                program_id,
                payer_account,
                counter_account,
                Constants::PREFIX_AGENT_COUNTER,
                Constants::AGENT_COUNTER_SEED,
                Constants::AGENT_COUNTER_SIZE,
            )?;
            DataAccountUtils::write_account_data(counter_account, AgentCounter { current_id: 0 })?;
        }

        // Get current ID and increment
        let mut counter: AgentCounter = DataAccountUtils::read_account_data(counter_account)?;
        let agent_id = counter.current_id;
        counter.current_id += 1;
        DataAccountUtils::write_account_data(counter_account, counter)?;

        // Create agent PDA
        let id_bytes = agent_id.to_le_bytes();
        DataAccountUtils::create_related_account(
            program_id,
            payer_account,
            agent_account,
            Constants::PREFIX_AGENT_ADDRESS,
            &id_bytes,
            Constants::AGENT_INFO_SIZE,
        )?;

        // Initialize agent info with unique ID
        let agent_info = AgentInfo {
            agent_id, // Set from counter
            is_allowed: false,
            is_removed: false,
            is_new_settings: false,
            agent_settings: agent_settings.clone(),
            agent_config: AgentConfig {
                config_digest: [0; 32],
                config_block_number: 0,
                is_active: false,
                settings: agent_settings,
            },
        };

        // Debug log before writing
        msg!("Writing agent info:");
        msg!("- agent_id: {}", agent_info.agent_id);
        msg!("- is_allowed: {}", agent_info.is_allowed);
        msg!("- is_removed: {}", agent_info.is_removed);
        msg!("- is_new_settings: {}", agent_info.is_new_settings);
        msg!("- agent_settings: {:?}", agent_info.agent_settings);
        msg!("- agent_config: {:?}", agent_info.agent_config);

        // Serialize to check the data format
        let mut data = vec![];
        agent_info.serialize(&mut data).unwrap();
        msg!("Serialized data length: {}", data.len());
        msg!("First 32 bytes: {:?}", &data[..32.min(data.len())]);

        DataAccountUtils::write_account_data(agent_account, agent_info)?;

        msg!("Agent registered with ID: {}", agent_id);
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
}

pub struct CounterProcessor;

impl CounterProcessor {
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
                Self::process_initialize_counter(program_id, accounts, initial_value)?
            }
            CounterInstruction::IncrementCounter => {
                Self::process_increment_counter(program_id, accounts)?
            }
            CounterInstruction::AddAnyValue { amount } => {
                Self::process_add_any_value(program_id, accounts, amount)?
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

        // Create a new data account for the counter
        DataAccountUtils::create_related_account(
            program_id,
            payer_account,
            counter_account,
            b"counter",
            b"-1",
            8, // data size
        )?;

        // Write the initial value to the counter account
        DataAccountUtils::write_account_data(
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
        let mut counter_data: CounterAccount =
            DataAccountUtils::read_account_data(counter_account)?;

        // Increment the counter value and write back
        counter_data.count = counter_data
            .count
            .checked_add(1)
            .ok_or(ProgramError::InvalidAccountData)?;
        msg!("Counter incremented to: {}", counter_data.count);
        DataAccountUtils::write_account_data(counter_account, counter_data)?;

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
}
