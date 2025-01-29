use std::fmt::{self, Display};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::error::StateError;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CounterAccount {
    pub count: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Default, PartialEq)]
pub enum MessageType {
    #[default]
    Event,
    Request,
    Response,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Default, PartialEq)]
pub enum Priority {
    #[default]
    Low,
    Medium,
    High,
}

/// A message header containing routing and metadata information for agent communication.
///
/// The header includes versioning, identifiers, timestamps, and message characteristics
/// used for routing and processing messages between agents.
///
/// # Size
///
/// Total size in bytes = 38 + <sum of all string lengths>, where:
/// - Each string field adds (4 + content length) bytes
/// - Fixed fields add (8 + 1 + 1 + 8 = 18) bytes (timestamps, enums, etc)
///
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Default, PartialEq)]
pub struct AgentHeader {
    pub version: String,
    pub message_id: String,
    pub source_agent_id: String,
    pub source_agent_name: String,
    pub target_agent_id: String,
    pub timestamp: u64,
    pub message_type: MessageType,
    pub priority: Priority,
    pub ttl: u64,
}

/// A collection of cryptographic proofs used for verification.
///
/// # Size
///
/// Each Vec<u8> field requires 4 bytes for length + the content length.
/// - zk_proof: (4 + zk_proof.len()) bytes
/// - merkle_proof: (4 + merkle_proof.len()) bytes
/// - signature_proof: (4 + signature_proof.len()) bytes
///
/// Total size = 12 + (zk_proof.len() + merkle_proof.len() + signature_proof.len())
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Proofs {
    pub zk_proof: Vec<u8>,
    pub merkle_proof: Vec<u8>,
    pub signature_proof: Vec<u8>,
}

/// Metadata information about the message content.
///
/// # Size
///
/// Each String field requires 4 bytes for length + the content length.
/// - content_type: (4 + content_type.len()) bytes
/// - encoding: (4 + encoding.len()) bytes
/// - compression: (4 + compression.len()) bytes
///
/// Total size = 12 + (content_type.len() + encoding.len() + compression.len())
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Metadata {
    pub content_type: String,
    pub encoding: String,
    pub compression: String,
}

/// The main payload of a message, containing data and associated metadata.
///
/// # Size
///
/// Fixed size components:
/// - data_hash: 32 bytes ([u8; 32])
///
/// Variable size components:
/// - data: (4 + data.len()) bytes
/// - proofs: See Proofs struct size calculation
/// - metadata: See Metadata struct size calculation
///
/// Total size = 36 + data.len() + proofs_size + metadata_size
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MessagePayload {
    pub data: Vec<u8>,
    pub data_hash: [u8; 32],
    pub proofs: Proofs,
    pub metadata: Metadata,
}

/// Settings configuration for an agent.
///
/// # Size
///
/// Fixed size components:
/// - threshold: 1 byte (u8)
/// - converter_address: 32 bytes (Pubkey)
///
/// Variable size components:
/// - signers: (4 + 20*signers.len()) bytes (Vec<[u8; 20]> for Ethereum addresses)
/// - agent_header: See AgentHeader struct size calculation
///
/// Total size = 37 + (20*signers.len()) + agent_header_size
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Default, PartialEq)]
pub struct AgentSettings {
    pub signers: Vec<[u8; 20]>,
    pub threshold: u8,
    pub converter_address: Pubkey,
    pub agent_header: AgentHeader,
}

/// Configuration for an agent including its settings and status.
///
/// # Size
///
/// Fixed size components:
/// - config_digest: 32 bytes ([u8; 32])
/// - config_block_number: 8 bytes (u64)
/// - is_active: 1 byte (bool)
///
/// Nested components:
/// - settings: See AgentSettings struct size calculation
///
/// Total size = 41 + settings_size
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Default)]
pub struct AgentConfig {
    pub config_digest: [u8; 32],
    pub config_block_number: u64,
    pub is_active: bool,
    pub settings: AgentSettings,
}

/// Basic information about the contract.
///
/// # Size
///
/// Components:
/// - owner: 32 bytes (Pubkey)
/// - agent_counter: 16 bytes (u128)
/// - type_and_version: (4 + type_and_version.len()) bytes
/// - agent_version: (4 + agent_version.len()) bytes
///
/// Total size = 56 + (type_and_version.len() + agent_version.len())
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ContractInfo {
    pub owner: Pubkey,
    pub agent_counter: u128,
    pub type_and_version: String,
    pub agent_version: String,
}

/// Information about an agent including its settings and status.
///
/// # Size
///
/// Components:
/// - agent_id: 16 bytes (u128)
/// - is_registered: 1 byte (bool)
/// - is_allowed: 1 byte (bool)
/// - is_removed: 1 byte (bool)
/// - is_new_settings: 1 byte (bool)
/// - agent_settings: See AgentSettings struct size calculation
/// - pending_settings: See AgentSettings struct size calculation
/// - agent_config: See AgentConfig struct size calculation
///
/// Total size = 20 + 2 * settings_size + config_size
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Default)]
pub struct AgentInfo {
    pub agent_id: u128, // Unique auto-incrementing identifier
    pub is_registered: bool,
    pub is_allowed: bool,
    pub is_removed: bool,
    pub agent_settings: AgentSettings,
    pub pending_settings: Option<AgentSettings>,
    pub agent_config: AgentConfig,
}

impl ContractInfo {
    pub fn only_owner(&self, owner_account: &AccountInfo) -> ProgramResult {
        if *owner_account.key != self.owner {
            Err(StateError::InvalidOwner.into())
        } else if !owner_account.is_signer {
            Err(StateError::OwnerAccountNotSigner.into())
        } else {
            Ok(())
        }
    }
}

impl Display for AgentSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AgentSettings {{ signers: [")?;
        for (i, signer) in self.signers.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "0x{}", hex::encode(signer))?;
        }
        write!(
            f,
            "], threshold: {}, converter_address: {}, agent_header: {:?} }}",
            self.threshold, self.converter_address, self.agent_header
        )
    }
}

impl Display for AgentConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AgentConfig {{ config_digest: {}, config_block_number: {}, is_active: {}, settings: {} }}",
            hex::encode(self.config_digest),
            self.config_block_number,
            self.is_active,
            self.settings,
        )
    }
}

impl Display for AgentInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Agent state: \n - agent_id: {}\n - is_registered: {}\n - is_allowed: {}\n - is_removed: {}\n",
            self.agent_id, self.is_registered, self.is_allowed, self.is_removed
        )?;
        write!(
            f,
            " - agent_settings: {}\n - pending_settings: {}\n - agent_config: {}",
            self.agent_settings,
            match &self.pending_settings {
                Some(settings) => format!("Some({})", settings),
                None => "None".to_string(),
            },
            self.agent_config
        )
    }
}
