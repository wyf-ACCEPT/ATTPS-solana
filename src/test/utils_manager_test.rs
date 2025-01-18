#[cfg(test)]
mod utils_manager_test {

    use crate::{
        error::AgentHeaderError,
        state::{AgentHeader, AgentSettings, MessageType, Priority},
        utils::AgentManagerUtils,
    };
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_is_valid_uuid() {
        // Valid UUIDs
        assert!(AgentManagerUtils::is_valid_uuid(
            "123e4567-e89b-4d3c-a456-426614174000"
        ));
        assert!(AgentManagerUtils::is_valid_uuid(
            "987fcdeb-51a2-4bc3-9876-543210987654"
        ));

        // Invalid UUIDs
        assert!(!AgentManagerUtils::is_valid_uuid("not-a-uuid"));
        assert!(!AgentManagerUtils::is_valid_uuid(
            "123e4567-e89b-1d3c-a456-426614174000"
        )); // Wrong version
        assert!(!AgentManagerUtils::is_valid_uuid(
            "123e4567-e89b-4d3c-x456-426614174000"
        )); // Invalid hex
        assert!(!AgentManagerUtils::is_valid_uuid("123e4567-e89b-4d3c-a456")); // Too short
        assert!(!AgentManagerUtils::is_valid_uuid(
            "123e4567-e89b-4d3c-a456-4266141740001"
        )); // Too long
    }

    #[test]
    fn test_validate_agent_header() {
        let valid_header = AgentHeader {
            version: "1.0".to_string(),
            message_id: "123e4567-e89b-4d3c-a456-426614174000".to_string(),
            source_agent_id: "987fcdeb-51a2-4bc3-9876-543210987654".to_string(),
            source_agent_name: "Test Agent".to_string(),
            target_agent_id: "555e4567-e89b-4d3c-a456-426614174000".to_string(),
            timestamp: 1234567890,
            message_type: MessageType::Request,
            priority: Priority::High,
            ttl: 3600,
        };

        // Test valid header
        assert!(AgentManagerUtils::validate_agent_header(&valid_header).is_ok());

        // Test invalid version
        let mut invalid_header = valid_header.clone();
        invalid_header.version = "2.0".to_string();
        assert_eq!(
            AgentManagerUtils::validate_agent_header(&invalid_header).unwrap_err(),
            AgentHeaderError::InvalidAgentHeaderVersion.into()
        );

        // Test invalid message_id
        let mut invalid_header = valid_header.clone();
        invalid_header.message_id = "invalid-uuid".to_string();
        assert_eq!(
            AgentManagerUtils::validate_agent_header(&invalid_header).unwrap_err(),
            AgentHeaderError::InvalidAgentHeaderMessageId.into()
        );

        // Test invalid source_agent_id
        let mut invalid_header = valid_header.clone();
        invalid_header.source_agent_id = "invalid-uuid".to_string();
        assert_eq!(
            AgentManagerUtils::validate_agent_header(&invalid_header).unwrap_err(),
            AgentHeaderError::InvalidAgentHeaderAgentId.into()
        );
    }

    #[test]
    fn test_setting_digest_from_settings_data() {
        let agent = Pubkey::from([1u8; 32]);
        let settings = AgentSettings {
            signers: vec![[2u8; 20], [3u8; 20]],
            threshold: 2,
            converter_address: Pubkey::from([4u8; 32]),
            agent_header: AgentHeader {
                version: "1.0".to_string(),
                message_id: "123e4567-e89b-4d3c-a456-426614174000".to_string(),
                source_agent_id: "987fcdeb-51a2-4bc3-9876-543210987654".to_string(),
                source_agent_name: "Test Agent".to_string(),
                target_agent_id: "555e4567-e89b-4d3c-a456-426614174000".to_string(),
                timestamp: 1234567890,
                message_type: MessageType::Event,
                priority: Priority::Low,
                ttl: 3600,
            },
        };

        let digest = AgentManagerUtils::setting_digest_from_settings_data(agent, &settings);

        // Original concat string:
        //  0101010101010101010101010101010101010101010101010101010101010101            // agent address
        //  0202020202020202020202020202020202020202                                    // signers[0]
        //  0303030303030303030303030303030303030303                                    // signers[1]
        //  02                                                                          // threshold
        //  0404040404040404040404040404040404040404040404040404040404040404            // converter_address
        //  312e30                                                                      // version
        //  31323365343536372d653839622d346433632d613435362d343236363134313734303030    // message_id
        //  39383766636465622d353161322d346263332d393837362d353433323130393837363534    // source_agent_id
        //  54657374204167656e74                                                        // source_agent_name
        //  35353565343536372d653839622d346433632d613435362d343236363134313734303030    // target_agent_id
        //  00000000499602d2                                                            // timestamp 0x499602d2 = 1234567890
        //  0000                                                                        // message_type & priority
        //  0000000000000e10                                                            // ttl 0x0e10 = 3600

        let mut expected_digest: [u8; 32] =
            hex::decode("d5c9fb2117d2488a1b7915d0d5fcbb272ec303de31f3ba8c2718d8008899feaa")
                .unwrap()
                .try_into()
                .unwrap();

        expected_digest[0] = 0x01;
        expected_digest[1] = 0x00;
        assert_eq!(digest, expected_digest);
    }
}
