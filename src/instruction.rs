use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{msg, program_error::ProgramError};

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
    AcceptAgent { agent_id: u128 },

    /// [5] Change an agent setting proposal
    ///
    /// 0. [writable] agent data account
    ChangeAgentSettingProposal {
        agent_id: u128,
        proposed_settings: AgentSettings,
    },

    /// [6] Accept an agent setting proposal
    ///
    /// 0. [signer] owner
    /// 1. [writable] contract_info
    /// 2. [writable] agent data account
    AcceptAgentSettingProposal { agent_id: u128 },

    /// [7] Remove an agent
    RemoveAgent { agent_id: u128 },

    /// [8] Verify a message of an agent
    Verify {
        agent_id: u128,
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
            4 | 5 | 6 | 7 | 8 => {
                // For All the remaining instructions
                // Parse agent_id (u128) from remaining bytes first
                let (agent_id_bytes, rest) = rest.split_at(16);
                let agent_id = u128::from_le_bytes(agent_id_bytes.try_into().unwrap());
                match variant {
                    4 => Ok(Self::AcceptAgent { agent_id }),
                    5 => {
                        // For ChangeAgentSettingProposal, parse agent_id (u128) and AgentSettings
                        let proposed_settings = AgentSettings::try_from_slice(&rest)
                            .map_err(|_| ProgramError::InvalidInstructionData)?;
                        msg!("\nProposed settings: {:?}", proposed_settings);
                        Ok(Self::ChangeAgentSettingProposal {
                            agent_id,
                            proposed_settings,
                        })
                    }
                    6 => Ok(Self::AcceptAgentSettingProposal { agent_id }),
                    7 => Ok(Self::RemoveAgent { agent_id }),
                    8 => {
                        // For Verify, parse settings_digest ([u8; 32]) and MessagePayload
                        let (settings_digest_u8, rest) = rest.split_at(32);
                        let settings_digest = settings_digest_u8.try_into().unwrap();
                        let payload = MessagePayload::try_from_slice(&rest)
                            .map_err(|_| ProgramError::InvalidInstructionData)?;
                        Ok(Self::Verify {
                            agent_id,
                            settings_digest,
                            payload,
                        })
                    }
                    _ => unreachable!(),
                }
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
