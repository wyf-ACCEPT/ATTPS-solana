use crate::utils::{
    address_add, address_exists, verify_merkle, verify_signature, verify_zk,
    VerificationError,
};
use solana_program::{
    program_error::ProgramError,
    secp256k1_recover::Secp256k1Pubkey,
};

#[cfg(test)]
mod utils_test {
    use super::*;

    #[test]
    fn test_address_management() {
        // Test empty vector
        let mut addresses = Vec::new();
        let addr1 = [1u8; 20];
        let addr2 = [2u8; 20];
        let addr3 = [3u8; 20];
        assert!(!address_exists(&addresses, &addr1), "Empty vector should not contain any address");

        // Test adding first address
        address_add(&mut addresses, addr1);
        assert!(address_exists(&addresses, &addr1), "Address should exist after adding");
        assert_eq!(addresses.len(), 1, "Vector should have length 1");

        // Test adding duplicate address
        address_add(&mut addresses, addr1);
        assert_eq!(addresses.len(), 1, "Duplicate address should not increase length");
        assert!(address_exists(&addresses, &addr1), "Address should still exist after duplicate add");

        // Test adding multiple unique addresses
        address_add(&mut addresses, addr2);
        address_add(&mut addresses, addr3);
        assert!(address_exists(&addresses, &addr2), "Second address should exist");
        assert!(address_exists(&addresses, &addr3), "Third address should exist");
        assert_eq!(addresses.len(), 3, "Vector should have length 3");

        // Test non-existent address
        let addr4 = [4u8; 20];
        assert!(!address_exists(&addresses, &addr4), "Non-existent address should not be found");

        // Test adding more duplicates
        address_add(&mut addresses, addr1);
        address_add(&mut addresses, addr2);
        assert_eq!(addresses.len(), 3, "Duplicates should not increase length");
    }

    #[test]
    fn test_verify_signature_with_different_v_values() {
        let settings_digest = [1u8; 32];
        let message_hash = [2u8; 32];
        let allowed_signers = vec![[5u8; 20]];
        let threshold = 1;

        // Test both v values (0 and 1, which become 27 and 28)
        for v in [0u8, 1u8] {
            let mut signature_proof = Vec::new();
            signature_proof.extend_from_slice(&[3u8; 32]); // r
            signature_proof.extend_from_slice(&[4u8; 32]); // s
            signature_proof.push(v);

            let result = verify_signature(
                &settings_digest,
                &message_hash,
                &signature_proof,
                &allowed_signers,
                threshold,
            );
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                ProgramError::Custom(2) // InvalidSignature
            ));
        }
    }

    #[test]
    fn test_verify_signature_invalid_proof() {
        let settings_digest = [1u8; 32];
        let message_hash = [2u8; 32];
        let allowed_signers = vec![[5u8; 20]];
        let threshold = 1;

        // Test invalid length
        let invalid_proof = vec![1u8; 10];
        let result = verify_signature(
            &settings_digest,
            &message_hash,
            &invalid_proof,
            &allowed_signers,
            threshold,
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProgramError::Custom(6) // InvalidSignatureProof
        ));

        // Test empty proof
        let empty_proof = vec![];
        let result = verify_signature(
            &settings_digest,
            &message_hash,
            &empty_proof,
            &allowed_signers,
            threshold,
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProgramError::Custom(6) // InvalidSignatureProof
        ));
    }

    #[test]
    fn test_verify_signature_threshold_and_duplicates() {
        let settings_digest = [1u8; 32];
        let message_hash = [2u8; 32];
        let allowed_signers = vec![[5u8; 20], [6u8; 20]];

        // Test threshold > number of signatures
        let mut signature_proof = Vec::new();
        signature_proof.extend_from_slice(&[3u8; 32]); // r1
        signature_proof.extend_from_slice(&[4u8; 32]); // s1
        signature_proof.push(0); // v1

        let result = verify_signature(
            &settings_digest,
            &message_hash,
            &signature_proof,
            &allowed_signers,
            2, // Requires 2 signatures but only 1 provided
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProgramError::Custom(3) // InvalidThreshold
        ));

        // Test duplicate signatures
        let mut signature_proof = Vec::new();
        // First signature
        signature_proof.extend_from_slice(&[3u8; 32]); // r1
        signature_proof.extend_from_slice(&[4u8; 32]); // s1
        signature_proof.push(0); // v1
        // Duplicate signature
        signature_proof.extend_from_slice(&[3u8; 32]); // r1 again
        signature_proof.extend_from_slice(&[4u8; 32]); // s1 again
        signature_proof.push(0); // v1 again

        let result = verify_signature(
            &settings_digest,
            &message_hash,
            &signature_proof,
            &allowed_signers,
            2,
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProgramError::Custom(4) // DuplicateSigner
        ));
    }

    #[test]
    fn test_verify_zk() {
        let settings_digest = [1u8; 32];
        let message_hash = [2u8; 32];
        let zk_proof = vec![3u8; 32];

        let result = verify_zk(&settings_digest, &message_hash, &zk_proof);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProgramError::Custom(1) // UnsupportedProofMethod
        ));
    }

    #[test]
    fn test_verify_merkle() {
        let settings_digest = [1u8; 32];
        let message_hash = [2u8; 32];
        let merkle_proof = vec![3u8; 32];

        let result = verify_merkle(&settings_digest, &message_hash, &merkle_proof);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProgramError::Custom(1) // UnsupportedProofMethod
        ));
    }
}
