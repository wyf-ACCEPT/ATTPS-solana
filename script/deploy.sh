#!/usr/bin/env bash

set -e

# Default values
NETWORK="devnet"
KEYPAIR_PATH="$HOME/.config/solana/id.json"

# Help message
print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Deploy ATTPS program to Solana network"
    echo ""
    echo "Options:"
    echo "  -n, --network <network>    Solana network to deploy to (devnet/testnet) [default: devnet]"
    echo "  -k, --keypair <path>       Path to deployer keypair [default: ~/.config/solana/id.json]"
    echo "  -h, --help                 Show this help message"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--network)
            NETWORK="$2"
            shift 2
            ;;
        -k|--keypair)
            KEYPAIR_PATH="$2"
            shift 2
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Validate network
if [[ "$NETWORK" != "devnet" && "$NETWORK" != "testnet" && "$NETWORK" != "mainnet" ]]; then
    echo "Error: Network must be 'devnet', 'testnet' or 'mainnet'"
    exit 1
fi

# Validate keypair file exists
if [[ ! -f "$KEYPAIR_PATH" ]]; then
    echo "Error: Keypair file not found at $KEYPAIR_PATH"
    exit 1
fi

echo "\nDeploying ATTPS program to Solana $NETWORK..."

# Configure Solana CLI
solana config set --url "$NETWORK"
solana config set --keypair "$KEYPAIR_PATH"

# Build the program
echo "\nBuilding program..."
cargo build-sbf

# Get the program keypair path
PROGRAM_KEYPAIR="target/deploy/attps_solana-keypair.json"

# Deploy the program
echo "\nDeploying program..."
solana program deploy \
    --program-id "$PROGRAM_KEYPAIR" \
    "target/deploy/attps_solana.so"

# Get program ID
PROGRAM_ID=$(solana-keygen pubkey "$PROGRAM_KEYPAIR")

echo "\nDeployment complete!"
echo "Program ID: $PROGRAM_ID"
echo "Network: $NETWORK"
echo ""
echo "You can view the program on Solana Explorer:"
echo "https://explorer.solana.com/address/$PROGRAM_ID?cluster=$NETWORK"
