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

