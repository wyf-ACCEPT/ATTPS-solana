use solana_program::program_error::ProgramError;

#[derive(Debug)]
pub enum VerificationError {
    UnsupportedProofMethod = 101,
    InvalidSignature,
    InvalidThreshold,
    DuplicateSigner,
    SignerNotAllowed,
    InvalidSignatureProof,
}

impl From<VerificationError> for ProgramError {
    fn from(e: VerificationError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[derive(Debug)]
pub enum AttpsAccountError {
    PdaAccountMismatch = 201,
    PdaAccountNotWritable,
    PdaAccountAlreadyCreated,
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
