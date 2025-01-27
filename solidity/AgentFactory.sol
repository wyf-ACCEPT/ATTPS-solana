// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {IAgentFactory} from "./interfaces/IAgentFactory.sol";
import {IAgentManager} from "./interfaces/IAgentManager.sol";
import {IAgentProxy} from "./interfaces/IAgentProxy.sol";
import {ITypeAndVersion} from "./interfaces/ITypeAndVersion.sol";
import {Common} from "./libraries/Common.sol";
import {Agent} from "./Agent.sol";
import {Create2} from "./vendor/openzeppelin-solidity/v4.8.3/contracts/utils/Create2.sol";
import {Address} from "./vendor/openzeppelin-solidity/v4.8.3/contracts/utils/Address.sol";
import {EnumerableSet} from "./vendor/openzeppelin-solidity/v4.8.3/contracts/utils/structs/EnumerableSet.sol";

// import "@openzeppelin/contracts/utils/Create2.sol";
// import "@openzeppelin/contracts/utils/Address.sol";
// import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

/*
 * The agent factory contract is used to deploy agent
 * by AI Agent.
 **/
contract AgentFactory is IAgentFactory, ITypeAndVersion {
    using Address for address;
    using EnumerableSet for EnumerableSet.AddressSet;

    event AgentCreated(
        address indexed owner,
        address indexed agent,
        address agentManager,
        uint256 agentId
    );

    error InvalidCallData();
    error AccessForbidden();

    IAgentManager private immutable i_agentManager;
    IAgentProxy private immutable i_agentProxy;
    EnumerableSet.AddressSet private s_agents;
    uint256 private s_counter;

    // ================================================================
    // |                       Initialization                         |
    // ================================================================
    constructor(address manager, address proxy) {
        i_agentManager = IAgentManager(manager);
        i_agentProxy = IAgentProxy(proxy);
    }

    /// @inheritdoc ITypeAndVersion
    function typeAndVersion() external pure override returns (string memory) {
        return "AI Agent Factory 1.0.0";
    }

    /// @inheritdoc IAgentFactory
    function createAgent() external override returns (address) {
        if (msg.sender != address(i_agentProxy)) revert AccessForbidden();
        bytes32 salt = keccak256(abi.encodePacked(s_counter, msg.sender));

        s_counter++;
        address agent = Create2.deploy(0, salt, _fullByteCode());
        s_agents.add(agent);

        emit AgentCreated(
            msg.sender,
            agent,
            address(i_agentManager),
            s_counter
        );

        return agent;
    }

    /// @inheritdoc IAgentFactory
    function getAllAgents() external view override returns (address[] memory) {
        return s_agents.values();
    }

    /// @inheritdoc IAgentFactory
    function getAgentsCount() external view override returns (uint64) {
        return uint64(s_agents.length());
    }

    /// @inheritdoc IAgentFactory
    function getAgentsInRange(
        uint64 agentIdxStart,
        uint64 agentIdxEnd
    ) external view override returns (address[] memory agents) {
        if (agentIdxStart > agentIdxEnd || agentIdxEnd >= s_agents.length()) {
            revert InvalidCallData();
        }

        agents = new address[]((agentIdxEnd - agentIdxStart) + 1);
        for (uint256 i = 0; i <= agentIdxEnd - agentIdxStart; ++i) {
            agents[i] = s_agents.at(uint256(agentIdxStart + i));
        }

        return agents;
    }

    /// @inheritdoc IAgentFactory
    function hasAgent(address agent) external view override returns (bool) {
        return s_agents.contains(agent);
    }

    /// @inheritdoc IAgentFactory
    function agentManager() external view override returns (address) {
        return address(i_agentManager);
    }

    /// @inheritdoc IAgentFactory
    function agentProxy() external view override returns (address) {
        return address(i_agentProxy);
    }

    // ================================================================
    // |                       Internal Methods                       |
    // ================================================================
    function _fullByteCode() internal view returns (bytes memory) {
        bytes memory bytecode = type(Agent).creationCode;
        bytes memory encodedParams = abi.encode(
            address(i_agentManager),
            address(i_agentProxy)
        );

        return abi.encodePacked(bytecode, encodedParams);
    }
}
