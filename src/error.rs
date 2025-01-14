
use solana_program::program_error::ProgramError;

#[derive(Debug)]
pub enum VerificationError {
    UnsupportedProofMethod,
    InvalidSignature,
    InvalidThreshold,
    DuplicateSigner,
    SignerNotAllowed,
    InvalidSignatureProof,
}

impl From<VerificationError> for ProgramError {
    fn from(e: VerificationError) -> Self {
        ProgramError::Custom(match e {
            VerificationError::UnsupportedProofMethod => 1,
            VerificationError::InvalidSignature => 2,
            VerificationError::InvalidThreshold => 3,
            VerificationError::DuplicateSigner => 4,
            VerificationError::SignerNotAllowed => 5,
            VerificationError::InvalidSignatureProof => 6,
        })
    }
}