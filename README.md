# ATTPS (AgentText Transfer Protocol Secure) on Solana

ATTPS is a Solana program implementing a secure messaging and verification protocol for agent-based communication. It provides robust mechanisms for agent management, message verification, and secure communication on the Solana blockchain.

## Table of Contents
- [Overview](#overview)
- [Project Structure](#project-structure)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Building](#building)
- [Testing](#testing)
- [Development Workflow](#development-workflow)

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
   - Support for zero-knowledge proofs
   - Merkle proof verification
   - Multi-signature threshold validation

3. **Security Features**
   - UUID validation for message and agent IDs
   - PDA (Program Derived Address) account management
   - Owner-only administrative operations
   - Threshold-based multi-signature verification

## Project Structure

```
src/
├── lib.rs           # Module declarations and exports
├── entrypoint.rs    # Program entry point
├── state.rs         # Program state and account structures
├── error.rs         # Custom error definitions
├── processor.rs     # Instruction processing logic
├── instructions.rs  # Instruction definitions
├── constants.rs     # System constants
└── utils.rs         # Utility functions
```

### Account Structure
- **ContractInfo**: Stores contract-wide information (owner, agent counter)
- **AgentInfo**: Individual agent data and settings
- **AgentSettings**: Configuration for each agent
- **MessagePayload**: Structure for verified messages

## Prerequisites

1. **Required Software**
   - Rust toolchain (latest stable version)
   - Solana CLI tools (v1.18.26)

2. **Development Dependencies**
   - solana-program = "1.18.26"
   - borsh = "1.5.3"
   - hex = "0.4.3"
   - solana-program-test = "1.18.26"
   - solana-sdk = "1.18.26"
   - tokio = "1.43.0"

## Installation

1. Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Install Solana CLI tools:
```bash
sh -c "$(curl -sSfL https://release.solana.com/v1.18.26/install)"
```

3. Clone and setup the repository:
```bash
git clone <repository-url>
cd ATTPS-solana
```

## Building

Build the Solana program:
```bash
cargo build-sbf
```

## Testing

Run all tests:
```bash
# Run all tests with output
cargo test-sbf -- --nocapture

# Run specific test
cargo test-sbf test_name -- --nocapture
```

### Test Organization
Tests are located in `./src/test/` directory:
- `instruction_test.rs`: Instruction-related tests
- `state_test.rs`: State-related tests

## Development Workflow

1. **Branch Management**
   - Always pull from `develop` branch
   - Create PRs targeting the `develop` branch

2. **Code Formatting**
   ```bash
   # Format code before committing
   cargo fmt
   ```

3. **Commit Messages**
   Format: `[type] message`
   Types:
   - [fix]: Bug fixes
   - [build]: Build system changes
   - [feat]: New features
   - [init]: Initial commits
   - [test]: Test updates
   - [doc]: Documentation updates
   - [perf]: Performance improvements
   - [clean]: Code cleanup

4. **Best Practices**
   - Format code before committing
   - Follow existing test patterns
   - Use descriptive commit messages
   - Keep PRs focused and well-scoped


## Interaction Script Usage

The project includes a command-line tool for interacting with the deployed Solana program. You can use this tool to manage agents and verify messages.

### Available Commands

```bash
# Initialize the contract
cargo run --package attps_script -- initialize

# Create a new agent
cargo run --package attps_script -- create-agent

# Register an existing agent
cargo run --package attps_script -- register-agent -a <ID>
# or using long form
cargo run --package attps_script -- register-agent --agent-id <ID>

# Create and register an agent in one transaction
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
```

### Example Usage

```bash
# Create and register a new agent
cargo run --package attps_script -- create-and-register-agent

# Accept agent with ID 4
cargo run --package attps_script -- accept-agent --agent-id 4

# Verify a message from agent 4
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

## Deployment

Run `sh ./script/deploy.sh` to deploy the program to the devnet.
Run `sh ./script/deploy.sh --help` to see the help message.
