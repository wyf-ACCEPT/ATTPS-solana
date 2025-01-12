// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {EnumerableSet} from "../vendor/openzeppelin-solidity/v4.8.3/contracts/utils/structs/EnumerableSet.sol";
// import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

/*
 * @title Common
 * @notice Common functions and structs
 */
library Common {
    using EnumerableSet for EnumerableSet.AddressSet;

    enum Priority {
        High,
        Medium,
        Low
    }

    enum MessageType {
        Request,
        Response,
        Event
    }

    // @notice The asset struct to hold the address of an asset and amount
    struct Proofs {
        bytes zkProof;
        bytes merkleProof;
        bytes signatureProof;
    }

    // @notice The asset struct to hold the address of an asset and amount
    struct Metadata {
        string contentType;
        string encoding;
        string compression;
    }

    // @notice The asset struct to hold the address of an asset and amount
    struct MessagePayload {
        bytes data;
        bytes32 dataHash;
        Proofs proofs;
        Metadata metadata;
    }

    // @notice The asset struct to hold the agent config
    struct AgentHeader {
        string version;
        string messageId;
        string sourceAgentId;
        string sourceAgentName;
        string targetAgentId;
        uint256 timestamp;
        MessageType messageType; // request/response/event
        Priority priority; // high/medium/low
        uint256 ttl;
    }

    // @notice The asset struct to hold the agent signers and config
    struct AgentSettings {
        address[] signers;
        uint8 threshold;
        address converterAddress; // use converter for data process if not set 0
        AgentHeader agentHeader;
    }

    struct AgentConfig {
        bytes32 configDigest;
        uint32 configBlockNumber;
        bool isActive;
        AgentSettings settings;
    }

    struct AgentConfigState {
        /// The latest config digest set
        bytes32 latestConfigDigest;
        /// The historical record of all previously set configs by agent
        AgentConfig[] configs;
    }
}
