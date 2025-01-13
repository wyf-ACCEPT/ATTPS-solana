// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Common} from "../libraries/Common.sol";

/// @notice A contract to handle access control of subscription management dependent on signing a AI Agent Manager
interface IAgentManager {
    /// @notice Get a list of all registering agents
    /// @return addresses - all registering addresses
    function getAllRegisteringAgents() external view returns (address[] memory);

    /// @notice Get details about the total number of registering Agents
    /// @return count - total number of registering agents in the system
    function getRegisteringAgentsCount() external view returns (uint64);

    /// @notice Retrieve a list of Registering agents using an inclusive range
    /// @dev WARNING: getRegisteringAgentsInRange uses EnumerableSet .length() and .at() methods to iterate over the list
    /// without the need for an extra mapping. These method can not guarantee the ordering when new elements are added.
    /// Evaluate if eventual consistency will satisfy your usecase before using it.
    /// @param registeringAgentIdxStart - index of the registering agents to start the range at
    /// @param registeringAgentIdxEnd - index of the registering agents to end the range at
    /// @return registeringAgents - registering addresses in the range provided
    function getRegisteringAgentsInRange(
        uint64 registeringAgentIdxStart,
        uint64 registeringAgentIdxEnd
    ) external view returns (address[] memory registeringAgents);

    /// @notice Get a list of all allowed agents
    /// @return addresses - all allowed addresses
    function getAllAllowedAgents() external view returns (address[] memory);

    /// @notice Get details about the total number of allowed Agents
    /// @return count - total number of allowed agents in the system
    function getAllowedAgentsCount() external view returns (uint64);

    /// @notice Retrieve a list of allowed agents using an inclusive range
    /// @dev WARNING: getAllowedAgentsInRange uses EnumerableSet .length() and .at() methods to iterate over the list
    /// without the need for an extra mapping. These method can not guarantee the ordering when new elements are added.
    /// Evaluate if eventual consistency will satisfy your usecase before using it.
    /// @param allowedAgentIdxStart - index of the allowed agents to start the range at
    /// @param allowedAgentIdxEnd - index of the allowed agents to end the range at
    /// @return allowedAgents - allowed addresses in the range provided
    function getAllowedAgentsInRange(
        uint64 allowedAgentIdxStart,
        uint64 allowedAgentIdxEnd
    ) external view returns (address[] memory allowedAgents);

    /// @notice Get a list of agent configs
    /// @return Common.AgentConfigs - all agent configs
    function getAgentConfigs(
        address agent
    ) external view returns (Common.AgentConfig[] memory);

    /// @notice Get details about the total number of agent config
    /// @return count - total number of agent config in the system
    function getAgentConfigsCount(address agent) external view returns (uint64);

    /// @notice Retrieve a list of agent configs using an inclusive range
    /// @param agentConfigIdxStart - index of the agent config to start the range at
    /// @param agentConfigIdxEnd - index of the agent config to end the range at
    /// @return agentConfigs - agent configs in the range provided
    function getAgentConfigsInRange(
        address agent,
        uint64 agentConfigIdxStart,
        uint64 agentConfigIdxEnd
    ) external view returns (Common.AgentConfig[] memory agentConfigs);

    /// @notice Detail of agent latest config
    /// @param agent - agent address
    /// @param settingDigest - agent setting digest
    /// @return agentConfig - agent detail
    function getAgentConfig(
        address agent,
        bytes32 settingDigest
    ) external view returns (Common.AgentConfig memory);

    /// @notice Determine if agent have permission
    /// @param agent - agent address
    /// @return bool
    function allowedAgent(address agent) external view returns (bool);

    /// @notice Determine if signer in
    /// @param agent - agent address
    /// @param settingDigest - agent setting digest
    /// @param signer - signer address
    /// @return bool
    function allowedSigner(
        address agent,
        bytes32 settingDigest,
        address signer
    ) external view returns (bool);

    /// @notice get agent threshold
    /// @param agent - agent address
    /// @param settingDigest - agent setting digest
    /// @return uint8
    function signerThreshold(
        address agent,
        bytes32 settingDigest
    ) external view returns (uint8);

    /// @notice The data to be validated is converted into a format allowed by the agent.
    /// @param agent - The recipient address that the acceptor is taking responsibility for
    /// @param data - The data to be converted.
    /// @return The converted data
    function validateDataConversion(
        address agent,
        bytes calldata data
    ) external view returns (bytes memory);

    /// @notice Register access to the agent based on acceptance of the AI Agent Manager
    /// @param agent The agent's address
    /// @param agentSettings The agent's settings
    function registerAgent(
        address agent,
        Common.AgentSettings memory agentSettings
    ) external;

    /// @notice Allows access to the agent based on acceptance of the AI agent manager
    /// @param agent - The recipient address that the acceptor is taking responsibility for
    function acceptAgent(address agent) external;

    /// @notice remove the Agent
    /// @param agent - Address of the agent to remove
    function removeAgent(address agent) external;

    /// @notice Change a settings update proposal for registered or allowed agents
    /// @param agent - Address of the agent
    /// @param agentSettings - new settings of the agent
    function changeAgentSettingProposal(
        address agent,
        Common.AgentSettings memory agentSettings
    ) external;

    /// @notice Review the settings update proposal initiated for registered or allowed agents
    /// @param agent - Address of the agent
    function acceptAgentSettingProposal(address agent) external;

    /// @notice get support agent version
    function agentVersion() external pure returns (string memory);

    /// @notice Determine whether the message id is valid
    function isValidMessageId(
        string memory messageId
    ) external pure returns (bool);

    /// @notice Determine whether the source agent id is valid
    function isValidSourceAgentId(
        string memory sourceAgentId
    ) external view returns (bool);

    /// @notice get Agent Proxy
    function agentProxy() external view returns (address);

    /// @notice set Agent Proxy
    /// @param proxy - Address of the proxy
    function setAgentProxy(address proxy) external;
}

// ================================================================
// |                     Configuration state                      |
// ================================================================
