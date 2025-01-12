use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CounterAccount {
    pub count: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum MessageType {
    Request,
    Response,
    Event,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
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

