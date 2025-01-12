#[cfg(test)]
mod test {
    use crate::state::{
        AgentConfig, AgentConfigState, AgentHeader, AgentSettings, CounterAccount, Metadata,
        MessagePayload, MessageType, Priority, Proofs,
    };
    use crate::processor::process_instruction;
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_program::pubkey::Pubkey;
    use solana_program_test::*;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        signature::{Keypair, Signer},
        system_program,
        transaction::Transaction,
    };

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
        println!("\nSerialized bytes: {:?}", serialized);
        println!("Serialized length: {} bytes", serialized.len());
        
        let deserialized = AgentHeader::try_from_slice(&serialized).expect("Failed to deserialize");
        println!("✅ AgentHeader deserialized successfully: {:?}", deserialized);

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
        println!("\nSerialized bytes: {:?}", serialized);
        println!("Serialized length: {} bytes", serialized.len());
        
        let deserialized = Proofs::try_from_slice(&serialized).expect("Failed to deserialize");
        println!("✅ Proofs deserialized successfully: {:?}", deserialized);

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
        println!("\nSerialized bytes: {:?}", serialized);
        println!("Serialized length: {} bytes", serialized.len());
        
        let deserialized = Metadata::try_from_slice(&serialized).expect("Failed to deserialize");
        println!("✅ Metadata deserialized successfully: {:?}", deserialized);

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
        println!("\nSerialized bytes: {:?}", serialized);
        println!("Serialized length: {} bytes", serialized.len());
        
        let deserialized = MessagePayload::try_from_slice(&serialized).expect("Failed to deserialize");
        println!("✅ MessagePayload deserialized successfully: {:?}", deserialized);

        assert_eq!(payload.data, deserialized.data);
        assert_eq!(payload.data_hash, deserialized.data_hash);
        assert_eq!(payload.proofs.zk_proof, deserialized.proofs.zk_proof);
        assert_eq!(payload.metadata.content_type, deserialized.metadata.content_type);
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
        println!("\nSerialized bytes: {:?}", serialized);
        println!("Serialized length: {} bytes", serialized.len());
        
        let deserialized = AgentSettings::try_from_slice(&serialized).expect("Failed to deserialize");
        println!("✅ AgentSettings deserialized successfully: {:?}", deserialized);

        assert_eq!(settings.signers, deserialized.signers);
        assert_eq!(settings.threshold, deserialized.threshold);
        assert_eq!(settings.converter_address, deserialized.converter_address);
        assert_eq!(settings.agent_header.version, deserialized.agent_header.version);
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
        println!("\nSerialized bytes: {:?}", serialized);
        println!("Serialized length: {} bytes", serialized.len());
        
        let deserialized = AgentConfig::try_from_slice(&serialized).expect("Failed to deserialize");
        println!("✅ AgentConfig deserialized successfully: {:?}", deserialized);

        assert_eq!(config.config_digest, deserialized.config_digest);
        assert_eq!(config.config_block_number, deserialized.config_block_number);
        assert_eq!(config.is_active, deserialized.is_active);
        assert_eq!(config.settings.threshold, deserialized.settings.threshold);
    }

    #[test]
    fn test_agent_config_state_serialization() {
        let config_state = AgentConfigState {
            latest_config_digest: [2; 32],
            configs: vec![
                AgentConfig {
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
                },
            ],
        };

        let serialized = borsh::to_vec(&config_state).expect("Failed to serialize");
        println!("\nSerialized bytes: {:?}", serialized);
        println!("Serialized length: {} bytes", serialized.len());
        
        let deserialized = AgentConfigState::try_from_slice(&serialized).expect("Failed to deserialize");
        println!("✅ AgentConfigState deserialized successfully: {:?}", deserialized);

        assert_eq!(config_state.latest_config_digest, deserialized.latest_config_digest);
        assert_eq!(config_state.configs.len(), deserialized.configs.len());
        assert_eq!(config_state.configs[0].config_digest, deserialized.configs[0].config_digest);
    }

    #[tokio::test]
    async fn test_counter_program() {
        let program_id = Pubkey::new_unique();
        let (mut banks_client, payer, recent_blockhash) =
            ProgramTest::new("attps_solana", program_id, processor!(process_instruction))
                .start()
                .await;

        // Create a new keypair to use as the address for our counter account
        let counter_keypair = Keypair::new();
        let initial_value: u64 = 42;

        // Step 1: Initialize the counter
        println!("Testing counter initialization...");

        // Create initialization instruction
        let mut init_instruction_data = vec![0]; // 0 = initialize instruction
        init_instruction_data.extend_from_slice(&initial_value.to_le_bytes());

        let initialize_instruction = Instruction::new_with_bytes(
            program_id,
            &init_instruction_data,
            vec![
                AccountMeta::new(counter_keypair.pubkey(), true),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        // Send transaction with initialize instruction
        let mut transaction =
            Transaction::new_with_payer(&[initialize_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, &counter_keypair], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Check account data
        let account = banks_client
            .get_account(counter_keypair.pubkey())
            .await
            .expect("Failed to get counter account");

        if let Some(account_data) = account {
            let counter: CounterAccount = CounterAccount::try_from_slice(&account_data.data)
                .expect("Failed to deserialize counter data");
            assert_eq!(counter.count, 42);
            println!(
                "✅ Counter initialized successfully with value: {}",
                counter.count
            );
        }

        // Step 2: Increment the counter
        println!("Testing counter increment...");

        // Create increment instruction
        let increment_instruction = Instruction::new_with_bytes(
            program_id,
            &[1], // 1 = increment instruction
            vec![AccountMeta::new(counter_keypair.pubkey(), true)],
        );

        // Send transaction with increment instruction
        let mut transaction =
            Transaction::new_with_payer(&[increment_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, &counter_keypair], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Check account data
        let account = banks_client
            .get_account(counter_keypair.pubkey())
            .await
            .expect("Failed to get counter account");

        if let Some(account_data) = account {
            let counter: CounterAccount = CounterAccount::try_from_slice(&account_data.data)
                .expect("Failed to deserialize counter data");
            assert_eq!(counter.count, 43);
            println!("✅ Counter incremented successfully to: {}", counter.count);
        }
    }
}
