use attps_solana::state::{ContractInfo, AgentInfo};
use borsh::BorshDeserialize;
use clap::Command;
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
        Ok(balance) => println!("Account [{}] balance: {} lamports", owner_pubkey, balance),
        Err(err) => eprintln!("Error getting balance: {}", err),
    }

    // Get the balance of the account
    let program_id_base58 = env::var("DEVNET_PROGRAM_ID").expect("program id must be set");
    let program_id = Pubkey::from_str(&program_id_base58).expect("Invalid program id");
    println!("Program id: {}\n", program_id);

    // Match command
    let matches = Command::new("ATTPS scripts for interacting with Solana")
        .version("1.0")
        .author("PlanD")
        .about("Interacts with the blockchain")
        .subcommand(Command::new("initialize").about("Initializes the contract"))
        .subcommand(Command::new("create_agent").about("Creates a new agent"))
        .get_matches();

    // Create instruction and call it
    let (contract_info_pubkey, _) = Pubkey::find_program_address(&[b"contract-info"], &program_id);
    println!("Contract info pubkey: {}\n", contract_info_pubkey);

    match matches.subcommand() {
        Some(("initialize", _)) => {
            let contract_info_data = client.get_account_data(&contract_info_pubkey).unwrap();
            match contract_info_data.len() {
                0 => {
                    let transaction = Transaction::new_signed_with_payer(
                        &[Instruction::new_with_bytes(
                            program_id,
                            &[0], // Initialize instruction
                            vec![
                                AccountMeta::new(owner_pubkey, true),
                                AccountMeta::new(owner_pubkey, false),
                                AccountMeta::new(contract_info_pubkey, false),
                                AccountMeta::new_readonly(system_program::id(), false),
                            ],
                        )],
                        Some(&owner_pubkey),
                        &[&owner_keypair],
                        client.get_latest_blockhash().unwrap(),
                    );
                    match client.send_and_confirm_transaction(&transaction) {
                        Ok(signature) => {
                            println!("Initialize transaction successful: {}\n", signature);
                        }
                        Err(err) => eprintln!("Transaction failed: {}\n", err),
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
        Some(("create_agent", _)) => {
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
                    println!("Create agent transaction successful: {}", signature);
                }
                Err(err) => eprintln!("Transaction failed: {}", err),
            }

            let agent_data = client.get_account_data(&agent_pubkey).unwrap();
            let length = u32::from_le_bytes(agent_data[0..4].try_into().unwrap()) as usize;
            let info: AgentInfo = AgentInfo::try_from_slice(&agent_data[4..4 + length])
                .expect("Failed to deserialize agent data");
            println!("✅ {}", info);
        }
        _ => {
            eprintln!("Invalid subcommand");
            unreachable!()
        }
    };
}
