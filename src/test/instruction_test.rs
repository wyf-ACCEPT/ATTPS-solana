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
    use crate::state::ContractInfo;

    use super::*;

    #[tokio::test]
    async fn test_initialize_create_register() {
        use crate::processor::Processor;
        let program_id = Pubkey::new_unique();
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
                    assert_eq!(agent.is_allowed, false);
                    assert_eq!(agent.is_removed, false);
                    assert_eq!(agent.is_new_settings, false);
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
    }
}
