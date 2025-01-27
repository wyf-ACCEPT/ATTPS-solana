use solana_program::{instruction::InstructionError, program_error::ProgramError};

#[derive(Debug)]
pub enum VerificationError {
    UnsupportedProofMethod = 101,
    InvalidSignature,
    InvalidThreshold,
    DuplicateSigner,
    SignerNotAllowed,
    InvalidSignatureProof,
    InvalidDataHash = 151,
    InvalidProofData,
}

impl From<VerificationError> for ProgramError {
    fn from(e: VerificationError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl From<VerificationError> for InstructionError {
    fn from(e: VerificationError) -> Self {
        InstructionError::Custom(e as u32)
    }
}

#[derive(Debug)]
pub enum AttpsAccountError {
    PdaAccountMismatch = 201,
    PdaAccountNotWritable,
    PdaAccountAlreadyCreated,
    PdaAccountNotOwned,
    AgentAlreadyRegistered,
    AgentAlreadyAllowed,
    AgentAlreadyRemoved,
    AgentNotRegistered,
    InvalidAllowedAgent,
    InvalidAgentConfig,
    DuplicateAgentSettings,
}

impl From<AttpsAccountError> for ProgramError {
    fn from(e: AttpsAccountError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[derive(Debug)]
pub enum AgentHeaderError {
    InvalidAgentHeaderVersion = 301,
    InvalidAgentHeaderAgentId,
    InvalidAgentHeaderMessageId,
}

impl From<AgentHeaderError> for ProgramError {
    fn from(e: AgentHeaderError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl From<AgentHeaderError> for InstructionError {
    fn from(e: AgentHeaderError) -> Self {
        InstructionError::Custom(e as u32)
    }
}

#[derive(Debug)]
pub enum StateError {
    InvalidOwner = 401,
    OwnerAccountNotSigner,
}

impl From<StateError> for ProgramError {
    fn from(e: StateError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl From<StateError> for InstructionError {
    fn from(e: StateError) -> Self {
        InstructionError::Custom(e as u32)
    }
}
