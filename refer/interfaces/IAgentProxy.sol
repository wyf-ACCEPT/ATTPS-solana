// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {Common} from "../libraries/Common.sol";

interface IAgentProxy {
    function agentFactory() external view returns (address);

    function agentManager() external view returns (address);

    function createAndRegisterAgent(
        Common.AgentSettings memory agentSettings
    ) external;

    function verify(
        address agent,
        bytes32 settingsDigest,
        Common.MessagePayload memory payload
    ) external;

    function setAgentManager(address manager) external;

    function setAgentFactory(address factory) external;
}
