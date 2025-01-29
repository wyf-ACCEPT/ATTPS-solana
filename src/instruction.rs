use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::state::{AgentSettings, MessagePayload};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum AgentInstruction {
    /// [0] Initialize the contract
    ///
    /// 0. [signer] payer
    /// 1. [] owner
    /// 2. [writable] contract_info
    Initialize,

    /// [1] Create a new agent (create a new data account)
    ///
    /// 0. [signer] payer
    /// 1. [writable] contract_info
    /// 2. [writable] agent data account
    CreateAgent,

    /// [2] Register an agent (write to agent data account)
    ///
    /// 0. [writable] contract_info
    /// 1. [writable] agent data account
    RegisterAgent { agent_settings: AgentSettings },

    /// [3] Create & register an agent ([1] + [2])
    ///
    /// 0. [signer] payer
    /// 1. [writable] contract_info
    /// 2. [writable] agent data account
    CreateAndRegisterAgent { agent_settings: AgentSettings },

    /// [4] Accept an agent
    ///
    /// 0. [signer] owner
    /// 1. [writable] contract_info
    /// 2. [writable] agent data account
    AcceptAgent,

    /// [5] Change an agent setting proposal
    ///
    /// 0. [writable] agent data account
    ChangeAgentSettingProposal { proposed_settings: AgentSettings },

    /// [6] Accept an agent setting proposal
    ///
    /// 0. [signer] owner
    /// 1. [writable] contract_info
    /// 2. [writable] agent data account
    AcceptAgentSettingProposal,

    /// [7] Remove an agent
    ///
    /// 0. [signer] owner
    /// 1. [writable] contract_info
    /// 2. [writable] agent data account
    RemoveAgent,

    /// [8] Verify a message of an agent
    ///
    /// 0. [] agent data account
    Verify {
        settings_digest: [u8; 32],
        payload: MessagePayload,
    },
}

impl AgentInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        // Get the instruction variant from the first byte
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        // Match instruction type and parse the remaining bytes based on the variant
        match variant {
            0 => Ok(Self::Initialize),
            1 => Ok(Self::CreateAgent),
            2 => {
                // For RegisterAgent, parse AgentSettings from remaining bytes
                let agent_settings = AgentSettings::try_from_slice(rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::RegisterAgent { agent_settings })
            }
            3 => {
                // For CreateAndRegisterAgent, parse AgentSettings from remaining bytes
                let agent_settings = AgentSettings::try_from_slice(rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::CreateAndRegisterAgent { agent_settings })
            }
            4 => Ok(Self::AcceptAgent),
            5 => {
                // For ChangeAgentSettingProposal, parse AgentSettings
                let proposed_settings = AgentSettings::try_from_slice(rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::ChangeAgentSettingProposal { proposed_settings })
            }
            6 => Ok(Self::AcceptAgentSettingProposal),
            7 => Ok(Self::RemoveAgent),
            8 => {
                // For Verify, parse settings_digest ([u8; 32]) and MessagePayload
                let (settings_digest_u8, rest) = rest.split_at(32);
                let settings_digest = settings_digest_u8.try_into().unwrap();
                let payload = MessagePayload::try_from_slice(&rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::Verify {
                    settings_digest,
                    payload,
                })
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
