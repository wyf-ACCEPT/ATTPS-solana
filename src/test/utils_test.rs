#[cfg(test)]
mod utils_test {

    use crate::utils::{
        address_exists, address_pushback, pubkey_to_eth_address, verify_merkle, verify_signature,
        verify_zk, VerificationError,
    };
    use solana_program::{program_error::ProgramError, secp256k1_recover::Secp256k1Pubkey};

    #[test]
    fn test_pubkey_to_eth_address() {
        let pubkey = Secp256k1Pubkey::new(
            &hex::decode(
                "5139c6f948e38d3ffa36df836016aea08f37a940a91323f2a785d17be4353e38\
                2b488d0c543c505ec40046afbb2543ba6bb56ca4e26dc6abee13e9add6b7e189",
            )
            .unwrap(),
        );
        assert_eq!(
            Vec::from(pubkey_to_eth_address(&pubkey)),
            hex::decode("052c7707093534035fc2ed60de35e11bebb6486b").unwrap()
        );
    }

    #[test]
    fn test_address_management() {
        // Test empty vector
        let mut addresses = Vec::new();
        let addr1 = [1u8; 20];
        let addr2 = [2u8; 20];
        let addr3 = [3u8; 20];
        assert!(
            !address_exists(&addresses, &addr1),
            "Empty vector should not contain any address"
        );

        // Test adding first address
        address_pushback(&mut addresses, addr1);
        assert!(
            address_exists(&addresses, &addr1),
            "Address should exist after adding"
        );
        assert_eq!(addresses.len(), 1, "Vector should have length 1");

        // Test adding multiple unique addresses
        address_pushback(&mut addresses, addr2);
        address_pushback(&mut addresses, addr3);
        assert!(
            address_exists(&addresses, &addr2),
            "Second address should exist"
        );
        assert!(
            address_exists(&addresses, &addr3),
            "Third address should exist"
        );
        assert_eq!(addresses.len(), 3, "Vector should have length 3");

        // Test non-existent address
        let addr4 = [4u8; 20];
        assert!(
            !address_exists(&addresses, &addr4),
            "Non-existent address should not be found"
        );
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
        signature_proof.extend_from_slice(&[3u8; 32]); // r1
        signature_proof.extend_from_slice(&[4u8; 32]); // s1
        signature_proof.push(0); // v1
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
