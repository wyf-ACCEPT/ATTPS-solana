use crate::error::{AttpsAccountError, VerificationError};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    keccak,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    secp256k1_recover::{secp256k1_recover, Secp256k1Pubkey},
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

/// Check if an Ethereum-style address exists in a vector of addresses
pub(crate) fn address_exists(addresses: &Vec<[u8; 20]>, target: &[u8; 20]) -> bool {
    addresses.iter().any(|addr| addr == target)
}

/// Add an Ethereum-style address to a vector if it doesn't exist
pub(crate) fn address_pushback(addresses: &mut Vec<[u8; 20]>, address: [u8; 20]) {
    addresses.push(address);
}

/// Convert a recovered Secp256k1 public key to an Ethereum address
pub(crate) fn pubkey_to_eth_address(pubkey: &Secp256k1Pubkey) -> [u8; 20] {
    let hash = keccak::hash(&pubkey.to_bytes()).to_bytes();
    hash[12..32].try_into().unwrap()
}

/// Verify a signature proof against a message hash
pub(crate) fn verify_signature(
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

    // Check proof format. Each signature is 65 bytes: [r(32) || s(32) || v(1)]
    if signature_proof.len() % 65 != 0 {
        return Err(VerificationError::InvalidSignatureProof.into());
    }

    let sig_count = signature_proof.len() / 65;
    let mut seen_signatures = Vec::new();

    // Check for duplicate signatures first
    for i in 0..sig_count {
        let start = i * 65;
        let sig = &signature_proof[start..start + 65];
        if seen_signatures.contains(&sig.to_vec()) {
            return Err(VerificationError::DuplicateSigner.into());
        }
        seen_signatures.push(sig.to_vec());
    }

    // Check threshold after duplicate check
    if sig_count < threshold as usize {
        return Err(VerificationError::InvalidThreshold.into());
    }

    let mut valid_signers = Vec::new();

    // Process each signature
    for i in 0..sig_count {
        let start = i * 65;

        // Extract r, s, v components
        let r = &signature_proof[start..start + 32];
        let s = &signature_proof[start + 32..start + 64];
        let recovery_id = signature_proof[start + 64];

        // Combine r and s into signature for recovery
        let mut signature = Vec::with_capacity(64);
        signature.extend_from_slice(r);
        signature.extend_from_slice(s);

        // Recover public key
        let pubkey = secp256k1_recover(message_hash, recovery_id, &signature)
            .map_err(|_| VerificationError::InvalidSignature)?;

        // Convert to Ethereum address
        let recovered_address = pubkey_to_eth_address(&pubkey);

        // Check if signer is allowed
        // TODO: `allowedSigner`
        if !address_exists(allowed_signers, &recovered_address) {
            return Err(VerificationError::SignerNotAllowed.into());
        }

        // Check for duplicate signers
        if address_exists(&valid_signers, &recovered_address) {
            return Err(VerificationError::DuplicateSigner.into());
        }

        valid_signers.push(recovered_address);
    }

    // Check threshold after processing all valid signatures
    if valid_signers.len() < threshold as usize {
        return Err(VerificationError::InvalidThreshold.into());
    }

    Ok(())
}

/// Verify a zero-knowledge proof (currently unsupported)
pub(crate) fn verify_zk(
    _settings_digest: &[u8; 32],
    _message_hash: &[u8; 32],
    _zk_proof: &[u8],
) -> Result<(), ProgramError> {
    Err(VerificationError::UnsupportedProofMethod.into())
}

/// Verify a Merkle proof (currently unsupported)
pub(crate) fn verify_merkle(
    _settings_digest: &[u8; 32],
    _message_hash: &[u8; 32],
    _merkle_proof: &[u8],
) -> Result<(), ProgramError> {
    Err(VerificationError::UnsupportedProofMethod.into())
}

pub(crate) fn create_related_account<'a>(
    program_id: &Pubkey,
    payer_account: &AccountInfo<'a>,
    map_account: &AccountInfo<'a>,
    prefix: &[u8],
    phrase: &[u8],
    data_length: usize,
) -> ProgramResult {
    let (pda_pubkey, bump) = Pubkey::find_program_address(&[prefix, phrase], program_id);
    if pda_pubkey != *map_account.key {
        Err(AttpsAccountError::PdaAccountMismatch.into())
    } else if !map_account.is_writable {
        Err(AttpsAccountError::PdaAccountNotWritable.into())
    } else if !map_account.data_is_empty() {
        Err(AttpsAccountError::PdaAccountAlreadyCreated.into())
    } else {
        let rent = Rent::get()?;
        let rent_lamports = rent.minimum_balance(data_length);
        invoke_signed(
            &system_instruction::create_account(
                payer_account.key,
                map_account.key,
                rent_lamports,
                data_length as u64,
                program_id,
            ),
            &[payer_account.clone(), map_account.clone()],
            &[&[prefix.as_ref(), phrase.as_ref(), &[bump]]],
        )
    }
}

pub(crate) fn write_related_account(map_account: &AccountInfo, content: &[u8]) -> ProgramResult {
    // No need to check because only this program can rewrite the value
    let mut account_data = map_account.data.borrow_mut();
    account_data.copy_from_slice(content);
    Ok(())
}
