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

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Proofs {
    pub zk_proof: Vec<u8>,
    pub merkle_proof: Vec<u8>,
    pub signature_proof: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Metadata {
    pub content_type: String,
    pub encoding: String,
    pub compression: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MessagePayload {
    pub data: Vec<u8>,
    pub data_hash: [u8; 32],
    pub proofs: Proofs,
    pub metadata: Metadata,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AgentSettings {
    pub signers: Vec<Pubkey>,
    pub threshold: u8,
    pub converter_address: Pubkey,
    pub agent_header: AgentHeader,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AgentConfig {
    pub config_digest: [u8; 32],
    pub config_block_number: u32,
    pub is_active: bool,
    pub settings: AgentSettings,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AgentConfigState {
    pub latest_config_digest: [u8; 32],
    pub configs: Vec<AgentConfig>,
}
