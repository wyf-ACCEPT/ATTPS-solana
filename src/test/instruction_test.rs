use crate::constants::Constants;
use crate::state::{AgentHeader, AgentInfo, AgentSettings, MessageType, Priority};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{pubkey::Pubkey, system_program};
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::Signer,
    transaction::Transaction,
};

#[cfg(test)]
mod instruction_test {
    use solana_sdk::{signature::Keypair, transaction::TransactionError};

    use crate::{
        error::{AgentHeaderError, StateError},
        state::{AgentInfo, ContractInfo},
        utils::AgentManagerUtils,
    };

    use super::*;

    #[tokio::test]
    async fn test_initialize_create_register() {
        use crate::processor::Processor;

        let program_id = Pubkey::new_unique();
        let owner_account = Keypair::new();

        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "attps_solana",
            program_id,
            processor!(Processor::process_instruction),
        )
        .start()
        .await;

        // Get AgentCounter PDA
        let (contract_info_pubkey, _) =
            Pubkey::find_program_address(&[Constants::PREFIX_CONTRACT_INFO, b""], &program_id);

        // Step 1: Initialize the program
        println!("Testing program initialization...");

        let initialize_instruction = Instruction::new_with_bytes(
            program_id,
            &[0], // 0 = Initialize instruction
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(owner_account.pubkey(), false),
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        // Send transaction with initialize instruction
        let mut transaction =
            Transaction::new_with_payer(&[initialize_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Verify counter was created and initialized to 0
        let account = banks_client
            .get_account(contract_info_pubkey)
            .await
            .expect("Failed to get counter account");

        if let Some(account_data) = account {
            let length = u32::from_le_bytes(account_data.data[..4].try_into().unwrap()) as usize;
            let info: ContractInfo =
                ContractInfo::try_from_slice(&account_data.data[4..4 + length])
                    .expect("Failed to deserialize counter data");
            assert_eq!(info.agent_counter, 0);
            println!("✅ Agent counter initialized successfully");
        }

        // Step 2: Create an agent
        println!("Testing agent creation...");

        // Get agent PDA (will be created by instruction)
        let agent_id_bytes = 0u128.to_le_bytes();
        let (agent_pubkey, _) = Pubkey::find_program_address(
            &[Constants::PREFIX_AGENT_ADDRESS, &agent_id_bytes],
            &program_id,
        );
        let create_instruction = Instruction::new_with_bytes(
            program_id,
            &[1], // 1 = CreateAgent instruction
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new(agent_pubkey, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        // Send transaction with create instruction
        let mut transaction =
            Transaction::new_with_payer(&[create_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Step 3: Register an agent
        println!("Testing agent registration...");

        // Create sample agent settings
        let agent_settings = AgentSettings {
            signers: vec![[1u8; 20], [2u8; 20]],
            threshold: 2,
            converter_address: Pubkey::new_unique(),
            agent_header: AgentHeader {
                version: "1.0".to_string(),
                message_id: "123e4567-e89b-4d3c-a456-426614174000".to_string(),
                source_agent_id: "987fcdeb-51a2-4bc3-9876-543210987654".to_string(),
                source_agent_name: "Test Agent".to_string(),
                target_agent_id: "555e4567-e89b-4d3c-a456-426614174000".to_string(),
                timestamp: 1234567890,
                message_type: MessageType::Request,
                priority: Priority::High,
                ttl: 3600,
            },
        };

        // Create RegisterAgent instruction data
        let mut register_instruction_data = vec![2]; // 2 = RegisterAgent instruction
        agent_settings
            .serialize(&mut register_instruction_data)
            .unwrap();

        let register_instruction = Instruction::new_with_bytes(
            program_id,
            &register_instruction_data,
            vec![
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new(agent_pubkey, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        // Send transaction with register instruction
        let mut transaction =
            Transaction::new_with_payer(&[register_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Verify agent was created with correct ID
        let account = banks_client
            .get_account(agent_pubkey)
            .await
            .expect("Failed to get agent account");

        if let Some(account_data) = account {
            println!("Account data size: {}", account_data.data.len());
            println!(
                "Account data (first 32 bytes): {:?}",
                &account_data.data[..32.min(account_data.data.len())]
            );

            let length = u32::from_le_bytes(account_data.data[..4].try_into().unwrap()) as usize;
            println!("Real agent info length: {}", length);

            match AgentInfo::try_from_slice(&account_data.data[4..4 + length]) {
                Ok(agent) => {
                    assert_eq!(agent.agent_id, 0);
                    assert_eq!(agent.is_registered, true);
                    assert_eq!(agent.is_allowed, false);
                    assert_eq!(agent.is_removed, false);
                    println!(
                        "✅ Agent registered successfully with ID: {}",
                        agent.agent_id
                    );
                    println!("✅ Agent settings: {:?}", agent.agent_settings);
                    println!("✅ Agent config: {:?}", agent.agent_config);
                }
                Err(e) => {
                    println!("❌ Failed to deserialize agent data: {:?}", e);
                    println!("Account data length: {}", account_data.data.len());
                    panic!("Failed to deserialize agent data: {:?}", e);
                }
            }
        }

        // Verify counter was incremented
        let account = banks_client
            .get_account(contract_info_pubkey)
            .await
            .expect("Failed to get counter account");

        if let Some(account_data) = account {
            let length = u32::from_le_bytes(account_data.data[..4].try_into().unwrap()) as usize;
            let info: ContractInfo =
                ContractInfo::try_from_slice(&account_data.data[4..4 + length])
                    .expect("Failed to deserialize counter data");
            assert_eq!(info.agent_counter, 1);
            println!("✅ Agent counter incremented successfully");
        }

        // Create & register agent in one transaction
        let mut create_and_register_instruction_data = vec![3]; // 3 = CreateAndRegisterAgent instruction
        let agent_id_bytes_1 = 1u128.to_le_bytes();
        let (agent_pubkey_1, _) = Pubkey::find_program_address(
            &[Constants::PREFIX_AGENT_ADDRESS, &agent_id_bytes_1],
            &program_id,
        );
        agent_settings
            .serialize(&mut create_and_register_instruction_data)
            .unwrap();

        let create_and_register_instruction = Instruction::new_with_bytes(
            program_id,
            &create_and_register_instruction_data,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new(agent_pubkey_1, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let mut transaction =
            Transaction::new_with_payer(&[create_and_register_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
    }

    #[tokio::test]
    async fn test_accept_agent() {
        use crate::processor::Processor;

        let program_id = Pubkey::new_unique();
        let owner_account = Keypair::new();

        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "attps_solana",
            program_id,
            processor!(Processor::process_instruction),
        )
        .start()
        .await;

        // Get contract info PDA
        let (contract_info_pubkey, _) =
            Pubkey::find_program_address(&[Constants::PREFIX_CONTRACT_INFO, b""], &program_id);

        // Step 1: Initialize the program
        println!("Testing program initialization...");
        let initialize_instruction = Instruction::new_with_bytes(
            program_id,
            &[0], // Initialize instruction
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(owner_account.pubkey(), false),
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let mut transaction =
            Transaction::new_with_payer(&[initialize_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Step 2: Create and register an agent
        println!("Testing agent creation and registration...");
        let agent_id_bytes = 0u128.to_le_bytes();
        let (agent_pubkey, _) = Pubkey::find_program_address(
            &[Constants::PREFIX_AGENT_ADDRESS, &agent_id_bytes],
            &program_id,
        );

        // Create sample agent settings
        let agent_settings = AgentSettings {
            signers: vec![[1u8; 20], [2u8; 20]],
            threshold: 2,
            converter_address: Pubkey::new_unique(),
            agent_header: AgentHeader {
                version: "1.0".to_string(),
                message_id: "123e4567-e89b-4d3c-a456-426614174000".to_string(),
                source_agent_id: "987fcdeb-51a2-4bc3-9876-543210987654".to_string(),
                source_agent_name: "Test Agent".to_string(),
                target_agent_id: "555e4567-e89b-4d3c-a456-426614174000".to_string(),
                timestamp: 1234567890,
                message_type: MessageType::Request,
                priority: Priority::High,
                ttl: 3600,
            },
        };

        let mut create_and_register_instruction_data = vec![3]; // CreateAndRegisterAgent
        agent_settings
            .serialize(&mut create_and_register_instruction_data)
            .unwrap();

        let create_and_register_instruction = Instruction::new_with_bytes(
            program_id,
            &create_and_register_instruction_data,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new(agent_pubkey, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let mut transaction =
            Transaction::new_with_payer(&[create_and_register_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Step 3: Accept the agent
        println!("Testing agent acceptance...");
        let accept_instruction_data = {
            let mut data = vec![6]; // AcceptAgent instruction
            data.extend_from_slice(&0u128.to_le_bytes()); // agent_id = 0
            data
        };

        let accept_instruction = Instruction::new_with_bytes(
            program_id,
            &accept_instruction_data,
            vec![
                AccountMeta::new(owner_account.pubkey(), true),
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new(agent_pubkey, false),
            ],
        );

        let mut transaction =
            Transaction::new_with_payer(&[accept_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, &owner_account], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Verify agent state after acceptance
        let account = banks_client
            .get_account(agent_pubkey)
            .await
            .expect("Failed to get agent account");

        if let Some(account_data) = account {
            let length = u32::from_le_bytes(account_data.data[..4].try_into().unwrap()) as usize;
            match AgentInfo::try_from_slice(&account_data.data[4..4 + length]) {
                Ok(agent) => {
                    assert_eq!(agent.agent_id, 0);
                    assert_eq!(agent.is_registered, false);
                    assert_eq!(agent.is_allowed, true);
                    assert_eq!(agent.is_removed, false);
                    println!("✅ Agent accepted successfully");
                    println!("✅ Agent config: {:?}", agent.agent_config);
                }
                Err(e) => {
                    println!("❌ Failed to deserialize agent data: {:?}", e);
                    panic!("Failed to deserialize agent data: {:?}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_change_accept_agent_setting() {
        use crate::processor::Processor;

        let program_id = Pubkey::new_unique();
        let owner_account = Keypair::new();

        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "attps_solana",
            program_id,
            processor!(Processor::process_instruction),
        )
        .start()
        .await;

        // Get contract info PDA
        let (contract_info_pubkey, _) =
            Pubkey::find_program_address(&[Constants::PREFIX_CONTRACT_INFO, b""], &program_id);

        // Step 1: Initialize the program
        println!("Testing program initialization...");
        let initialize_instruction = Instruction::new_with_bytes(
            program_id,
            &[0], // Initialize instruction
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(owner_account.pubkey(), false),
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let mut transaction =
            Transaction::new_with_payer(&[initialize_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Step 2: Create and register an agent
        println!("Testing agent creation and registration...");
        let agent_id_bytes = 0u128.to_le_bytes();
        let (agent_pubkey, _) = Pubkey::find_program_address(
            &[Constants::PREFIX_AGENT_ADDRESS, &agent_id_bytes],
            &program_id,
        );

        // Create initial agent settings
        let initial_settings = AgentSettings {
            signers: vec![[1u8; 20], [2u8; 20]],
            threshold: 2,
            converter_address: Pubkey::new_unique(),
            agent_header: AgentHeader {
                version: "1.0".to_string(),
                message_id: "123e4567-e89b-4d3c-a456-426614174000".to_string(),
                source_agent_id: "987fcdeb-51a2-4bc3-9876-543210987654".to_string(),
                source_agent_name: "Test Agent".to_string(),
                target_agent_id: "555e4567-e89b-4d3c-a456-426614174000".to_string(),
                timestamp: 1234567890,
                message_type: MessageType::Request,
                priority: Priority::High,
                ttl: 3600,
            },
        };

        let mut create_and_register_instruction_data = vec![3]; // CreateAndRegisterAgent
        initial_settings
            .serialize(&mut create_and_register_instruction_data)
            .unwrap();

        let create_and_register_instruction = Instruction::new_with_bytes(
            program_id,
            &create_and_register_instruction_data,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new(agent_pubkey, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let mut transaction =
            Transaction::new_with_payer(&[create_and_register_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Step 3: Accept the agent
        println!("Testing agent acceptance...");
        let accept_instruction_data = {
            let mut data = vec![6]; // AcceptAgent instruction
            data.extend_from_slice(&0u128.to_le_bytes()); // agent_id = 0
            data
        };

        let accept_instruction = Instruction::new_with_bytes(
            program_id,
            &accept_instruction_data,
            vec![
                AccountMeta::new(owner_account.pubkey(), true),
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new(agent_pubkey, false),
            ],
        );

        let mut transaction =
            Transaction::new_with_payer(&[accept_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, &owner_account], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Step 4: Change agent settings
        println!("Testing agent settings change proposal...");

        // Create new settings with different threshold and signers
        let mut new_settings = initial_settings.clone();
        new_settings.threshold = 3;
        new_settings.signers.push([3u8; 20]);

        let change_settings_instruction_data = {
            let mut data = vec![4]; // ChangeAgentSettingProposal instruction
            data.extend_from_slice(&0u128.to_le_bytes()); // agent_id = 0
            new_settings.serialize(&mut data).unwrap();
            data
        };

        let change_settings_instruction = Instruction::new_with_bytes(
            program_id,
            &change_settings_instruction_data,
            vec![AccountMeta::new(agent_pubkey, false)],
        );

        let mut transaction =
            Transaction::new_with_payer(&[change_settings_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Verify agent state after settings change
        let account = banks_client
            .get_account(agent_pubkey)
            .await
            .expect("Failed to get agent account");

        if let Some(account_data) = account {
            let length = u32::from_le_bytes(account_data.data[..4].try_into().unwrap()) as usize;
            match AgentInfo::try_from_slice(&account_data.data[4..4 + length]) {
                Ok(agent) => {
                    assert_eq!(agent.agent_id, 0);
                    assert_eq!(agent.is_registered, false);
                    assert_eq!(agent.is_allowed, true);
                    assert_eq!(agent.is_removed, false);
                    assert!(agent.pending_settings.is_some());

                    // Verify pending settings are updated with new values
                    let pending = agent.pending_settings.as_ref().unwrap();
                    assert_eq!(pending.threshold, 3);
                    assert_eq!(pending.signers.len(), 3);

                    println!("✅ Agent settings change proposed successfully");
                    println!("✅ Pending settings: {:?}", pending);
                }
                Err(e) => {
                    println!("❌ Failed to deserialize agent data: {:?}", e);
                    panic!("Failed to deserialize agent data: {:?}", e);
                }
            }
        }

        // Test error case: Try to change settings with same source_agent_id but different agent_id
        println!("Testing invalid settings change...");
        let mut invalid_settings = new_settings.clone();
        invalid_settings.agent_header.source_agent_id = "different-agent-id".to_string();

        let invalid_change_instruction_data = {
            let mut data = vec![4]; // ChangeAgentSettingProposal instruction
            data.extend_from_slice(&0u128.to_le_bytes()); // agent_id = 0
            invalid_settings.serialize(&mut data).unwrap();
            data
        };

        let invalid_change_instruction = Instruction::new_with_bytes(
            program_id,
            &invalid_change_instruction_data,
            vec![AccountMeta::new(agent_pubkey, false)],
        );

        let mut transaction =
            Transaction::new_with_payer(&[invalid_change_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        if let TransactionError::InstructionError(_, err) = banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap()
        {
            assert_eq!(err, AgentHeaderError::InvalidAgentHeaderAgentId.into());
        }
        println!("✅ Invalid settings change rejected as expected");

        // Test accepting settings proposal
        println!("Testing settings proposal acceptance...");
        let accept_settings_instruction_data = {
            let mut data = vec![7]; // AcceptAgentSettingProposal instruction
            data.extend_from_slice(&0u128.to_le_bytes()); // agent_id = 0
            data
        };

        let accept_settings_instruction = Instruction::new_with_bytes(
            program_id,
            &accept_settings_instruction_data,
            vec![
                AccountMeta::new(owner_account.pubkey(), true),
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new(agent_pubkey, false),
            ],
        );

        let mut transaction = Transaction::new_with_payer(
            &[accept_settings_instruction.clone()],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &owner_account], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Test error case: Try to accept settings with non-signer owner
        println!("Testing non-signer owner rejection...");
        let non_signer_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new_readonly(owner_account.pubkey(), false), // Not a signer, not writable
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new(agent_pubkey, false),
            ],
            data: accept_settings_instruction_data,
        };
        let mut transaction =
            Transaction::new_with_payer(&[non_signer_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        if let TransactionError::InstructionError(_, err) = banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap()
        {
            assert_eq!(err, StateError::OwnerAccountNotSigner.into());
        }
        println!("✅ Non-signer owner rejection verified");

        // Verify final state
        let account = banks_client
            .get_account(agent_pubkey)
            .await
            .expect("Failed to get agent account");

        if let Some(account_data) = account {
            let length = u32::from_le_bytes(account_data.data[..4].try_into().unwrap()) as usize;
            match AgentInfo::try_from_slice(&account_data.data[4..4 + length]) {
                Ok(agent) => {
                    assert_eq!(agent.agent_id, 0);
                    assert_eq!(agent.is_allowed, true);
                    assert_eq!(agent.is_removed, false);
                    assert!(agent.pending_settings.is_none());
                    assert_eq!(agent.agent_settings.threshold, 3);
                    assert_eq!(agent.agent_settings.signers.len(), 3);
                    println!("✅ Settings proposal accepted successfully");
                    println!("✅ Final agent state: {:?}", agent);
                }
                Err(e) => {
                    println!("❌ Failed to deserialize agent data: {:?}", e);
                    panic!("Failed to deserialize agent data: {:?}", e);
                }
            }
        }
    }
}
