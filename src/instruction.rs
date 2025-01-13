use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::state::{AgentSettings, MessagePayload};

#[deprecated]
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CounterInstruction {
    InitializeCounter { initial_value: u64 }, // variant 0
    IncrementCounter,                         // variant 1
    AddAnyValue { amount: u64 },              // variant 2
}

impl CounterInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        // Get the instruction variant from the first byte
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        // Match instruction type and parse the remaining bytes based on the variant
        match variant {
            0 => {
                // For InitializeCounter, parse a u64 from the remaining bytes
                let initial_value = u64::from_le_bytes(
                    rest.try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );
                Ok(Self::InitializeCounter { initial_value })
            }
            1 => Ok(Self::IncrementCounter), // No additional data needed
            2 => {
                // For AddAnyValue, parse a u64 from the remaining bytes
                let amount = u64::from_le_bytes(
                    rest.try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );
                Ok(Self::AddAnyValue { amount })
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum AgentInstruction {
    Initialize,

    CreateAgent,

    RegisterAgent {
        agent_settings: AgentSettings,
    },

    CreateAndRegisterAgent {
        agent_settings: AgentSettings,
    },

    ChangeAgentSettingProposal {
        agent_id: u128,
        agent_settings: AgentSettings,
    },

    Verify {
        settings_digest: [u8; 32],
        payload: MessagePayload,
    },

    AcceptAgent {
        agent_id: u128,
    },

    AcceptAgentSettingProposal {
        agent_id: u128,
    },

    RemoveAgent {
        agent_id: u128,
    },
}

impl AgentInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        todo!()
    }
}
