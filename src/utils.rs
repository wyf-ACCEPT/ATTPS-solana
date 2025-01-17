use crate::error::{AttpsAccountError, VerificationError};
use crate::state::{AgentHeader, AgentSettings, MessageType, Priority};
use borsh::{BorshDeserialize, BorshSerialize};
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
        println!("rent get: {:?}", Rent::get());
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

pub(crate) fn write_account_data<Data: BorshSerialize>(
    data_account: &AccountInfo,
    content: Data,
) -> ProgramResult {
    let mut account_data = &mut data_account.data.borrow_mut()[..];
    content
        .serialize(&mut account_data)
        .map_err(|_| ProgramError::InvalidAccountData)
}

pub(crate) fn read_account_data<Data: BorshDeserialize>(
    data_account: &AccountInfo,
) -> Result<Data, ProgramError> {
    let account_data = &data_account.data.borrow()[..];
    Data::try_from_slice(account_data).map_err(|_| ProgramError::InvalidAccountData)
}

/// Validates a UUID string according to v4 format
pub fn is_valid_uuid(uuid: &str) -> bool {
    if uuid.len() != 36 {
        return false;
    }

    let bytes = uuid.as_bytes();

    // Check hyphens at positions 8, 13, 18, 23
    if bytes[8] != b'-' || bytes[13] != b'-' || bytes[18] != b'-' || bytes[23] != b'-' {
        return false;
    }

    // Check version number (position 14 must be '4')
    if bytes[14] != b'4' {
        return false;
    }

    // Check variant (position 19 must be '8', '9', 'a', 'b', 'A', 'B')
    match bytes[19] {
        b'8' | b'9' | b'a' | b'b' | b'A' | b'B' => (),
        _ => return false,
    }

    // Verify all other characters are hexadecimal
    for (i, &byte) in bytes.iter().enumerate() {
        if i != 8 && i != 13 && i != 18 && i != 23 {
            match byte {
                b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => continue,
                _ => return false,
            }
        }
    }

    true
}

/// Validates message type is within allowed range
pub fn is_valid_message_type(message_type: &MessageType) -> bool {
    matches!(
        message_type,
        MessageType::Request | MessageType::Response | MessageType::Event
    )
}

/// Validates priority is within allowed range
pub fn is_valid_priority(priority: &Priority) -> bool {
    matches!(priority, Priority::High | Priority::Medium | Priority::Low)
}

/// Validates an agent header according to protocol rules
pub fn validate_agent_header(header: &AgentHeader) -> Result<(), ProgramError> {
    // Check version matches "1.0"
    if header.version != "1.0" {
        return Err(VerificationError::InvalidSignatureProof.into());
    }

    // Validate source agent ID
    if !is_valid_uuid(&header.source_agent_id) {
        return Err(VerificationError::InvalidSignatureProof.into());
    }

    // Validate message ID
    if !is_valid_uuid(&header.message_id) {
        return Err(VerificationError::InvalidSignatureProof.into());
    }

    // Validate message type
    if !is_valid_message_type(&header.message_type) {
        return Err(VerificationError::InvalidSignatureProof.into());
    }

    // Validate priority
    if !is_valid_priority(&header.priority) {
        return Err(VerificationError::InvalidSignatureProof.into());
    }

    Ok(())
}

/// Computes a unique digest from agent settings using keccak256
/// The digest includes a version prefix (0x0100) in the most significant 16 bits
pub fn setting_digest_from_settings_data(agent: [u8; 20], settings: &AgentSettings) -> [u8; 32] {
    // Encode all fields in the same order as Solidity's abi.encode
    let mut data = Vec::new();
    data.extend_from_slice(&agent);

    // Encode signers array
    for signer in &settings.signers {
        data.extend_from_slice(signer);
    }

    data.push(settings.threshold);
    data.extend_from_slice(&settings.converter_address.to_bytes());
    data.extend_from_slice(settings.agent_header.version.as_bytes());
    data.extend_from_slice(settings.agent_header.message_id.as_bytes());
    data.extend_from_slice(settings.agent_header.source_agent_id.as_bytes());
    data.extend_from_slice(settings.agent_header.source_agent_name.as_bytes());
    data.extend_from_slice(settings.agent_header.target_agent_id.as_bytes());
    data.extend_from_slice(&settings.agent_header.timestamp.to_be_bytes());
    data.extend_from_slice(&[settings.agent_header.message_type.clone() as u8]);
    data.extend_from_slice(&[settings.agent_header.priority.clone() as u8]);
    data.extend_from_slice(&settings.agent_header.ttl.to_be_bytes());

    // Compute keccak256 hash
    let mut hash = keccak::hash(&data).to_bytes();

    // Apply prefix mask logic
    // 0xFFFF000000000000000000000000000000000000000000000000000000000000
    let prefix_mask: [u8; 32] = {
        let mut mask = [0u8; 32];
        mask[0] = 0xFF;
        mask[1] = 0xFF;
        mask
    };

    // 0x0100000000000000000000000000000000000000000000000000000000000000
    let prefix: [u8; 32] = {
        let mut p = [0u8; 32];
        p[0] = 0x01;
        p[1] = 0x00;
        p
    };

    // Apply mask: (prefix & prefix_mask) | (hash & ~prefix_mask)
    for i in 0..32 {
        hash[i] = (prefix[i] & prefix_mask[i]) | (hash[i] & !prefix_mask[i]);
    }

    hash
}
