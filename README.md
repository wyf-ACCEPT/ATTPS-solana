# ATTPS (AgentText Transfer Protocol Secure) on Solana

ATTPS is a Solana program implementing a secure messaging and verification protocol for agent-based communication. It provides robust mechanisms for agent management, message verification, and secure communication on the Solana blockchain.

## Table of Contents
- [Overview](#overview)
- [Project Structure](#project-structure)
- [Build and Test](#build-and-test)
- [Deploy and Interact](#deploy-and-interact)

<br>

## Overview

ATTPS implements a secure messaging protocol with the following key features:
- Multi-signature support with threshold verification
- Ethereum-compatible address handling
- Upgradeable agent settings
- Owner-controlled agent management
- Multiple proof types for message verification (signatures, ZK proofs, Merkle proofs)

### Core Components

1. **Agent Management**
   - Agent creation and registration
   - Settings management
   - Lifecycle control (registration, acceptance, removal)

2. **Message Verification**
   - Signature verification using secp256k1
   - Multi-signature threshold validation
   - Support for zero-knowledge proofs (coming soon)
   - Merkle proof verification (coming soon)

3. **Security Features**
   - UUID validation for message and agent IDs
   - PDA (Program Derived Address) account management
   - Owner-only administrative operations
   - Threshold-based multi-signature verification

<br>

## Project Structure

```log
├── Cargo.lock           # Dependencies lock file
├── Cargo.toml           # Project(main-package) dependencies and configuration
├── README.md            # Project documentation
├── script
│   ├── Cargo.toml       # Script(sub-package) dependencies and configuration
│   ├── deploy.sh        # Script for deploying the program
│   └── interact.rs      # Script for interacting with the program
└── src
    ├── constants.rs     # System constants
    ├── entrypoint.rs    # Program entry point
    ├── error.rs         # Custom error definitions
    ├── instruction.rs   # Instruction definitions
    ├── lib.rs           # Module declarations and exports
    ├── processor.rs     # Instruction processing logic
    ├── state.rs         # Program state and account structures
    ├── test
    │   ├── instruction_test.rs     # Test for instructions
    │   ├── state_test.rs           # Test for data structures
    │   ├── utils_agent_test.rs     # Test for agent utilities functions
    │   └── utils_manager_test.rs   # Test for manager utilities functions
    └── utils.rs         # Utility functions
```

### Account Structure
- **ContractInfo**: Stores contract-wide information (owner, agent counter)
- **AgentInfo**: Individual agent data and settings
- **AgentSettings**: Configuration for each agent
- **MessagePayload**: Structure for verified messages

<br>

## Build and Test

### Prerequisites

1. Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Install Solana CLI tools:
```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```

3. Clone and setup the repository:
```bash
git clone <repository-url>
cd ATTPS-solana
```

### Building

Build the Solana program:
```bash
# Build the program, using the Solana BPF toolchain
cargo build-sbf

# If failed, try to build directly
cargo build
```

### Testing

Run all tests:
```bash
# Run all tests with output, using the Solana BPF toolchain
cargo test-sbf -- --nocapture

# Run specific test, using the Solana BPF toolchain
cargo test-sbf test_name -- --nocapture

# If failed, try to run directly
cargo test -- --nocapture
```

<br>

## Deploy and Interact

After building and testing the program, you should generate a keypair using Solana CLI by running `solana-keygen new`, then deploy the program to the Solana blockchain. You can use the [deploy script](./script/deploy.sh) tool. The usage is as follows:

```bash
# Deploy the program to the devnet
sh ./script/deploy.sh

# Deploy the program to the mainnet
sh ./script/deploy.sh --network mainnet

# Deploy the program to the devnet, use a specific keypair
sh ./script/deploy.sh --keypair ~/.config/solana/id.json

# Print help message
sh ./script/deploy.sh --help
```

If you don't have enough $SOL to deploy the program on devnet, you can use this command to get some SOL:
```bash
solana airdrop 3 <your-address>
```

Remember to set-up the `.env` file by running `cp .env.example .env` first, and then paste the deployed program address to `.env` file. 

### Available Commands

The project includes a command-line tool for interacting with the deployed Solana program. You can use this tool to manage agents and verify messages.

```bash
# Generate a new keypair (paste the result to `.env` file)
cargo run --package attps_script -- generate-keypair

# Initialize the contract (only once)
cargo run --package attps_script -- initialize

# Create a new agent (will get an `agent_id` as the result)
cargo run --package attps_script -- create-agent

# Register an existing agent
cargo run --package attps_script -- register-agent -a <ID>
# or using long form
cargo run --package attps_script -- register-agent --agent-id <ID>

# Create and register an agent in one transaction (will get an `agent_id` as the result)
cargo run --package attps_script -- create-and-register-agent

# Accept an agent
cargo run --package attps_script -- accept-agent -a <ID>
# or using long form
cargo run --package attps_script -- accept-agent --agent-id <ID>

# Propose new settings for an agent
cargo run --package attps_script -- change-agent-setting-proposal -a <ID>
# or using long form
cargo run --package attps_script -- change-agent-setting-proposal --agent-id <ID>

# Accept proposed settings for an agent
cargo run --package attps_script -- accept-agent-setting-proposal -a <ID>
# or using long form
cargo run --package attps_script -- accept-agent-setting-proposal --agent-id <ID>

# Remove an agent
cargo run --package attps_script -- remove-agent -a <ID>
# or using long form
cargo run --package attps_script -- remove-agent --agent-id <ID>

# Verify a message from an agent
cargo run --package attps_script -- verify -a <ID> -d <32_BYTE_HEX>
# or using long form
cargo run --package attps_script -- verify --agent-id <ID> --settings-digest <32_BYTE_HEX>

# View agent information
cargo run --package attps_script -- view-agent -a <ID>
# or using long form
cargo run --package attps_script -- view-agent --agent-id <ID>
```

### Example Usage

```bash
# Create and register a new agent (and returned `agent_id` is 4)
cargo run --package attps_script -- create-and-register-agent

# Accept agent with ID 4
cargo run --package attps_script -- accept-agent --agent-id 4

# Verify a message from agent 4 (the `digest` is currently not used, so just put any value)
cargo run --package attps_script -- verify --agent-id 4 --settings-digest 0100231df1ab9e7cbdea3018c65ddced9598e0a13942cb4480d3798da83dfd2f
```

### Help and Version Information

```bash
# Show help information
cargo run --package attps_script -- --help
# or
cargo run --package attps_script -- -h

# Show version information
cargo run --package attps_script -- --version
# or
cargo run --package attps_script -- -V
```
