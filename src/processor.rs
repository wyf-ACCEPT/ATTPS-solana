use crate::constants::Constants;
use crate::error::{AgentHeaderError, AttpsAccountError};
use crate::instruction::AgentInstruction;
use crate::state::{AgentConfig, AgentInfo, AgentSettings, ContractInfo, MessagePayload};
use crate::utils::{AgentManagerUtils, DataAccountUtils};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    sysvar::Sysvar,
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
                let owner_account = next_account_info(accounts_iter)?;
                let contract_info_account = next_account_info(accounts_iter)?;
                Self::process_initialize(
                    program_id,
                    payer_account,
                    owner_account,
                    contract_info_account,
                )
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
                let payer_account = next_account_info(accounts_iter)?;
                let contract_info_account = next_account_info(accounts_iter)?;
                let agent_account = next_account_info(accounts_iter)?;
                Self::process_create_and_register_agent(
                    program_id,
                    payer_account,
                    contract_info_account,
                    agent_account,
                    agent_settings,
                )
            }
            AgentInstruction::AcceptAgent { agent_id } => {
                let owner_account = next_account_info(accounts_iter)?;
                let contract_info_account = next_account_info(accounts_iter)?;
                let agent_account = next_account_info(accounts_iter)?;
                Self::process_accept_agent(
                    program_id,
                    owner_account,
                    contract_info_account,
                    agent_account,
                    agent_id,
                )
            }
            AgentInstruction::ChangeAgentSettingProposal {
                agent_id,
                proposed_settings,
            } => {
                let agent_account = next_account_info(accounts_iter)?;
                Self::process_change_agent_setting_proposal(
                    program_id,
                    agent_account,
                    agent_id,
                    proposed_settings,
                )
            }
            AgentInstruction::AcceptAgentSettingProposal { agent_id } => {
                let owner_account = next_account_info(accounts_iter)?;
                let contract_info_account = next_account_info(accounts_iter)?;
                let agent_account = next_account_info(accounts_iter)?;
                Self::process_accept_agent_setting_proposal(
                    program_id,
                    owner_account,
                    contract_info_account,
                    agent_account,
                    agent_id,
                )
            }
            AgentInstruction::RemoveAgent { agent_id } => {
                Self::process_remove_agent(program_id, accounts, agent_id)
            }
            AgentInstruction::Verify {
                settings_digest,
                payload,
            } => Self::process_verify(program_id, accounts, settings_digest, payload),
        }
    }

    fn process_initialize<'a>(
        program_id: &Pubkey,
        payer_account: &AccountInfo<'a>,
        owner_account: &AccountInfo,
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
                owner: *owner_account.key,
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
        let mut contract_info: ContractInfo =
            DataAccountUtils::read_account_data(contract_info_account)?;

        let mut agent_info = AgentInfo::default();
        agent_info.agent_id = contract_info.agent_counter;

        DataAccountUtils::create_related_account(
            program_id,
            payer_account,
            agent_account,
            Constants::PREFIX_AGENT_ADDRESS,
            &agent_info.agent_id.to_le_bytes(),
            Constants::SIZE_AGENT_INFO,
        )?;
        DataAccountUtils::write_account_data(agent_account, agent_info)?;

        contract_info.agent_counter += 1;
        DataAccountUtils::write_account_data(contract_info_account, contract_info)?;

        Ok(())
    }

    fn process_register_agent(
        program_id: &Pubkey,
        contract_info_account: &AccountInfo,
        agent_account: &AccountInfo,
        initial_settings: AgentSettings,
    ) -> ProgramResult {
        AgentManagerUtils::validate_agent_header(&initial_settings.agent_header)?;
        DataAccountUtils::check_account_match(
            program_id,
            contract_info_account,
            Constants::PREFIX_CONTRACT_INFO,
            b"",
        )?;
        let mut agent_info: AgentInfo = DataAccountUtils::read_account_data(agent_account)?;

        if agent_info.is_registered {
            Err(AttpsAccountError::AgentAlreadyRegistered.into())
        } else if agent_info.is_allowed {
            Err(AttpsAccountError::AgentAlreadyAllowed.into())
        } else if agent_info.is_removed {
            Err(AttpsAccountError::AgentAlreadyRemoved.into())
        } else {
            agent_info.is_registered = true;
            agent_info.agent_settings = initial_settings;
            agent_info.print_values();
            DataAccountUtils::write_account_data(agent_account, agent_info)
        }
    }

    fn process_create_and_register_agent<'a>(
        program_id: &Pubkey,
        payer_account: &AccountInfo<'a>,
        contract_info_account: &AccountInfo<'a>,
        agent_account: &AccountInfo<'a>,
        agent_settings: AgentSettings,
    ) -> ProgramResult {
        Self::process_create_agent(
            program_id,
            payer_account,
            contract_info_account,
            agent_account,
        )?;
        Self::process_register_agent(
            program_id,
            contract_info_account,
            agent_account,
            agent_settings,
        )
    }

    fn process_accept_agent(
        program_id: &Pubkey,
        owner_account: &AccountInfo,
        contract_info_account: &AccountInfo,
        agent_account: &AccountInfo,
        agent_id: u128,
    ) -> ProgramResult {
        DataAccountUtils::check_account_match(
            program_id,
            contract_info_account,
            Constants::PREFIX_CONTRACT_INFO,
            b"",
        )?;
        DataAccountUtils::check_account_match(
            program_id,
            agent_account,
            Constants::PREFIX_AGENT_ADDRESS,
            &agent_id.to_le_bytes(),
        )?;

        let contract_info: ContractInfo =
            DataAccountUtils::read_account_data(contract_info_account)?;
        contract_info.only_owner(owner_account)?;

        let mut agent_info: AgentInfo = DataAccountUtils::read_account_data(agent_account)?;
        if !agent_info.is_registered {
            Err(AttpsAccountError::AgentNotRegistered.into())
        } else if agent_info.is_allowed {
            Err(AttpsAccountError::AgentAlreadyAllowed.into())
        } else if agent_info.is_removed {
            Err(AttpsAccountError::AgentAlreadyRemoved.into())
        } else {
            agent_info.is_registered = false;
            agent_info.is_allowed = true;

            let settings = agent_info.agent_settings.clone();
            agent_info.agent_config = AgentConfig {
                config_digest: AgentManagerUtils::setting_digest_from_settings_data(
                    *agent_account.key,
                    &settings,
                ),
                config_block_number: Clock::get()?.slot,
                is_active: true,
                settings,
            };

            agent_info.print_values();
            DataAccountUtils::write_account_data(agent_account, agent_info)?;
            Ok(())
        }
    }

    fn process_change_agent_setting_proposal(
        program_id: &Pubkey,
        agent_account: &AccountInfo,
        agent_id: u128,
        proposed_settings: AgentSettings,
    ) -> ProgramResult {
        DataAccountUtils::check_account_match(
            program_id,
            agent_account,
            Constants::PREFIX_AGENT_ADDRESS,
            &agent_id.to_le_bytes(),
        )?;
        AgentManagerUtils::validate_agent_header(&proposed_settings.agent_header)?;

        let mut agent_info: AgentInfo = DataAccountUtils::read_account_data(agent_account)?;
        let settings = agent_info.agent_settings.clone();
        let proposed_settings_digest = AgentManagerUtils::setting_digest_from_settings_data(
            *agent_account.key,
            &proposed_settings,
        );
        let digest = agent_info.agent_config.config_digest;

        if (!agent_info.is_registered && !agent_info.is_allowed) || agent_info.is_removed {
            Err(AttpsAccountError::InvalidAllowedAgent.into())
        } else if settings.agent_header.source_agent_id
            != proposed_settings.agent_header.source_agent_id
        {
            Err(AgentHeaderError::InvalidAgentHeaderAgentId.into())
        } else if digest == proposed_settings_digest {
            Err(AttpsAccountError::DuplicateAgentSettings.into())
        } else {
            let _old_pending_settings = agent_info.pending_settings.replace(proposed_settings);
            agent_info.print_values();
            DataAccountUtils::write_account_data(agent_account, agent_info)
        }
    }

    fn process_accept_agent_setting_proposal(
        program_id: &Pubkey,
        owner_account: &AccountInfo,
        contract_info_account: &AccountInfo,
        agent_account: &AccountInfo,
        agent_id: u128,
    ) -> ProgramResult {
        DataAccountUtils::check_account_match(
            program_id,
            contract_info_account,
            Constants::PREFIX_CONTRACT_INFO,
            b"",
        )?;
        DataAccountUtils::check_account_match(
            program_id,
            agent_account,
            Constants::PREFIX_AGENT_ADDRESS,
            &agent_id.to_le_bytes(),
        )?;

        let contract_info: ContractInfo =
            DataAccountUtils::read_account_data(contract_info_account)?;
        contract_info.only_owner(owner_account)?;

        let mut agent_info: AgentInfo = DataAccountUtils::read_account_data(agent_account)?;

        if !agent_info.is_allowed || agent_info.is_removed {
            Err(AttpsAccountError::InvalidAllowedAgent.into())
        } else {
            let proposed_settings = agent_info.pending_settings.take().unwrap();
            agent_info.agent_settings = proposed_settings.clone();
            agent_info.agent_config = AgentConfig {
                config_digest: AgentManagerUtils::setting_digest_from_settings_data(
                    *agent_account.key,
                    &proposed_settings,
                ),
                config_block_number: Clock::get()?.slot,
                is_active: true,
                settings: proposed_settings,
            };
            agent_info.print_values();
            DataAccountUtils::write_account_data(agent_account, agent_info)
        }
    }

    fn process_remove_agent(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _agent_id: u128,
    ) -> ProgramResult {
        // TODO: Remove agent
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
}
