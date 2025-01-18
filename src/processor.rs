use crate::constants::Constants;
use crate::instruction::{AgentInstruction, CounterInstruction};
use crate::state::{
    AgentInfo, AgentSettings, ContractInfo, CounterAccount, MessagePayload,
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
        let accounts_iter = &mut accounts.iter();

        match instruction {
            AgentInstruction::Initialize => {
                let payer_account = next_account_info(accounts_iter)?;
                let contract_info_account = next_account_info(accounts_iter)?;
                Self::process_initialize(program_id, payer_account, contract_info_account)
            }
            AgentInstruction::CreateAgent => {
                let payer_account = next_account_info(accounts_iter)?;
                let contract_info_account = next_account_info(accounts_iter)?;
                let agent_account = next_account_info(accounts_iter)?;
                Self::process_create_agent(
                    program_id,
                    payer_account,
                    contract_info_account,
                    agent_account,
                )
            }
            AgentInstruction::RegisterAgent { agent_settings } => {
                let contract_info_account = next_account_info(accounts_iter)?;
                let agent_account = next_account_info(accounts_iter)?;
                Self::process_register_agent(
                    program_id,
                    contract_info_account,
                    agent_account,
                    agent_settings,
                )
            }
            AgentInstruction::CreateAndRegisterAgent { agent_settings } => {
                Self::process_create_and_register_agent(program_id, accounts, agent_settings)
            }
            AgentInstruction::ChangeAgentSettingProposal {
                agent_id,
                agent_settings,
            } => Self::process_change_agent_setting_proposal(
                program_id,
                accounts,
                agent_id,
                agent_settings,
            ),
            AgentInstruction::Verify {
                settings_digest,
                payload,
            } => Self::process_verify(program_id, accounts, settings_digest, payload),
            AgentInstruction::AcceptAgent { agent_id } => {
                Self::process_accept_agent(program_id, accounts, agent_id)
            }
            AgentInstruction::AcceptAgentSettingProposal { agent_id } => {
                Self::process_accept_agent_setting_proposal(program_id, accounts, agent_id)
            }
            AgentInstruction::RemoveAgent { agent_id } => {
                Self::process_remove_agent(program_id, accounts, agent_id)
            }
        }
    }

    fn process_initialize<'a>(
        program_id: &Pubkey,
        payer_account: &AccountInfo<'a>,
        contract_info_account: &AccountInfo<'a>,
    ) -> ProgramResult {
        // Create and initialize the counter account
        DataAccountUtils::create_related_account(
            program_id,
            payer_account,
            contract_info_account,
            Constants::PREFIX_CONTRACT_INFO,
            b"",
            Constants::SIZE_CONTRACT_INFO,
        )?;

        // Initialize counter with starting ID of 0
        DataAccountUtils::write_account_data(
            contract_info_account,
            ContractInfo {
                agent_counter: 0,
                type_and_version: "AI Agent 1.0.0".to_string(),
                agent_version: "AI Agent 1.0.0".to_string(),
            },
        )?;

        msg!("Contract info data account initialized");
        Ok(())
    }

    fn process_create_agent<'a>(
        program_id: &Pubkey,
        payer_account: &AccountInfo<'a>,
        contract_info_account: &AccountInfo<'a>,
        agent_account: &AccountInfo<'a>,
    ) -> ProgramResult {
        DataAccountUtils::check_account_match(
            program_id,
            contract_info_account,
            Constants::PREFIX_CONTRACT_INFO,
            b"",
        )?;

        let mut info: ContractInfo = DataAccountUtils::read_account_data(contract_info_account)?;

        DataAccountUtils::create_related_account(
            program_id,
            payer_account,
            agent_account,
            Constants::PREFIX_AGENT_ADDRESS,
            &info.agent_counter.to_le_bytes(),
            Constants::SIZE_AGENT_INFO,
        )?;

        let mut agent_info = AgentInfo::default();
        agent_info.agent_id = info.agent_counter;
        DataAccountUtils::write_account_data(agent_account, agent_info)?;

        info.agent_counter += 1;
        DataAccountUtils::write_account_data(contract_info_account, info)?;

        Ok(())
    }

    fn process_register_agent<'a>(
        program_id: &Pubkey,
        contract_info_account: &AccountInfo<'a>,
        agent_account: &AccountInfo<'a>,
        agent_settings: AgentSettings,
    ) -> ProgramResult {
        DataAccountUtils::check_account_match(
            program_id,
            contract_info_account,
            Constants::PREFIX_CONTRACT_INFO,
            b"",
        )?;
        let mut agent_info: AgentInfo = DataAccountUtils::read_account_data(agent_account)?;
        agent_info.agent_settings = agent_settings;

        msg!("Writing agent info:");
        msg!(" - agent_id: {}", agent_info.agent_id);
        msg!(" - is_allowed: {}", agent_info.is_allowed);
        msg!(" - is_removed: {}", agent_info.is_removed);
        msg!(" - is_new_settings: {}", agent_info.is_new_settings);
        msg!(" - agent_settings: {:?}", agent_info.agent_settings);
        msg!(" - agent_config: {:?}", agent_info.agent_config);

        DataAccountUtils::write_account_data(agent_account, agent_info)?;
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

        let contract_info_account = next_account_info(accounts_iter)?;
        let payer_account = next_account_info(accounts_iter)?;

        // Create a new data account for the counter
        DataAccountUtils::create_related_account(
            program_id,
            payer_account,
            contract_info_account,
            b"counter",
            b"-1",
            8, // data size
        )?;

        // Write the initial value to the counter account
        DataAccountUtils::write_account_data(
            contract_info_account,
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
        let contract_info_account = next_account_info(accounts_iter)?;

        // Verify account ownership
        if contract_info_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Read the counter data
        let mut counter_data: CounterAccount =
            DataAccountUtils::read_account_data(contract_info_account)?;

        // Increment the counter value and write back
        counter_data.count = counter_data
            .count
            .checked_add(1)
            .ok_or(ProgramError::InvalidAccountData)?;
        msg!("Counter incremented to: {}", counter_data.count);
        DataAccountUtils::write_account_data(contract_info_account, counter_data)?;

        Ok(())
    }

    fn process_add_any_value(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let contract_info_account = next_account_info(accounts_iter)?;

        // Verify account ownership
        if contract_info_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Mutable borrow the account data
        let mut data = contract_info_account.data.borrow_mut();

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
