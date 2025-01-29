use attps_solana::state::{
    AgentHeader, AgentInfo, AgentSettings, ContractInfo, MessagePayload, MessageType, Metadata,
    Priority, Proofs,
};
use borsh::{BorshDeserialize, BorshSerialize};
use clap::{arg, Command};
use dotenv::dotenv;
use solana_client::rpc_client::RpcClient;
use solana_program::system_program;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::env;
use std::str::FromStr;

fn default_agent_settings() -> AgentSettings {
    let signer1 = hex::decode("6370eF2f4Db3611D657b90667De398a2Cc2a370C").unwrap();
    let signer2 = hex::decode("677bb7270e0b03f0A2993A697654fb8Ecb6deE91").unwrap();
    let signers: Vec<[u8; 20]> = vec![
        signer1[..20].try_into().unwrap(),
        signer2[..20].try_into().unwrap(),
    ];
    AgentSettings {
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
    }
}

fn default_agent_settings_another() -> AgentSettings {
    let mut agent_settings = default_agent_settings();
    agent_settings.agent_header.ttl = 3601;
    agent_settings
}

fn print_agent_info(client: &RpcClient, agent_pubkey: &Pubkey) {
    let agent_data = client.get_account_data(&agent_pubkey).unwrap();
    let length = u32::from_le_bytes(agent_data[0..4].try_into().unwrap()) as usize;
    let info: AgentInfo = AgentInfo::try_from_slice(&agent_data[4..4 + length])
        .expect("Failed to deserialize agent data");
    println!("✅ {}", info);
}

fn main() {
    // Connect to the Solana devnet
    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    // Load keypair from .env
    dotenv().ok();
    let owner_keypair_bytes =
        hex::decode(env::var("DEVNET_OWNER_KEYPAIR").expect("owner's private key must be set"))
            .expect("Failed to decode hex string");
    let owner_keypair = Keypair::from_bytes(&owner_keypair_bytes).expect("Invalid private key");
    let owner_pubkey = owner_keypair.pubkey();
    match client.get_balance(&owner_pubkey) {
        Ok(balance) => println!(
            "Owner account [{}] balance: {} lamports",
            owner_pubkey, balance
        ),
        Err(err) => eprintln!("Error getting balance: {}", err),
    }

    let payer_keypair_bytes =
        hex::decode(env::var("DEVNET_PAYER_KEYPAIR").expect("payer's private key must be set"))
            .expect("Failed to decode hex string");
    let payer_keypair = Keypair::from_bytes(&payer_keypair_bytes).expect("Invalid private key");
    let payer_pubkey = payer_keypair.pubkey();
    match client.get_balance(&payer_pubkey) {
        Ok(balance) => println!(
            "Payer account [{}] balance: {} lamports",
            payer_pubkey, balance
        ),
        Err(err) => eprintln!("Error getting balance: {}", err),
    }

    // Get the balance of the account
    let program_id_base58 = env::var("DEVNET_PROGRAM_ID").expect("program id must be set");
    let program_id = Pubkey::from_str(&program_id_base58).expect("Invalid program id");
    println!("Program id: {}\n", program_id);

    // Parse command line arguments first
    let matches = Command::new("ATTPS scripts for interacting with Solana")
        .version("1.0")
        .author("PlanD")
        .about("Interacts with the blockchain")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("view-agent")
                .about("View information about an agent")
                .arg(arg!(-a --"agent-id" <AGENT_ID> "Agent ID (u128)").required(true)),
        )
        .subcommand(Command::new("generate-keypair").about("Generates a new keypair"))
        .subcommand(Command::new("initialize").about("Initializes the contract"))
        .subcommand(Command::new("create-agent").about("Creates a new agent"))
        .subcommand(
            Command::new("register-agent")
                .about("Registers an existing agent data account")
                .arg(arg!(-a --"agent-id" <AGENT_ID> "Agent ID (u128)").required(true)),
        )
        .subcommand(
            Command::new("create-and-register-agent")
                .about("Creates and registers a new agent in one transaction"),
        )
        .subcommand(
            Command::new("accept-agent")
                .about("Accepts an agent")
                .arg(arg!(-a --"agent-id" <AGENT_ID> "Agent ID (u128)").required(true)),
        )
        .subcommand(
            Command::new("change-agent-setting-proposal")
                .about("Proposes new settings for an agent")
                .arg(arg!(-a --"agent-id" <AGENT_ID> "Agent ID (u128)").required(true)),
        )
        .subcommand(
            Command::new("accept-agent-setting-proposal")
                .about("Accepts proposed settings for an agent")
                .arg(arg!(-a --"agent-id" <AGENT_ID> "Agent ID (u128)").required(true)),
        )
        .subcommand(
            Command::new("remove-agent")
                .about("Removes an agent")
                .arg(arg!(-a --"agent-id" <AGENT_ID> "Agent ID (u128)").required(true)),
        )
        .subcommand(
            Command::new("verify")
                .about("Verifies a message from an agent")
                .arg(arg!(-a --"agent-id" <AGENT_ID> "Agent ID (u128)").required(true))
                .arg(
                    arg!(-d --"settings-digest" <DIGEST> "Settings digest as 32-byte hex string")
                        .required(true),
                ),
        )
        .get_matches();

    // Create instruction and call it
    let (contract_info_pubkey, _) = Pubkey::find_program_address(&[b"contract-info"], &program_id);
    println!("Contract info pubkey: {}\n", contract_info_pubkey);

    // Match command using the matches we already defined above
    match matches.subcommand() {
        Some(("view-agent", matches)) => {
            let agent_id: u128 = matches
                .get_one::<String>("agent-id")
                .expect("agent-id is required")
                .parse()
                .expect("agent-id must be a valid u128");
            let (agent_pubkey, _) =
                Pubkey::find_program_address(&[b"agent", &agent_id.to_le_bytes()], &program_id);
            println!("Agent pubkey (id: {}): {}\n", agent_id, agent_pubkey);
            print_agent_info(&client, &agent_pubkey);
        }
        Some(("generate-keypair", _)) => {
            let keypair = Keypair::new();
            println!("Keypair: {:?}", hex::encode(keypair.to_bytes()));
            println!("Pubkey: {:?}", keypair.pubkey());
            println!("Save them in `.env` file and request some devnet $SOL by `solana airdrop 1 <address>`.")
        }
        Some(("initialize", _)) => {
            let contract_info_data = client.get_account_data(&contract_info_pubkey).unwrap();
            match contract_info_data.len() {
                0 => {
                    let transaction = Transaction::new_signed_with_payer(
                        &[Instruction::new_with_bytes(
                            program_id,
                            &[0], // Initialize instruction
                            vec![
                                AccountMeta::new(payer_pubkey, true),
                                AccountMeta::new(owner_pubkey, false),
                                AccountMeta::new(contract_info_pubkey, false),
                                AccountMeta::new_readonly(system_program::id(), false),
                            ],
                        )],
                        Some(&payer_pubkey),
                        &[&payer_keypair],
                        client.get_latest_blockhash().unwrap(),
                    );
                    match client.send_and_confirm_transaction(&transaction) {
                        Ok(signature) => {
                            println!("🟩 Initialize transaction successful: {}\n", signature);
                        }
                        Err(err) => eprintln!("🟥 Transaction failed: {}\n", err),
                    };
                }
                _ => {
                    println!("Contract already initialized\n");
                }
            };

            let contract_info_data = client.get_account_data(&contract_info_pubkey).unwrap();
            let length = u32::from_le_bytes(contract_info_data[0..4].try_into().unwrap()) as usize;
            let info: ContractInfo =
                ContractInfo::try_from_slice(&contract_info_data[4..4 + length])
                    .expect("Failed to deserialize counter data");
            println!("Contract info data: {:?}\n", info);
        }
        Some(("create-agent", _)) => {
            let contract_info_data = client.get_account_data(&contract_info_pubkey).unwrap();
            let length = u32::from_le_bytes(contract_info_data[0..4].try_into().unwrap()) as usize;
            let info: ContractInfo =
                ContractInfo::try_from_slice(&contract_info_data[4..4 + length])
                    .expect("Failed to deserialize counter data");

            let agent_id: u128 = info.agent_counter;
            let (agent_pubkey, _) =
                Pubkey::find_program_address(&[b"agent", &agent_id.to_le_bytes()], &program_id);
            println!("Agent pubkey (id: {}): {}\n", agent_id, agent_pubkey);

            let transaction = Transaction::new_signed_with_payer(
                &[Instruction::new_with_bytes(
                    program_id,
                    &[1], // Create agent instruction
                    vec![
                        AccountMeta::new(payer_pubkey, true),
                        AccountMeta::new(contract_info_pubkey, false),
                        AccountMeta::new(agent_pubkey, false),
                        AccountMeta::new_readonly(system_program::id(), false),
                    ],
                )],
                Some(&payer_pubkey),
                &[&payer_keypair],
                client.get_latest_blockhash().unwrap(),
            );
            match client.send_and_confirm_transaction(&transaction) {
                Ok(signature) => {
                    println!("🟩 Create agent transaction successful: {}", signature);
                }
                Err(err) => eprintln!("🟥 Transaction failed: {}", err),
            }
            print_agent_info(&client, &agent_pubkey);
        }
        Some(("register-agent", matches)) => {
            let agent_id: u128 = matches
                .get_one::<String>("agent-id")
                .expect("agent-id is required")
                .parse()
                .expect("agent-id must be a valid u128");
            let agent_settings = default_agent_settings();

            let (agent_pubkey, _) =
                Pubkey::find_program_address(&[b"agent", &agent_id.to_le_bytes()], &program_id);
            println!("Agent pubkey (id: {}): {}\n", agent_id, agent_pubkey);

            let mut register_instruction_data = vec![2]; // 2 = RegisterAgent instruction
            agent_settings
                .serialize(&mut register_instruction_data)
                .unwrap();

            let transaction = Transaction::new_signed_with_payer(
                &[Instruction::new_with_bytes(
                    program_id,
                    &register_instruction_data,
                    vec![
                        AccountMeta::new(contract_info_pubkey, false),
                        AccountMeta::new(agent_pubkey, false),
                        AccountMeta::new_readonly(system_program::id(), false),
                    ],
                )],
                Some(&payer_pubkey),
                &[&payer_keypair],
                client.get_latest_blockhash().unwrap(),
            );
            match client.send_and_confirm_transaction(&transaction) {
                Ok(signature) => {
                    println!("🟩 Register agent transaction successful: {}", signature);
                }
                Err(err) => eprintln!("🟥 Transaction failed: {}", err),
            }
            print_agent_info(&client, &agent_pubkey);
        }
        Some(("create-and-register-agent", _)) => {
            let contract_info_data = client.get_account_data(&contract_info_pubkey).unwrap();
            let length = u32::from_le_bytes(contract_info_data[0..4].try_into().unwrap()) as usize;
            let info: ContractInfo =
                ContractInfo::try_from_slice(&contract_info_data[4..4 + length])
                    .expect("Failed to deserialize counter data");

            let agent_id: u128 = info.agent_counter;
            let (agent_pubkey, _) =
                Pubkey::find_program_address(&[b"agent", &agent_id.to_le_bytes()], &program_id);
            println!("Agent pubkey (id: {}): {}\n", agent_id, agent_pubkey);

            let agent_settings = default_agent_settings();
            let mut create_and_register_instruction_data = vec![3]; // 3 = CreateAndRegisterAgent instruction
            agent_settings
                .serialize(&mut create_and_register_instruction_data)
                .unwrap();

            let transaction = Transaction::new_signed_with_payer(
                &[Instruction::new_with_bytes(
                    program_id,
                    &create_and_register_instruction_data,
                    vec![
                        AccountMeta::new(payer_pubkey, true),
                        AccountMeta::new(contract_info_pubkey, false),
                        AccountMeta::new(agent_pubkey, false),
                        AccountMeta::new_readonly(system_program::id(), false),
                    ],
                )],
                Some(&payer_pubkey),
                &[&payer_keypair],
                client.get_latest_blockhash().unwrap(),
            );
            match client.send_and_confirm_transaction(&transaction) {
                Ok(signature) => {
                    println!(
                        "🟩 Create and register agent transaction successful: {}",
                        signature
                    );
                }
                Err(err) => eprintln!("🟥 Transaction failed: {}", err),
            }
            print_agent_info(&client, &agent_pubkey);
        }
        Some(("accept-agent", matches)) => {
            let agent_id: u128 = matches
                .get_one::<String>("agent-id")
                .expect("agent-id is required")
                .parse()
                .expect("agent-id must be a valid u128");

            let (agent_pubkey, _) =
                Pubkey::find_program_address(&[b"agent", &agent_id.to_le_bytes()], &program_id);
            println!("Agent pubkey (id: {}): {}\n", agent_id, agent_pubkey);

            let transaction = Transaction::new_signed_with_payer(
                &[Instruction::new_with_bytes(
                    program_id,
                    &[4], // Accept agent instruction
                    vec![
                        AccountMeta::new(owner_pubkey, true),
                        AccountMeta::new(contract_info_pubkey, false),
                        AccountMeta::new(agent_pubkey, false),
                        AccountMeta::new_readonly(system_program::id(), false),
                    ],
                )],
                Some(&owner_pubkey),
                &[&owner_keypair],
                client.get_latest_blockhash().unwrap(),
            );
            match client.send_and_confirm_transaction(&transaction) {
                Ok(signature) => {
                    println!("🟩 Accept agent transaction successful: {}", signature);
                }
                Err(err) => eprintln!("🟥 Transaction failed: {}", err),
            }
            print_agent_info(&client, &agent_pubkey);
        }
        Some(("change-agent-setting-proposal", matches)) => {
            let agent_id: u128 = matches
                .get_one::<String>("agent-id")
                .expect("agent-id is required")
                .parse()
                .expect("agent-id must be a valid u128");
            let proposed_settings = default_agent_settings_another();

            let (agent_pubkey, _) =
                Pubkey::find_program_address(&[b"agent", &agent_id.to_le_bytes()], &program_id);
            println!("Agent pubkey (id: {}): {}\n", agent_id, agent_pubkey);

            let mut instruction_data = vec![5]; // Change agent setting proposal instruction
            proposed_settings.serialize(&mut instruction_data).unwrap();

            let transaction = Transaction::new_signed_with_payer(
                &[Instruction::new_with_bytes(
                    program_id,
                    &instruction_data,
                    vec![
                        AccountMeta::new(agent_pubkey, false),
                        AccountMeta::new_readonly(system_program::id(), false),
                    ],
                )],
                Some(&payer_pubkey),
                &[&payer_keypair],
                client.get_latest_blockhash().unwrap(),
            );
            match client.send_and_confirm_transaction(&transaction) {
                Ok(signature) => {
                    println!(
                        "🟩 Change agent setting proposal transaction successful: {}",
                        signature
                    );
                }
                Err(err) => eprintln!("🟥 Transaction failed: {}", err),
            }
            print_agent_info(&client, &agent_pubkey);
        }
        Some(("accept-agent-setting-proposal", matches)) => {
            let agent_id: u128 = matches
                .get_one::<String>("agent-id")
                .expect("agent-id is required")
                .parse()
                .expect("agent-id must be a valid u128");

            let (agent_pubkey, _) =
                Pubkey::find_program_address(&[b"agent", &agent_id.to_le_bytes()], &program_id);
            println!("Agent pubkey (id: {}): {}\n", agent_id, agent_pubkey);

            let default_settings = AgentSettings::default();
            let mut instruction_data = vec![6]; // Accept agent setting proposal instruction
            instruction_data.extend(borsh::to_vec(&default_settings).unwrap());

            let transaction = Transaction::new_signed_with_payer(
                &[Instruction::new_with_bytes(
                    program_id,
                    &instruction_data,
                    vec![
                        AccountMeta::new(owner_pubkey, true),
                        AccountMeta::new(contract_info_pubkey, false),
                        AccountMeta::new(agent_pubkey, false),
                        AccountMeta::new_readonly(system_program::id(), false),
                    ],
                )],
                Some(&owner_pubkey),
                &[&owner_keypair],
                client.get_latest_blockhash().unwrap(),
            );
            match client.send_and_confirm_transaction(&transaction) {
                Ok(signature) => {
                    println!(
                        "🟩 Accept agent setting proposal transaction successful: {}",
                        signature
                    );
                }
                Err(err) => eprintln!("🟥 Transaction failed: {}", err),
            }

            let agent_data = client.get_account_data(&agent_pubkey).unwrap();
            let length = u32::from_le_bytes(agent_data[0..4].try_into().unwrap()) as usize;
            let info: AgentInfo = AgentInfo::try_from_slice(&agent_data[4..4 + length])
                .expect("Failed to deserialize agent data");
            println!("✅ {}", info);
        }
        Some(("remove-agent", matches)) => {
            let agent_id: u128 = matches
                .get_one::<String>("agent-id")
                .expect("agent-id is required")
                .parse()
                .expect("agent-id must be a valid u128");

            let (agent_pubkey, _) =
                Pubkey::find_program_address(&[b"agent", &agent_id.to_le_bytes()], &program_id);
            println!("Agent pubkey (id: {}): {}\n", agent_id, agent_pubkey);

            let transaction = Transaction::new_signed_with_payer(
                &[Instruction::new_with_bytes(
                    program_id,
                    &[7], // Remove agent instruction
                    vec![
                        AccountMeta::new(owner_pubkey, true),
                        AccountMeta::new(contract_info_pubkey, false),
                        AccountMeta::new(agent_pubkey, false),
                        AccountMeta::new_readonly(system_program::id(), false),
                    ],
                )],
                Some(&owner_pubkey),
                &[&owner_keypair],
                client.get_latest_blockhash().unwrap(),
            );
            match client.send_and_confirm_transaction(&transaction) {
                Ok(signature) => {
                    println!("🟩 Remove agent transaction successful: {}", signature);
                }
                Err(err) => eprintln!("🟥 Transaction failed: {}", err),
            }
            print_agent_info(&client, &agent_pubkey);
        }
        Some(("verify", matches)) => {
            let agent_id: u128 = matches
                .get_one::<String>("agent-id")
                .expect("agent-id is required")
                .parse()
                .expect("agent-id must be a valid u128");

            let settings_digest_hex = matches
                .get_one::<String>("settings-digest")
                .expect("settings-digest is required");
            let settings_digest = hex::decode(settings_digest_hex)
                .expect("settings_digest must be a valid hex string of 32 bytes");
            if settings_digest.len() != 32 {
                panic!("settings_digest must be exactly 32 bytes");
            }

            let (agent_pubkey, _) =
                Pubkey::find_program_address(&[b"agent", &agent_id.to_le_bytes()], &program_id);
            println!("Agent pubkey (id: {}): {}\n", agent_id, agent_pubkey);

            let message = "hello world!";
            let message_hash: [u8; 32] =
                hex::decode("57caa176af1ac0433c5df30e8dabcd2ec1af1e92a26eced5f719b88458777cd6")
                    .unwrap()
                    .try_into()
                    .unwrap();
            let mut signature = Vec::new();
            signature.extend_from_slice(
                &hex::decode("26eebcfa4a0f21ed6e03722eebba46377a6d394686d83cb47be28fd1bf6b984a")
                    .unwrap(),
            ); // r
            signature.extend_from_slice(
                &hex::decode("119870fe1cc35ddfae19900ce8afcd88531bb000e8bc4c82dcd5cab0fe7db54d")
                    .unwrap(),
            ); // s
            signature.push(1); // yParity (recovery_id)
            let payload = MessagePayload {
                data: message.as_bytes().to_vec(),
                data_hash: message_hash,
                proofs: Proofs {
                    signature_proof: signature,
                    zk_proof: vec![],
                    merkle_proof: vec![],
                },
                metadata: Metadata {
                    content_type: "application/json".to_string(),
                    encoding: "utf-8".to_string(),
                    compression: "none".to_string(),
                },
            };

            let mut verify_instruction_data = vec![8]; // Verify instruction
            verify_instruction_data.extend(&settings_digest);
            payload.serialize(&mut verify_instruction_data).unwrap();

            let transaction = Transaction::new_signed_with_payer(
                &[Instruction::new_with_bytes(
                    program_id,
                    &verify_instruction_data,
                    vec![
                        AccountMeta::new(agent_pubkey, false),
                        AccountMeta::new_readonly(system_program::id(), false),
                    ],
                )],
                Some(&owner_pubkey),
                &[&owner_keypair],
                client.get_latest_blockhash().unwrap(),
            );
            match client.send_and_confirm_transaction(&transaction) {
                Ok(signature) => {
                    println!("🟩 Verify transaction successful: {}", signature);
                }
                Err(err) => eprintln!("🟥 Transaction failed: {}", err),
            }
        }
        _ => {
            eprintln!("Invalid subcommand");
            unreachable!()
        }
    };
}
