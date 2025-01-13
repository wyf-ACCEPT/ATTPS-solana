use crate::state::{
    AgentConfig, AgentConfigState, AgentHeader, AgentSettings, CounterAccount, MessagePayload,
    MessageType, Metadata, Priority, Proofs,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[cfg(test)]
mod state_test {
    use super::*;

    #[test]
    fn test_agent_header_serialization() {
        let header = AgentHeader {
            version: "1.0.0".to_string(),
            message_id: "msg123".to_string(),
            source_agent_id: "agent1".to_string(),
            source_agent_name: "TestAgent".to_string(),
            target_agent_id: "agent2".to_string(),
            timestamp: 1234567890,
            message_type: MessageType::Response,
            priority: Priority::Low,
            ttl: 3600,
        };

        let serialized = borsh::to_vec(&header).expect("Failed to serialize");
        let deserialized = AgentHeader::try_from_slice(&serialized).expect("Failed to deserialize");

        println!(
            "📝 AgentHeader serialized bytes (length ~ {} bytes): {:?}\n\
             ✅ AgentHeader deserialized successfully: {:?}\n",
            serialized.len(),
            serialized,
            deserialized
        );

        assert_eq!(header.version, deserialized.version);
        assert_eq!(header.message_id, deserialized.message_id);
        assert_eq!(header.timestamp, deserialized.timestamp);
    }

    #[test]
    fn test_proofs_serialization() {
        let proofs = Proofs {
            zk_proof: vec![1, 2, 3],
            merkle_proof: vec![4, 5, 6],
            signature_proof: vec![7, 8, 9],
        };

        let serialized = borsh::to_vec(&proofs).expect("Failed to serialize");
        let deserialized = Proofs::try_from_slice(&serialized).expect("Failed to deserialize");

        println!(
            "📝 Proofs serialized bytes (length ~ {} bytes): {:?}\n\
             ✅ Proofs deserialized successfully: {:?}\n",
            serialized.len(),
            serialized,
            deserialized
        );

        assert_eq!(proofs.zk_proof, deserialized.zk_proof);
        assert_eq!(proofs.merkle_proof, deserialized.merkle_proof);
        assert_eq!(proofs.signature_proof, deserialized.signature_proof);
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = Metadata {
            content_type: "application/json".to_string(),
            encoding: "utf-8".to_string(),
            compression: "gzip".to_string(),
        };

        let serialized = borsh::to_vec(&metadata).expect("Failed to serialize");
        let deserialized = Metadata::try_from_slice(&serialized).expect("Failed to deserialize");

        println!(
            "📝 Metadata serialized bytes (length ~ {} bytes): {:?}\n\
             ✅ Metadata deserialized successfully: {:?}\n",
            serialized.len(),
            serialized,
            deserialized
        );

        assert_eq!(metadata.content_type, deserialized.content_type);
        assert_eq!(metadata.encoding, deserialized.encoding);
        assert_eq!(metadata.compression, deserialized.compression);
    }

    #[test]
    fn test_message_payload_serialization() {
        let payload = MessagePayload {
            data: vec![1, 2, 3, 4, 5],
            data_hash: [0; 32],
            proofs: Proofs {
                zk_proof: vec![1, 2, 3],
                merkle_proof: vec![4, 5, 6],
                signature_proof: vec![7, 8, 9],
            },
            metadata: Metadata {
                content_type: "application/json".to_string(),
                encoding: "utf-8".to_string(),
                compression: "gzip".to_string(),
            },
        };

        let serialized = borsh::to_vec(&payload).expect("Failed to serialize");
        let deserialized =
            MessagePayload::try_from_slice(&serialized).expect("Failed to deserialize");

        println!(
            "📝 MessagePayload serialized bytes (length ~ {} bytes): {:?}\n\
             ✅ MessagePayload deserialized successfully: {:?}\n",
            serialized.len(),
            serialized,
            deserialized
        );

        assert_eq!(payload.data, deserialized.data);
        assert_eq!(payload.data_hash, deserialized.data_hash);
        assert_eq!(payload.proofs.zk_proof, deserialized.proofs.zk_proof);
        assert_eq!(
            payload.metadata.content_type,
            deserialized.metadata.content_type
        );
    }

    #[test]
    fn test_agent_settings_serialization() {
        let settings = AgentSettings {
            signers: vec![Pubkey::new_unique(), Pubkey::new_unique()],
            threshold: 2,
            converter_address: Pubkey::new_unique(),
            agent_header: AgentHeader {
                version: "1.0.0".to_string(),
                message_id: "msg123".to_string(),
                source_agent_id: "agent1".to_string(),
                source_agent_name: "TestAgent".to_string(),
                target_agent_id: "agent2".to_string(),
                timestamp: 1234567890,
                message_type: MessageType::Request,
                priority: Priority::High,
                ttl: 3600,
            },
        };

        let serialized = borsh::to_vec(&settings).expect("Failed to serialize");
        let deserialized =
            AgentSettings::try_from_slice(&serialized).expect("Failed to deserialize");

        println!(
            "📝 AgentSettings serialized bytes (length ~ {} bytes): {:?}\n\
             ✅ AgentSettings deserialized successfully: {:?}\n",
            serialized.len(),
            serialized,
            deserialized
        );

        assert_eq!(settings.signers, deserialized.signers);
        assert_eq!(settings.threshold, deserialized.threshold);
        assert_eq!(settings.converter_address, deserialized.converter_address);
        assert_eq!(
            settings.agent_header.version,
            deserialized.agent_header.version
        );
    }

    #[test]
    fn test_agent_config_serialization() {
        let config = AgentConfig {
            config_digest: [1; 32],
            config_block_number: 12345,
            is_active: true,
            settings: AgentSettings {
                signers: vec![Pubkey::new_unique()],
                threshold: 1,
                converter_address: Pubkey::new_unique(),
                agent_header: AgentHeader {
                    version: "1.0.0".to_string(),
                    message_id: "msg123".to_string(),
                    source_agent_id: "agent1".to_string(),
                    source_agent_name: "TestAgent".to_string(),
                    target_agent_id: "agent2".to_string(),
                    timestamp: 1234567890,
                    message_type: MessageType::Request,
                    priority: Priority::High,
                    ttl: 3600,
                },
            },
        };

        let serialized = borsh::to_vec(&config).expect("Failed to serialize");
        let deserialized = AgentConfig::try_from_slice(&serialized).expect("Failed to deserialize");

        println!(
            "📝 AgentConfig serialized bytes (length ~ {} bytes): {:?}\n\
             ✅ AgentConfig deserialized successfully: {:?}\n",
            serialized.len(),
            serialized,
            deserialized
        );

        assert_eq!(config.config_digest, deserialized.config_digest);
        assert_eq!(config.config_block_number, deserialized.config_block_number);
        assert_eq!(config.is_active, deserialized.is_active);
        assert_eq!(config.settings.threshold, deserialized.settings.threshold);
    }

    #[test]
    fn test_agent_config_state_serialization() {
        let config_state = AgentConfigState {
            latest_config_digest: [2; 32],
            configs: vec![AgentConfig {
                config_digest: [1; 32],
                config_block_number: 12345,
                is_active: true,
                settings: AgentSettings {
                    signers: vec![Pubkey::new_unique()],
                    threshold: 1,
                    converter_address: Pubkey::new_unique(),
                    agent_header: AgentHeader {
                        version: "1.0.0".to_string(),
                        message_id: "msg123".to_string(),
                        source_agent_id: "agent1".to_string(),
                        source_agent_name: "TestAgent".to_string(),
                        target_agent_id: "agent2".to_string(),
                        timestamp: 1234567890,
                        message_type: MessageType::Request,
                        priority: Priority::High,
                        ttl: 3600,
                    },
                },
            }],
        };

        let serialized = borsh::to_vec(&config_state).expect("Failed to serialize");
        let deserialized =
            AgentConfigState::try_from_slice(&serialized).expect("Failed to deserialize");

        println!(
            "📝 AgentConfigState serialized bytes (length ~ {} bytes): {:?}\n\
             ✅ AgentConfigState deserialized successfully: {:?}\n",
            serialized.len(),
            serialized,
            deserialized
        );

        assert_eq!(
            config_state.latest_config_digest,
            deserialized.latest_config_digest
        );
        assert_eq!(config_state.configs.len(), deserialized.configs.len());
        assert_eq!(
            config_state.configs[0].config_digest,
            deserialized.configs[0].config_digest
        );
    }
}
