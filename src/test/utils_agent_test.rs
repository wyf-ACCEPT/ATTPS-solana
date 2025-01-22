#[cfg(test)]
mod utils_agent_test {

    use crate::{error::VerificationError, utils::AgentUtils};
    use solana_program::secp256k1_recover::Secp256k1Pubkey;

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
            Vec::from(AgentUtils::pubkey_to_eth_address(&pubkey)),
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
            !AgentUtils::address_exists(&addresses, &addr1),
            "Empty vector should not contain any address"
        );

        // Test adding first address
        addresses.push(addr1);
        assert!(
            AgentUtils::address_exists(&addresses, &addr1),
            "Address should exist after adding"
        );
        assert_eq!(addresses.len(), 1, "Vector should have length 1");

        // Test adding multiple unique addresses
        addresses.push(addr2);
        addresses.push(addr3);
        assert!(
            AgentUtils::address_exists(&addresses, &addr2),
            "Second address should exist"
        );
        assert!(
            AgentUtils::address_exists(&addresses, &addr3),
            "Third address should exist"
        );
        assert_eq!(addresses.len(), 3, "Vector should have length 3");

        // Test non-existent address
        let addr4 = [4u8; 20];
        assert!(
            !AgentUtils::address_exists(&addresses, &addr4),
            "Non-existent address should not be found"
        );
    }

    #[test]
    fn test_verify_signature_invalid_proof() {
        let settings_digest = [1u8; 32];
        let message_hash = [2u8; 32];
        let allowed_signers = vec![[5u8; 20]];
        let threshold = 1;

        // Test invalid length
        let invalid_proof = vec![1u8; 10];
        let result = AgentUtils::verify_signature(
            &settings_digest,
            &message_hash,
            &invalid_proof,
            &allowed_signers,
            threshold,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            VerificationError::InvalidSignatureProof.into()
        );

        // Test empty proof
        let empty_proof = vec![];
        let result = AgentUtils::verify_signature(
            &settings_digest,
            &message_hash,
            &empty_proof,
            &allowed_signers,
            threshold,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            VerificationError::InvalidSignatureProof.into()
        );
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

        let result = AgentUtils::verify_signature(
            &settings_digest,
            &message_hash,
            &signature_proof,
            &allowed_signers,
            2, // Requires 2 signatures but only 1 provided
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            VerificationError::InvalidThreshold.into()
        );

        // Test duplicate signatures
        let mut signature_proof = Vec::new();
        signature_proof.extend_from_slice(&[3u8; 32]); // r1
        signature_proof.extend_from_slice(&[4u8; 32]); // s1
        signature_proof.push(0); // v1
        signature_proof.extend_from_slice(&[3u8; 32]); // r1 again
        signature_proof.extend_from_slice(&[4u8; 32]); // s1 again
        signature_proof.push(0); // v1 again

        let result = AgentUtils::verify_signature(
            &settings_digest,
            &message_hash,
            &signature_proof,
            &allowed_signers,
            2,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            VerificationError::DuplicateSigner.into()
        );
    }

    #[test]
    fn test_verify_signature_success() {
        // Message hash: "hello world!"
        let message_hash =
            hex::decode("57caa176af1ac0433c5df30e8dabcd2ec1af1e92a26eced5f719b88458777cd6")
                .unwrap()
                .try_into()
                .unwrap();

        // Allowed signers
        let signer1 = hex::decode("6370eF2f4Db3611D657b90667De398a2Cc2a370C").unwrap();
        let signer2 = hex::decode("677bb7270e0b03f0A2993A697654fb8Ecb6deE91").unwrap();
        let allowed_signers: Vec<[u8; 20]> = vec![
            signer1[..20].try_into().unwrap(),
            signer2[..20].try_into().unwrap(),
        ];

        // Signature 1
        let mut sig1 = Vec::new();
        sig1.extend_from_slice(
            &hex::decode("26eebcfa4a0f21ed6e03722eebba46377a6d394686d83cb47be28fd1bf6b984a")
                .unwrap(),
        ); // r
        sig1.extend_from_slice(
            &hex::decode("119870fe1cc35ddfae19900ce8afcd88531bb000e8bc4c82dcd5cab0fe7db54d")
                .unwrap(),
        ); // s
        sig1.push(1); // yParity (recovery_id)

        // Signature 2
        let mut sig2 = Vec::new();
        sig2.extend_from_slice(
            &hex::decode("116b0c5a99594bc09a5b9e2e2b694f5c19fc9be1b266f10c06572e9f73350e46")
                .unwrap(),
        ); // r
        sig2.extend_from_slice(
            &hex::decode("48c8e675d1defa0033781a41c73ec2c7edfd59aefa32ddb8bed6119b51c28b2d")
                .unwrap(),
        ); // s
        sig2.push(0); // yParity (recovery_id)

        // Test with single signature (threshold = 1)
        let settings_digest = [0u8; 32];
        let result = AgentUtils::verify_signature(
            &settings_digest,
            &message_hash,
            &sig1,
            &allowed_signers,
            1,
        );
        assert!(result.is_ok(), "Single signature verification failed");

        // Test with not-enough signatures
        let result = AgentUtils::verify_signature(
            &settings_digest,
            &message_hash,
            &sig1,
            &allowed_signers,
            2,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            VerificationError::InvalidThreshold.into()
        );

        // Test with both signatures (threshold = 2)
        let mut combined_sig = Vec::new();
        combined_sig.extend_from_slice(&sig1);
        combined_sig.extend_from_slice(&sig2);
        let result = AgentUtils::verify_signature(
            &settings_digest,
            &message_hash,
            &combined_sig,
            &allowed_signers,
            2,
        );
        assert!(result.is_ok(), "Combined signatures verification failed");
    }

    #[test]
    fn test_verify_zk() {
        let settings_digest = [1u8; 32];
        let message_hash = [2u8; 32];
        let zk_proof = vec![3u8; 32];

        let result = AgentUtils::verify_zk(&settings_digest, &message_hash, &zk_proof);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            VerificationError::UnsupportedProofMethod.into()
        );
    }

    #[test]
    fn test_verify_merkle() {
        let settings_digest = [1u8; 32];
        let message_hash = [2u8; 32];
        let merkle_proof = vec![3u8; 32];

        let result = AgentUtils::verify_merkle(&settings_digest, &message_hash, &merkle_proof);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            VerificationError::UnsupportedProofMethod.into()
        );
    }
}
