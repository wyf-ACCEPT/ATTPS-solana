// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {Common} from "../libraries/Common.sol";

interface IAgentFactory {
    /**
     * @notice create the agent by factory
     * @return agent address
     */
    function createAgent() external returns (address);

    /**
     * @notice Get a list of all agents
     * @return addresses - all addresses
     */
    function getAllAgents() external view returns (address[] memory);

    /**
     * @notice Get details about the total number of registering Agents
     * @return count - total number of agents in the system
     */
    function getAgentsCount() external view returns (uint64);

    /**
     * @notice Retrieve a list of agents using an inclusive range
     * @param agentIdxStart - index of the registering agents to start the range at
     * @param agentIdxEnd - index of the registering agents to end the range at
     * @return agents - addresses in the range provided
     */
    function getAgentsInRange(
        uint64 agentIdxStart,
        uint64 agentIdxEnd
    ) external view returns (address[] memory agents);

    /**
     * @notice Does the agent exist
     * @param agent - address of agent
     * @return bool
     */
    function hasAgent(address agent) external view returns (bool);

    /// @notice get Agent Manager
    function agentManager() external view returns (address);

    /// @notice get Agent Proxy
    function agentProxy() external view returns (address);
}
