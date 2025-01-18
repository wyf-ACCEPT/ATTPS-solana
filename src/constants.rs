pub struct Constants;

impl Constants {
    pub const PREFIX_CONTRACT_INFO: &'static [u8] = b"contract-info";
    pub const PREFIX_AGENT_ADDRESS: &'static [u8] = b"agent";

    // Account sizes
    pub const SIZE_AGENT_HEADER: usize = 256;
    pub const SIZE_PROOF: usize = 256;
    pub const SIZE_METADATA: usize = 256;
    pub const SIZE_MESSAGE_PAYLOAD: usize = 2048;
    pub const SIZE_AGENT_SETTINGS: usize = 2048;
    pub const SIZE_AGENT_CONFIG: usize = 2048;
    pub const SIZE_CONTRACT_INFO: usize = 256;
    pub const SIZE_AGENT_INFO: usize = 4096;

}
