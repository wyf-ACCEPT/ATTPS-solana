// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

interface IConverter {
    /**
     * @notice converter use to conver the business data to agent data to be signed
     * correctly by routing to the correct verifier.
     * @param data The data to be verified.
     */
    function converter(
        bytes calldata data
    ) external pure returns (bytes memory);
}
