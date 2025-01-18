pub struct Constants;

impl Constants {
    pub const PREFIX_AGENT_ADDRESS: &'static [u8] = b"agent";
    pub const PREFIX_AGENT_COUNTER: &'static [u8] = b"agent_counter";
    pub const AGENT_COUNTER_SEED: &'static [u8] = b"-1";

    // Account sizes
    pub const AGENT_INFO_SIZE: usize = 528; // Size for AgentInfo account data (matched to actual serialized size)
    pub const AGENT_COUNTER_SIZE: usize = 16; // Size for AgentCounter (u128)
}
