use crate::constants::Constants;
use crate::state::{AgentConfig, AgentHeader, AgentInfo, AgentSettings, MessageType, Priority};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{pubkey::Pubkey, system_program};
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::Signer,
    transaction::Transaction,
};
use std::fmt::{self, Display};

impl Display for AgentSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AgentSettings {{ signers: [")?;
        for (i, signer) in self.signers.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "0x{}", hex::encode(signer))?;
        }
        write!(
            f,
            "], threshold: {}, converter_address: {}, agent_header: {:?} }}",
            self.threshold, self.converter_address, self.agent_header
        )
    }
}

impl Display for AgentConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AgentConfig {{ config_digest: {}, config_block_number: {}, is_active: {}, settings: {} }}",
            hex::encode(self.config_digest),
            self.config_block_number,
            self.is_active,
            self.settings,
        )
    }
}

impl Display for AgentInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Agent state: \n - agent_id: {}\n - is_registered: {}\n - is_allowed: {}\n - is_removed: {}\n",
            self.agent_id, self.is_registered, self.is_allowed, self.is_removed
        )?;
        write!(
            f,
            " - agent_settings: {}\n - pending_settings: {}\n - agent_config: {}",
            self.agent_settings,
            match &self.pending_settings {
                Some(settings) => format!("Some({})", settings),
                None => "None".to_string(),
            },
            self.agent_config
        )
    }
}

#[cfg(test)]
mod instruction_test {
    use solana_sdk::{signature::Keypair, transaction::TransactionError};

    use crate::{
        error::{AgentHeaderError, StateError, VerificationError},
        processor::Processor,
        state::{AgentInfo, ContractInfo, MessagePayload, Metadata, Proofs},
    };
    use solana_program::keccak;

    use super::*;

    #[tokio::test]
    async fn test_initialize_create_register_remove() {
        // 1. Setup environment
        let program_id = Pubkey::new_unique();
        let owner_account = Keypair::new();

        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "attps_solana",
            program_id,
            processor!(Processor::process_instruction),
        )
        .start()
        .await;

        let (contract_info_pubkey, _) =
            Pubkey::find_program_address(&[Constants::PREFIX_CONTRACT_INFO, b""], &program_id);

        // 2. Initialize the program
        println!("\nTesting program initialization...");

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

        let mut transaction =
            Transaction::new_with_payer(&[initialize_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

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

        // 3. Create an agent
        println!("\nTesting agent creation...");

        let agent_id: u128 = 0;
        let (agent_pubkey, _) = Pubkey::find_program_address(
            &[Constants::PREFIX_AGENT_ADDRESS, &agent_id.to_le_bytes()],
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

        let mut transaction =
            Transaction::new_with_payer(&[create_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // 4. Register an agent
        println!("\nTesting agent registration...");

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

        let mut transaction =
            Transaction::new_with_payer(&[register_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let account = banks_client
            .get_account(agent_pubkey)
            .await
            .expect("Failed to get agent account");

        if let Some(account_data) = account {
            println!("\tAccount data size: {}", account_data.data.len());
            println!(
                "\tAccount data (first 32 bytes): {:?}",
                &account_data.data[..32.min(account_data.data.len())]
            );

            let length = u32::from_le_bytes(account_data.data[..4].try_into().unwrap()) as usize;
            println!("\tReal agent info length: {}", length);

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
                    println!("✅ {}", agent);
                }
                Err(e) => {
                    println!("❌ Failed to deserialize agent data: {:?}", e);
                    println!("Account data length: {}", account_data.data.len());
                    panic!("Failed to deserialize agent data: {:?}", e);
                }
            }
        }

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

        // 5. Create & register agent in one transaction
        println!("\nTesting agent creation and registration...");
        let mut create_and_register_instruction_data = vec![3]; // 3 = CreateAndRegisterAgent instruction
        let agent_id_1: u128 = 1;
        let (agent_pubkey_1, _) = Pubkey::find_program_address(
            &[Constants::PREFIX_AGENT_ADDRESS, &agent_id_1.to_le_bytes()],
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

        // 6. Accept the agent
        println!("\nTesting agent acceptance...");
        let accept_instruction_data = {
            let mut data = vec![4]; // 4 = AcceptAgent instruction
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
                    println!("✅ {}", agent);
                }
                Err(e) => {
                    println!("❌ Failed to deserialize agent data: {:?}", e);
                    panic!("Failed to deserialize agent data: {:?}", e);
                }
            }
        }

        // 7. Remove the agent
        println!("\nTesting agent removal...");
        let remove_instruction_data = vec![7]; // 7 = RemoveAgent instruction

        let remove_instruction = Instruction::new_with_bytes(
            program_id,
            &remove_instruction_data,
            vec![
                AccountMeta::new(owner_account.pubkey(), true),
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new(agent_pubkey, false),
            ],
        );

        let mut transaction =
            Transaction::new_with_payer(&[remove_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, &owner_account], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

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
                    assert_eq!(agent.is_allowed, false);
                    assert_eq!(agent.is_removed, true);
                    println!("✅ Agent removed successfully");
                    println!("✅ {}", agent);
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
        // 1. Setup environment
        let program_id = Pubkey::new_unique();
        let owner_account = Keypair::new();

        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "attps_solana",
            program_id,
            processor!(Processor::process_instruction),
        )
        .start()
        .await;

        let (contract_info_pubkey, _) =
            Pubkey::find_program_address(&[Constants::PREFIX_CONTRACT_INFO, b""], &program_id);

        // 2. Initialize the program
        println!("\nTesting program initialization...");
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

        // 3. Create and register an agent
        println!("\nTesting agent creation and registration...");
        let agent_id: u128 = 0;
        let (agent_pubkey, _) = Pubkey::find_program_address(
            &[Constants::PREFIX_AGENT_ADDRESS, &agent_id.to_le_bytes()],
            &program_id,
        );

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

        // 4. Accept the agent
        println!("\nTesting agent acceptance...");
        let accept_instruction_data = {
            let mut data = vec![4]; // AcceptAgent instruction
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

        // 5. Change agent settings
        println!("\nTesting agent settings change proposal...");

        let mut new_settings = initial_settings.clone();
        new_settings.threshold = 3;
        new_settings.signers.push([3u8; 20]);

        let change_settings_instruction_data = {
            let mut data = vec![5]; // ChangeAgentSettingProposal instruction
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
                    println!("✅ Pending settings: {:?}", agent.pending_settings);
                }
                Err(e) => {
                    println!("❌ Failed to deserialize agent data: {:?}", e);
                    panic!("Failed to deserialize agent data: {:?}", e);
                }
            }
        }

        // 6. Test error case: Try to change settings with same source_agent_id but different agent_id
        println!("\nTesting invalid settings change...");
        let mut invalid_settings = new_settings.clone();
        invalid_settings.agent_header.source_agent_id = "different-agent-id".to_string();

        let invalid_change_instruction_data = {
            let mut data = vec![5]; // ChangeAgentSettingProposal instruction
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

        // 7. Accept settings proposal
        println!("\nTesting settings proposal acceptance...");
        let accept_settings_instruction_data = vec![6]; // AcceptAgentSettingProposal instruction

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

        // 8. Test error case: Try to accept settings with non-signer owner
        println!("\nTesting non-signer owner rejection...");
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

        // 9. Verify final state
        println!("\nTesting final state verification...");
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
                    println!("✅ Final {}", agent);
                }
                Err(e) => {
                    println!("❌ Failed to deserialize agent data: {:?}", e);
                    panic!("Failed to deserialize agent data: {:?}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_verify_signature_proof_success() {
        // 1. Setup environment
        let program_id = Pubkey::new_unique();
        let owner_account = Keypair::new();

        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "attps_solana",
            program_id,
            processor!(Processor::process_instruction),
        )
        .start()
        .await;

        let (contract_info_pubkey, _) =
            Pubkey::find_program_address(&[Constants::PREFIX_CONTRACT_INFO, b""], &program_id);

        // 2. Initialize the program
        println!("\nTesting program initialization...");
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

        // 3. Create and register an agent with the test signers
        println!("\nTesting agent creation and registration...");
        let agent_id: u128 = 0;
        let (agent_pubkey, _) = Pubkey::find_program_address(
            &[Constants::PREFIX_AGENT_ADDRESS, &agent_id.to_le_bytes()],
            &program_id,
        );

        // Use signers from test_verify_signature_success
        let signer1 = hex::decode("6370eF2f4Db3611D657b90667De398a2Cc2a370C").unwrap();
        let signer2 = hex::decode("677bb7270e0b03f0A2993A697654fb8Ecb6deE91").unwrap();
        let signers: Vec<[u8; 20]> = vec![
            signer1[..20].try_into().unwrap(),
            signer2[..20].try_into().unwrap(),
        ];

        let initial_settings = AgentSettings {
            signers,
            threshold: 1,
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

        // 4. Accept the agent
        println!("\nTesting agent acceptance...");
        let accept_instruction_data = vec![4];

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

        let account = banks_client
            .get_account(agent_pubkey)
            .await
            .expect("Failed to get agent account");

        if let Some(account_data) = account {
            let length = u32::from_le_bytes(account_data.data[..4].try_into().unwrap()) as usize;
            match AgentInfo::try_from_slice(&account_data.data[4..4 + length]) {
                Ok(agent) => {
                    println!("✅ Agent accepted successfully");
                    println!("✅ {}", agent);
                }
                Err(e) => {
                    println!("❌ Failed to deserialize agent data: {:?}", e);
                    panic!("Failed to deserialize agent data: {:?}", e);
                }
            }
        }

        // 5. Verify with signature proof
        println!("\nTesting signature verification...");

        // Use message and signature from test_verify_signature_success
        let message = "hello world!";
        let message_hash: [u8; 32] =
            hex::decode("57caa176af1ac0433c5df30e8dabcd2ec1af1e92a26eced5f719b88458777cd6")
                .unwrap()
                .try_into()
                .unwrap();

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

        let settings_digest = [
            1, 0, 47, 48, 92, 182, 101, 215, 149, 226, 246, 131, 240, 89, 34, 249, 86, 205, 175,
            40, 57, 221, 200, 246, 131, 18, 102, 80, 150, 206, 114, 208,
        ];
        let mut verify_instruction_data = vec![8]; // Verify instruction
        verify_instruction_data.extend_from_slice(&settings_digest); // settings_digest

        // Create MessagePayload with signature proof
        let payload = MessagePayload {
            data: message.as_bytes().to_vec(),
            data_hash: message_hash,
            proofs: Proofs {
                signature_proof: sig1,
                zk_proof: vec![],
                merkle_proof: vec![],
            },
            metadata: Metadata {
                content_type: "application/json".to_string(),
                encoding: "utf-8".to_string(),
                compression: "none".to_string(),
            },
        };
        payload.serialize(&mut verify_instruction_data).unwrap();

        let verify_instruction = Instruction::new_with_bytes(
            program_id,
            &verify_instruction_data,
            vec![AccountMeta::new(agent_pubkey, false)],
        );

        let mut transaction =
            Transaction::new_with_payer(&[verify_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        println!("✅ Signature verification successful");
    }

    #[tokio::test]
    async fn test_verify_signature_proof_error() {
        // 1. Setup environment
        let program_id = Pubkey::new_unique();
        let owner_account = Keypair::new();

        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "attps_solana",
            program_id,
            processor!(Processor::process_instruction),
        )
        .start()
        .await;

        let (contract_info_pubkey, _) =
            Pubkey::find_program_address(&[Constants::PREFIX_CONTRACT_INFO, b""], &program_id);

        // 2. Initialize the program
        println!("\nTesting program initialization...");
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

        // 3. Create and register an agent with the test signers
        println!("\nTesting agent creation and registration...");
        let agent_id: u128 = 0;
        let (agent_pubkey, _) = Pubkey::find_program_address(
            &[Constants::PREFIX_AGENT_ADDRESS, &agent_id.to_le_bytes()],
            &program_id,
        );

        // Use signers from test_verify_signature_success
        let signer1 = hex::decode("6370eF2f4Db3611D657b90667De398a2Cc2a370C").unwrap();
        let signer2 = hex::decode("677bb7270e0b03f0A2993A697654fb8Ecb6deE91").unwrap();
        let signers: Vec<[u8; 20]> = vec![
            signer1[..20].try_into().unwrap(),
            signer2[..20].try_into().unwrap(),
        ];

        let initial_settings = AgentSettings {
            signers,
            threshold: 1,
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

        // 4. Accept the agent
        println!("\nTesting agent acceptance...");
        let accept_instruction_data = vec![4];

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

        // 5. Verify with no proofs (should fail)
        println!("\nTesting signature verification with no proofs...");

        let message = "hello world!";
        let message_hash = keccak::hash(message.as_bytes()).to_bytes();
        let settings_digest = [
            1, 0, 47, 48, 92, 182, 101, 215, 149, 226, 246, 131, 240, 89, 34, 249, 86, 205, 175,
            40, 57, 221, 200, 246, 131, 18, 102, 80, 150, 206, 114, 208,
        ];

        let mut verify_instruction_data = vec![8]; // Verify instruction
        verify_instruction_data.extend_from_slice(&settings_digest); // settings_digest

        // Create MessagePayload with NO proofs (should trigger InvalidProofData error)
        let payload = MessagePayload {
            data: message.as_bytes().to_vec(),
            data_hash: message_hash,
            proofs: Proofs {
                signature_proof: vec![], // Empty signature proof
                zk_proof: vec![],        // Empty zk proof
                merkle_proof: vec![],    // Empty merkle proof
            },
            metadata: Metadata {
                content_type: "application/json".to_string(),
                encoding: "utf-8".to_string(),
                compression: "none".to_string(),
            },
        };
        payload.serialize(&mut verify_instruction_data).unwrap();

        let verify_instruction = Instruction::new_with_bytes(
            program_id,
            &verify_instruction_data,
            vec![AccountMeta::new(agent_pubkey, false)],
        );

        let mut transaction =
            Transaction::new_with_payer(&[verify_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);

        // The transaction should fail with InvalidProofData error
        if let TransactionError::InstructionError(_, err) = banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap()
        {
            assert_eq!(err, VerificationError::InvalidProofData.into());
            println!("✅ Verification failed as expected with InvalidProofData error");
        } else {
            panic!("Expected InvalidProofData error");
        }

        // 6. Test InvalidDataHash error case
        println!("\nTesting signature verification with invalid data hash...");

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

        let mut verify_instruction_data = vec![8]; // Verify instruction
        verify_instruction_data.extend_from_slice(&settings_digest); // settings_digest

        // Create MessagePayload with incorrect data_hash
        let payload = MessagePayload {
            data: message.as_bytes().to_vec(),
            data_hash: [0xFF; 32], // Incorrect hash that doesn't match the data
            proofs: Proofs {
                signature_proof: sig1.clone(),
                zk_proof: vec![],
                merkle_proof: vec![],
            },
            metadata: Metadata {
                content_type: "application/json".to_string(),
                encoding: "utf-8".to_string(),
                compression: "none".to_string(),
            },
        };
        payload.serialize(&mut verify_instruction_data).unwrap();

        let verify_instruction = Instruction::new_with_bytes(
            program_id,
            &verify_instruction_data,
            vec![AccountMeta::new(agent_pubkey, false)],
        );

        let mut transaction =
            Transaction::new_with_payer(&[verify_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);

        // The transaction should fail with InvalidDataHash error
        if let TransactionError::InstructionError(_, err) = banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap()
        {
            assert_eq!(err, VerificationError::InvalidDataHash.into());
            println!("✅ Verification failed as expected with InvalidDataHash error");
        } else {
            panic!("Expected InvalidDataHash error");
        }

        // 7. Test InvalidThreshold error case
        println!("\nTesting signature verification with insufficient signatures...");

        // Modify initial settings to require 2 signatures
        let mut agent_settings = initial_settings.clone();
        agent_settings.threshold = 2;

        // Update agent settings
        let change_settings_instruction_data = {
            let mut data = vec![5]; // ChangeAgentSettingProposal instruction
            agent_settings.serialize(&mut data).unwrap();
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

        // Accept the new settings
        let accept_settings_instruction_data = vec![6];

        let accept_settings_instruction = Instruction::new_with_bytes(
            program_id,
            &accept_settings_instruction_data,
            vec![
                AccountMeta::new(owner_account.pubkey(), true),
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new(agent_pubkey, false),
            ],
        );

        let mut transaction =
            Transaction::new_with_payer(&[accept_settings_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, &owner_account], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Try to verify with only one signature when threshold is 2
        let mut verify_instruction_data = vec![8]; // Verify instruction
        verify_instruction_data.extend_from_slice(&settings_digest); // settings_digest

        // Create MessagePayload with only one signature
        let payload = MessagePayload {
            data: message.as_bytes().to_vec(),
            data_hash: keccak::hash(message.as_bytes()).to_bytes(),
            proofs: Proofs {
                signature_proof: sig1.clone(), // Only one signature when threshold is 2
                zk_proof: vec![],
                merkle_proof: vec![],
            },
            metadata: Metadata {
                content_type: "application/json".to_string(),
                encoding: "utf-8".to_string(),
                compression: "none".to_string(),
            },
        };
        payload.serialize(&mut verify_instruction_data).unwrap();

        let verify_instruction = Instruction::new_with_bytes(
            program_id,
            &verify_instruction_data,
            vec![AccountMeta::new(agent_pubkey, false)],
        );

        let mut transaction =
            Transaction::new_with_payer(&[verify_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);

        // The transaction should fail with InvalidThreshold error
        if let TransactionError::InstructionError(_, err) = banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap()
        {
            assert_eq!(err, VerificationError::InvalidThreshold.into());
            println!("✅ Verification failed as expected with InvalidThreshold error");
        } else {
            panic!("Expected InvalidThreshold error");
        }

        // 8. Test DuplicateSigner error case
        println!("\nTesting signature verification with duplicate signatures...");

        // Reset threshold back to 1 for this test
        let mut agent_settings = initial_settings.clone();
        agent_settings.threshold = 1;

        // Update agent settings back
        let change_settings_instruction_data = {
            let mut data = vec![5]; // ChangeAgentSettingProposal instruction
            agent_settings.serialize(&mut data).unwrap();
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

        // Accept the settings change
        let accept_settings_instruction = Instruction::new_with_bytes(
            program_id,
            &accept_settings_instruction_data,
            vec![
                AccountMeta::new(owner_account.pubkey(), true),
                AccountMeta::new(contract_info_pubkey, false),
                AccountMeta::new(agent_pubkey, false),
            ],
        );

        let mut transaction =
            Transaction::new_with_payer(&[accept_settings_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer, &owner_account], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        // Create duplicate signature by repeating sig1
        let mut duplicate_sig = sig1.clone();
        duplicate_sig.extend_from_slice(&sig1);

        let verify_instruction_data = {
            let mut data = vec![8]; // Verify instruction
            data.extend_from_slice(&settings_digest); // settings_digest

            // Create MessagePayload with duplicate signatures
            let payload = MessagePayload {
                data: message.as_bytes().to_vec(),
                data_hash: keccak::hash(message.as_bytes()).to_bytes(),
                proofs: Proofs {
                    signature_proof: duplicate_sig, // Contains two identical signatures
                    zk_proof: vec![],
                    merkle_proof: vec![],
                },
                metadata: Metadata {
                    content_type: "application/json".to_string(),
                    encoding: "utf-8".to_string(),
                    compression: "none".to_string(),
                },
            };
            payload.serialize(&mut data).unwrap();
            data
        };

        let verify_instruction = Instruction::new_with_bytes(
            program_id,
            &verify_instruction_data,
            vec![AccountMeta::new(agent_pubkey, false)],
        );

        let mut transaction =
            Transaction::new_with_payer(&[verify_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);

        // The transaction should fail with DuplicateSigner error
        if let TransactionError::InstructionError(_, err) = banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap()
        {
            assert_eq!(err, VerificationError::DuplicateSigner.into());
            println!("✅ Verification failed as expected with DuplicateSigner error");
        } else {
            panic!("Expected DuplicateSigner error");
        }
    }
}
