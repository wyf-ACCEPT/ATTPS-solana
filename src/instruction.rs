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
    RegisterAgent {
        agent_settings: AgentSettings,
    },

    /// [3] Create & register an agent ([1] + [2])
    ///
    /// 0. [signer] payer
    /// 1. [writable] contract_info
    /// 2. [writable] agent data account
    CreateAndRegisterAgent {
        agent_settings: AgentSettings,
    },

    /// [4] Accept an agent
    ///
    /// 0. [signer] owner
    /// 1. [writable] contract_info
    /// 2. [writable] agent data account
    AcceptAgent {
        agent_id: u128,
    },

    /// [5] Change an agent setting proposal
    ///
    /// 0. [writable] agent data account
    ChangeAgentSettingProposal {
        agent_id: u128,
        new_agent_settings: AgentSettings,
    },

    AcceptAgentSettingProposal {
        agent_id: u128,
    },

    RemoveAgent {
        agent_id: u128,
    },

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
            4 => {
                // For ChangeAgentSettingProposal, parse agent_id (u128) and AgentSettings
                if rest.len() < 16 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let agent_id = u128::from_le_bytes(
                    rest[..16]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );
                let new_agent_settings = AgentSettings::try_from_slice(&rest[16..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::ChangeAgentSettingProposal {
                    agent_id,
                    new_agent_settings,
                })
            }
            5 => {
                // For Verify, parse settings_digest ([u8; 32]) and MessagePayload
                if rest.len() < 32 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let settings_digest: [u8; 32] = rest[..32]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let payload = MessagePayload::try_from_slice(&rest[32..])
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::Verify {
                    settings_digest,
                    payload,
                })
            }
            6 | 7 | 8 => {
                // For AcceptAgent, AcceptAgentSettingProposal, and RemoveAgent
                // Parse agent_id (u128) from remaining bytes
                let agent_id = u128::from_le_bytes(
                    rest.try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );
                match variant {
                    6 => Ok(Self::AcceptAgent { agent_id }),
                    7 => Ok(Self::AcceptAgentSettingProposal { agent_id }),
                    8 => Ok(Self::RemoveAgent { agent_id }),
                    _ => unreachable!(),
                }
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
