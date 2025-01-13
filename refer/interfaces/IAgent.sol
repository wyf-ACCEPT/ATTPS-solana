// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {Common} from "../libraries/Common.sol";

interface IAgent {
    /**
     * @notice the agent manager address
     * @return agent manager address
     */
    function agentManager() external view returns (address);

    /**
     * @notice the agent proxy address
     * @return agent proxy address
     */
    function agentProxy() external view returns (address);

    /**
     * @notice Agent that the data encoded has been signed
     * correctly by routing to the correct agent.
     * @param settingsDigest The agent setting digest
     * @param payload The struct data to be verified.
     * @dev Verification is typically only done through the proxy contract so
     * we can't just use msg.sender to log the requester as the msg.sender
     * contract will always be the proxy.
     */
    function verify(
        bytes32 settingsDigest,
        Common.MessagePayload calldata payload
    ) external;
}
