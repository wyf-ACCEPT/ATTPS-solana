use solana_program::{
    program_error::ProgramError,
    secp256k1_recover::{
        secp256k1_recover, Secp256k1Pubkey,
    },
};

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

/// Check if an Ethereum-style address exists in a vector of addresses
pub fn address_exists(addresses: &Vec<[u8; 20]>, target: &[u8; 20]) -> bool {
    addresses.iter().any(|addr| addr == target)
}

/// Add an Ethereum-style address to a vector if it doesn't exist
pub fn address_add(addresses: &mut Vec<[u8; 20]>, address: [u8; 20]) {
    if !address_exists(addresses, &address) {
        addresses.push(address);
    }
}

/// Convert a recovered Secp256k1 public key to an Ethereum address
fn pubkey_to_eth_address(pubkey: &Secp256k1Pubkey) -> [u8; 20] {
    let mut address = [0u8; 20];
    // In Ethereum, address is last 20 bytes of keccak256(public_key[1..])
    // TODO: Implement proper Ethereum address derivation
    address.copy_from_slice(&pubkey.0[44..64]);
    address
}

/// Verify a signature proof against a message hash
pub fn verify_signature(
    _settings_digest: &[u8; 32],
    message_hash: &[u8; 32],
    signature_proof: &[u8],
    allowed_signers: &Vec<[u8; 20]>,
    threshold: u8,
) -> Result<(), ProgramError> {
    // Check empty proof
    if signature_proof.is_empty() {
        return Err(VerificationError::InvalidSignatureProof.into());
    }

    // Check proof format
    // Each signature is 65 bytes: [r(32) || s(32) || v(1)]
    if signature_proof.len() % 65 != 0 {
        return Err(VerificationError::InvalidSignatureProof.into());
    }

    let sig_count = signature_proof.len() / 65;

    // Check threshold before processing signatures
    if sig_count < threshold as usize {
        return Err(VerificationError::InvalidThreshold.into());
    }

    let mut valid_signers = Vec::new();
    let mut seen_signatures = Vec::new();

    // Process each signature
    for i in 0..sig_count {
        let start = i * 65;
        if start + 65 > signature_proof.len() {
            return Err(VerificationError::InvalidSignatureProof.into());
        }

        // Extract r, s, v components
        let r = &signature_proof[start..start + 32];
        let s = &signature_proof[start + 32..start + 64];
        let v = signature_proof[start + 64];

        // Check for duplicate signatures by comparing raw components
        let mut current_sig = Vec::with_capacity(65);
        current_sig.extend_from_slice(r);
        current_sig.extend_from_slice(s);
        current_sig.push(v);
        if seen_signatures.contains(&current_sig) {
            return Err(VerificationError::DuplicateSigner.into());
        }
        seen_signatures.push(current_sig);

        // Combine r and s into signature for recovery
        let mut signature = Vec::with_capacity(64);
        signature.extend_from_slice(r);
        signature.extend_from_slice(s);

        // Recover public key
        let recovery_id = v.checked_add(27).ok_or(VerificationError::InvalidSignature)?;
        let pubkey = secp256k1_recover(
            message_hash,
            recovery_id,
            &signature,
        ).map_err(|_| VerificationError::InvalidSignature)?;

        // Convert to Ethereum address
        let recovered_address = pubkey_to_eth_address(&pubkey);

        // Check if signer is allowed
        if !address_exists(allowed_signers, &recovered_address) {
            return Err(VerificationError::SignerNotAllowed.into());
        }

        valid_signers.push(recovered_address);
    }

    Ok(())
}

/// Verify a zero-knowledge proof (currently unsupported)
pub fn verify_zk(
    _settings_digest: &[u8; 32],
    _message_hash: &[u8; 32],
    _zk_proof: &[u8],
) -> Result<(), ProgramError> {
    Err(VerificationError::UnsupportedProofMethod.into())
}

/// Verify a Merkle proof (currently unsupported)
pub fn verify_merkle(
    _settings_digest: &[u8; 32],
    _message_hash: &[u8; 32],
    _merkle_proof: &[u8],
) -> Result<(), ProgramError> {
    Err(VerificationError::UnsupportedProofMethod.into())
}
