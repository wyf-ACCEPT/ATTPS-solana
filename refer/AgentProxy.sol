// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {ConfirmedOwner} from "./access/ConfirmedOwner.sol";
import {Common} from "./libraries/Common.sol";
import {IAgent} from "./interfaces/IAgent.sol";
import {IAgentFactory} from "./interfaces/IAgentFactory.sol";
import {IAgentManager} from "./interfaces/IAgentManager.sol";
import {IAgentProxy} from "./interfaces/IAgentProxy.sol";
import {ITypeAndVersion} from "./interfaces/ITypeAndVersion.sol";
import {Common} from "./libraries/Common.sol";

/*
 * The Proxy contract enables tighter integration of various functionalities.
 **/
contract AgentProxy is ConfirmedOwner, IAgentProxy, ITypeAndVersion {
    event AgentManagerSet(address oldManager, address newManager);
    event AgentFactorySet(address oldFactory, address newFactory);

    error InvalidAgentFactoryOrManager();

    IAgentManager private i_agentManager;
    IAgentFactory private i_agentFactory;

    // ================================================================
    // |                       Initialization                         |
    // ================================================================
    constructor() ConfirmedOwner(msg.sender) {}

    /// @inheritdoc ITypeAndVersion
    function typeAndVersion() external pure override returns (string memory) {
        return "AI Agent Proxy 1.0.0";
    }

    /// @inheritdoc IAgentProxy
    function agentFactory() external view override returns (address) {
        return address(i_agentFactory);
    }

    /// @inheritdoc IAgentProxy
    function agentManager() external view override returns (address) {
        return address(i_agentManager);
    }

    /// @inheritdoc IAgentProxy
    function createAndRegisterAgent(
        Common.AgentSettings memory agentSettings
    ) external override {
        IAgentManager manager = i_agentManager;
        IAgentFactory factory = i_agentFactory;

        if (address(manager) != address(0) && address(factory) != address(0)) {
            address agent = factory.createAgent();
            return manager.registerAgent(agent, agentSettings);
        } else {
            revert InvalidAgentFactoryOrManager();
        }
    }

    /// @inheritdoc IAgentProxy
    function verify(
        address agent,
        bytes32 settingsDigest,
        Common.MessagePayload memory payload
    ) external override {
        return IAgent(agent).verify(settingsDigest, payload);
    }

    /// @inheritdoc IAgentProxy
    function setAgentManager(address manager) external override onlyOwner {
        if (manager == address(0) || manager == address(i_agentManager))
            revert InvalidAgentFactoryOrManager();

        address old = address(i_agentManager);
        i_agentManager = IAgentManager(manager);

        emit AgentManagerSet(old, manager);
    }

    /// @inheritdoc IAgentProxy
    function setAgentFactory(address factory) external override onlyOwner {
        if (factory == address(0) || factory == address(i_agentFactory))
            revert InvalidAgentFactoryOrManager();

        address old = address(i_agentFactory);
        i_agentFactory = IAgentFactory(factory);

        emit AgentFactorySet(old, factory);
    }
}
