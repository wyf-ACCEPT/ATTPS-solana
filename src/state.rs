use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CounterAccount {
    pub count: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum MessageType {
    Request,
    Response,
    Event,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum Priority {
    High,
    Medium,
    Low,
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
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
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
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
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
/// - config_block_number: 4 bytes (u32)
/// - is_active: 1 byte (bool)
///
/// Nested components:
/// - settings: See AgentSettings struct size calculation
///
/// Total size = 37 + settings_size
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AgentConfig {
    pub config_digest: [u8; 32],
    pub config_block_number: u32,
    pub is_active: bool,
    pub settings: AgentSettings,
}

/// State container for agent configurations.
///
/// # Size
///
/// Fixed size components:
/// - latest_config_digest: 32 bytes ([u8; 32])
///
/// Variable size components:
/// - configs: (4 + configs.len()*config_size) bytes
///   where config_size is the size of AgentConfig
///
/// Total size = 36 + (configs.len() * config_size)
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AgentConfigState {
    pub latest_config_digest: [u8; 32],
    pub configs: Vec<AgentConfig>,
}

/// Basic information about the contract.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ContractInfo {
    pub type_and_version: String,
    pub agent_version: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AgentInfo {
    pub is_allowed: bool,
    pub is_removed: bool,
    pub is_new_settings: bool,
    pub agent_settings: AgentSettings,
    pub agent_config: AgentConfig,
}
