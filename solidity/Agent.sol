// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import {Common} from "./libraries/Common.sol";
import {IAgent} from "./interfaces/IAgent.sol";
import {IAgentManager} from "./interfaces/IAgentManager.sol";
import {IAgentProxy} from "./interfaces/IAgentProxy.sol";
import {ITypeAndVersion} from "./interfaces/ITypeAndVersion.sol";
import {Common} from "./libraries/Common.sol";

/*
 * The verifier contract is used to verify offchain reports signed
 * by AI Agent.
 **/
contract Agent is IAgent, ITypeAndVersion {
    event MessageVerified(
        bytes32 indexed agentSettingsDigest,
        bytes32 indexed dataHash,
        bytes data,
        Common.Proofs proofs,
        Common.Metadata metadata
    );

    /// @notice This error is thrown whenever an address tries
    /// to exeecute a transaction that it is not authorized to do so
    error AccessForbidden();

    /// @notice This error is thrown whenever a zero address is passed
    error ZeroAddress();

    /// @notice This error is thrown whenever a invalid data hash is passed
    error InvalidDataHash();

    /// @notice This error is thrown whenever a report fails to verify due to bad signature
    error BadVerification();

    /// @notice This error is thrown whenever a report fails to verify due to proof method
    error UnsupportedProofMethod();

    /// @notice This error is thrown whenever a report fails to load proof data
    error InvalidProofData();

    /// @notice This error is thrown whenever an invalid allowed agent
    error InvalidAllowedAgent();

    IAgentManager private immutable i_agentManager;
    IAgentProxy private immutable i_agentProxy;

    // ================================================================
    // |                       Initialization                         |
    // ================================================================
    constructor(address manager, address proxy) {
        i_agentManager = IAgentManager(manager);
        i_agentProxy = IAgentProxy(proxy);
    }

    /// @inheritdoc ITypeAndVersion
    function typeAndVersion() external pure override returns (string memory) {
        return "AI Agent 1.0.0";
    }

    /// @inheritdoc IAgent
    function agentManager() external view override returns (address) {
        return address(i_agentManager);
    }

    /// @inheritdoc IAgent
    function agentProxy() external view override returns (address) {
        return address(i_agentProxy);
    }

    /// @inheritdoc IAgent
    function verify(
        bytes32 settingsDigest,
        Common.MessagePayload memory payload
    ) external override {
        if (msg.sender != address(i_agentProxy)) revert AccessForbidden();

        bytes32 hash = keccak256(
            i_agentManager.validateDataConversion(address(this), payload.data)
        );

        if (hash != payload.dataHash) revert InvalidDataHash();

        if (
            payload.proofs.signatureProof.length == 0 &&
            payload.proofs.zkProof.length == 0 &&
            payload.proofs.merkleProof.length == 0
        ) revert InvalidProofData();

        if (payload.proofs.signatureProof.length != 0) {
            bytes memory proof = payload.proofs.signatureProof;
            _verifySignature(settingsDigest, hash, proof);
        }
        if (payload.proofs.zkProof.length != 0) {
            bytes memory proof = payload.proofs.zkProof;
            _verifyZk(settingsDigest, hash, proof);
        }
        if (payload.proofs.merkleProof.length != 0) {
            bytes memory proof = payload.proofs.merkleProof;
            _verifyMerkle(settingsDigest, hash, proof);
        }

        emit MessageVerified(
            settingsDigest,
            payload.dataHash,
            payload.data,
            payload.proofs,
            payload.metadata
        );
    }

    // ================================================================
    // |                       Internal Methods                       |
    // ================================================================

    /// @notice Verifies that a report has been signed by the correct
    /// @param settingsDigest The agent setting digest
    /// @param hash The keccak256 hash of the raw report's bytes
    /// @param signatureProof is the signature on report.
    function _verifySignature(
        bytes32 settingsDigest,
        bytes32 hash,
        bytes memory signatureProof
    ) private view {
        uint256 signedCount;
        address[] memory signers;

        if (false == i_agentManager.allowedAgent(address(this)))
            revert InvalidAllowedAgent();
        (bytes32[] memory rs, bytes32[] memory ss, uint8[] memory rawV) = abi
            .decode(signatureProof, (bytes32[], bytes32[], uint8[]));

        for (uint256 i; i < rs.length; ++i) {
            address recoverSigner = ecrecover(hash, rawV[i] + 27, rs[i], ss[i]);
            if (
                true ==
                i_agentManager.allowedSigner(
                    address(this),
                    settingsDigest,
                    recoverSigner
                )
            ) {
                if (_addressExists(signers, recoverSigner)) {
                    revert BadVerification();
                } else {
                    signers = _addressAdd(signers, recoverSigner);
                    signedCount += 1;
                }
            }
        }

        if (
            signedCount <
            i_agentManager.signerThreshold(address(this), settingsDigest)
        ) revert BadVerification();
    }

    /// @notice Verifies that a report has been signed by the correct
    /// @param settingsDigest The agent setting digest
    /// @param hash The keccak256 hash of the raw report's bytes
    /// @param zkProof is the signature on report.
    function _verifyZk(
        bytes32 settingsDigest,
        bytes32 hash,
        bytes memory zkProof
    ) private pure {
        settingsDigest = settingsDigest;
        hash = hash;
        zkProof = zkProof;

        revert UnsupportedProofMethod();
    }

    /// @notice Verifies that a report has been signed by the correct
    /// @param settingsDigest The agent setting digest
    /// @param hash The keccak256 hash of the raw report's bytes
    /// @param merkleProof is the signature on report.
    function _verifyMerkle(
        bytes32 settingsDigest,
        bytes32 hash,
        bytes memory merkleProof
    ) private pure {
        settingsDigest = settingsDigest;
        hash = hash;
        merkleProof = merkleProof;

        revert UnsupportedProofMethod();
    }

    function _addressExists(
        address[] memory addrs,
        address addr
    ) private pure returns (bool) {
        for (uint256 i = 0; i < addrs.length; i++) {
            if (addrs[i] == addr) {
                return true;
            }
        }
        return false;
    }

    function _addressAdd(
        address[] memory addrs,
        address addr
    ) private pure returns (address[] memory) {
        address[] memory expand = new address[](addrs.length + 1);

        for (uint256 i = 0; i < addrs.length; i++) {
            expand[i] = addrs[i];
        }

        expand[addrs.length] = addr;

        return expand;
    }
}
