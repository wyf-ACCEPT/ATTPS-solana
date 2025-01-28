use dotenv::dotenv;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use std::env;

fn main() {
    // Connect to the Solana devnet
    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    // Load keypair from .env
    dotenv().ok();
    let keypair_hex = env::var("DEVNET_OWNER_KEYPAIR").expect("owner's private key must be set");
    let keypair_bytes = hex::decode(keypair_hex).expect("Failed to decode hex string");
    let keypair = Keypair::from_bytes(&keypair_bytes).expect("Invalid private key");
    let pubkey = keypair.pubkey();
    println!("Public key: {}", pubkey);

    // Get the balance of the account
    match client.get_balance(&pubkey) {
        Ok(balance) => println!("Account balance: {} lamports", balance),
        Err(err) => eprintln!("Error getting balance: {}", err),
    }

    // Create a simple transfer transaction
    let to_pubkey = Pubkey::new_unique();
    let lamports = 100_000; // 0.0001 SOL

    let transaction = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(&pubkey, &to_pubkey, lamports)],
        Some(&pubkey),
        &[&keypair],
        client.get_latest_blockhash().unwrap(),
    );

    match client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => println!("Transaction successful: {}", signature),
        Err(err) => eprintln!("Transaction failed: {}", err),
    }
}
