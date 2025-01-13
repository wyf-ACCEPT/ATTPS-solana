// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {IAgentManager} from "./interfaces/IAgentManager.sol";
import {IAgentProxy} from "./interfaces/IAgentProxy.sol";
import {IAgentFactory} from "./interfaces/IAgentFactory.sol";
import {IConverter} from "./interfaces/IConverter.sol";
import {ITypeAndVersion} from "./interfaces/ITypeAndVersion.sol";
import {Common} from "./libraries/Common.sol";
import {ConfirmedOwner} from "./access/ConfirmedOwner.sol";
import {Address} from "./vendor/openzeppelin-solidity/v4.8.3/contracts/utils/Address.sol";
import {EnumerableSet} from "./vendor/openzeppelin-solidity/v4.8.3/contracts/utils/structs/EnumerableSet.sol";

// import "@openzeppelin/contracts/utils/Address.sol";
// import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

/// @notice A contract to handle access control of AI Agent Manager
contract AgentManager is IAgentManager, ITypeAndVersion, ConfirmedOwner {
    using Address for address;
    using EnumerableSet for EnumerableSet.AddressSet;
    using EnumerableSet for EnumerableSet.Bytes32Set;

    event AgentRegistered(
        address indexed agent,
        Common.AgentSettings agentSettings
    );
    event AgentAccepted(
        address indexed agent,
        bytes32 indexed digest,
        Common.AgentSettings agentSettings
    );
    event AgentRemoved(address indexed agent);
    event AgentSettingsProposed(
        address indexed agent,
        Common.AgentSettings agentSettings
    );
    event AgentSettingsUpdated(
        address indexed agent,
        bytes32 indexed digest,
        Common.AgentSettings agentSettings
    );
    event AgentProxySet(address oldProxy, address newProxy);

    error InvalidRegisteredAgent();
    error AgentIsRegistered();
    error InvalidAllowedAgent();
    error InvalidAgent();
    error InvalidFactoryAgent();
    error AgentIsAllowed();
    error InvalidCallData();
    error InvalidAgentConfig();
    error InvalidAgentSettingProposal();
    error InvalidAgentHeaderVersion();
    error InvalidAgentHeaderMessageId();
    error InvalidAgentHeaderAgentId();
    error InvalidAgentHeaderMessageType();
    error InvalidAgentHeaderPriority();

    /// @inheritdoc ITypeAndVersion
    string public constant override typeAndVersion = "AI Agent Manager v1.0.0";
    string private constant s_agentVersion = "1.0";
    EnumerableSet.Bytes32Set private s_agentSourceIds;
    EnumerableSet.AddressSet private s_registeringAgents;
    EnumerableSet.AddressSet private s_allowedAgents;
    mapping(address => Common.AgentSettings) private s_agentSettings;
    mapping(address => bool) private s_agentNewSettings;
    mapping(address => Common.AgentConfigState) private s_agentConfigStates;
    IAgentProxy private i_agentProxy;

    // ================================================================
    // |                       Initialization                         |
    // ================================================================

    constructor(address proxy) ConfirmedOwner(msg.sender) {
        i_agentProxy = IAgentProxy(proxy);
    }

    // ================================================================
    // |                        Configuration                         |
    // ================================================================

    // ================================================================
    // |                      Allow methods                           |
    // ================================================================

    /// @inheritdoc IAgentManager
    function registerAgent(
        address agent,
        Common.AgentSettings memory agentSettings
    ) external override {
        // Confirm that the agent is deployed by the system factory.
        if (!IAgentFactory(i_agentProxy.agentFactory()).hasAgent(agent))
            revert InvalidFactoryAgent();

        // Validate the Validity of Headers
        _validateAgentHeader(agentSettings.agentHeader);

        if (s_registeringAgents.contains(agent)) revert AgentIsRegistered();

        if (s_allowedAgents.contains(agent)) revert AgentIsAllowed();

        if (
            s_agentSourceIds.contains(
                keccak256(
                    abi.encodePacked(agentSettings.agentHeader.sourceAgentId)
                )
            )
        ) revert InvalidAgentHeaderAgentId();

        // Add recipient to the s_registered list
        if (s_registeringAgents.add(agent)) {
            s_agentSourceIds.add(
                keccak256(
                    abi.encodePacked(agentSettings.agentHeader.sourceAgentId)
                )
            );
            s_agentSettings[agent] = agentSettings;
            s_agentNewSettings[agent] = true;

            emit AgentRegistered(agent, agentSettings);
        }
    }

    /// @inheritdoc IAgentManager
    function acceptAgent(address agent) external override onlyOwner {
        if (!s_registeringAgents.contains(agent)) {
            revert InvalidRegisteredAgent();
        }

        s_registeringAgents.remove(agent);

        // Add recipient to the allow list
        if (s_allowedAgents.add(agent)) {
            // add the config to the config states
            Common.AgentSettings memory settings = s_agentSettings[agent];
            bytes32 digest = _settingDigestFromSettingsData(agent, settings);
            Common.AgentConfig memory config = Common.AgentConfig({
                configDigest: digest,
                configBlockNumber: uint32(block.number),
                isActive: true,
                settings: settings
            });

            s_agentConfigStates[agent].latestConfigDigest = digest;
            s_agentConfigStates[agent].configs.push(config);
            s_agentNewSettings[agent] = false;

            emit AgentAccepted(agent, digest, settings);
        }
    }

    /// @inheritdoc IAgentManager
    function changeAgentSettingProposal(
        address agent,
        Common.AgentSettings memory agentSettings
    ) external override {
        Common.AgentSettings memory settings;

        // Validate the Validity of Headers
        _validateAgentHeader(agentSettings.agentHeader);

        if (
            !s_registeringAgents.contains(agent) &&
            !s_allowedAgents.contains(agent)
        ) revert InvalidAllowedAgent();

        if (s_allowedAgents.contains(agent)) {
            settings = _getAgentConfigByDigest(
                agent,
                s_agentConfigStates[agent].latestConfigDigest
            ).settings;
        } else {
            settings = s_agentSettings[agent];
        }

        if (
            keccak256(abi.encodePacked(settings.agentHeader.sourceAgentId)) !=
            keccak256(abi.encodePacked(agentSettings.agentHeader.sourceAgentId))
        ) revert InvalidAgentHeaderAgentId();

        bytes32 digest = _settingDigestFromSettingsData(agent, agentSettings);
        if (
            s_registeringAgents.contains(agent) &&
            digest ==
            _settingDigestFromSettingsData(agent, s_agentSettings[agent])
        ) revert InvalidAgentConfig();
        if (
            s_agentNewSettings[agent] &&
            digest ==
            _settingDigestFromSettingsData(agent, s_agentSettings[agent])
        ) revert InvalidAgentConfig();
        if (
            s_allowedAgents.contains(agent) &&
            _isAgentConfigExists(agent, digest)
        ) revert InvalidAgentConfig();

        s_agentSettings[agent] = agentSettings;
        s_agentNewSettings[agent] = true;

        emit AgentSettingsProposed(agent, agentSettings);
    }

    /// @inheritdoc IAgentManager
    function acceptAgentSettingProposal(
        address agent
    ) external override onlyOwner {
        if (!s_allowedAgents.contains(agent)) revert InvalidAllowedAgent();

        if (!s_agentNewSettings[agent]) revert InvalidAgentSettingProposal();

        // add the config to the config states
        Common.AgentSettings memory settings = s_agentSettings[agent];
        bytes32 digest = _settingDigestFromSettingsData(agent, settings);
        Common.AgentConfig memory config = Common.AgentConfig({
            configDigest: digest,
            configBlockNumber: uint32(block.number),
            isActive: true,
            settings: settings
        });

        if (_isAgentConfigExists(agent, digest)) revert InvalidAgentConfig();

        s_agentConfigStates[agent].latestConfigDigest = digest;
        s_agentConfigStates[agent].configs.push(config);
        s_agentNewSettings[agent] = false;

        emit AgentSettingsUpdated(agent, digest, settings);
    }

    /// @inheritdoc IAgentManager
    function getAllRegisteringAgents()
        external
        view
        override
        returns (address[] memory)
    {
        return s_registeringAgents.values();
    }

    /// @inheritdoc IAgentManager
    function getRegisteringAgentsCount()
        external
        view
        override
        returns (uint64)
    {
        return uint64(s_registeringAgents.length());
    }

    /// @inheritdoc IAgentManager
    function getRegisteringAgentsInRange(
        uint64 registeringAgentIdxStart,
        uint64 registeringAgentIdxEnd
    ) external view override returns (address[] memory registeringAgents) {
        if (
            registeringAgentIdxStart > registeringAgentIdxEnd ||
            registeringAgentIdxEnd >= s_registeringAgents.length()
        ) {
            revert InvalidCallData();
        }

        registeringAgents = new address[](
            (registeringAgentIdxEnd - registeringAgentIdxStart) + 1
        );
        for (
            uint256 i = 0;
            i <= registeringAgentIdxEnd - registeringAgentIdxStart;
            ++i
        ) {
            registeringAgents[i] = s_registeringAgents.at(
                uint256(registeringAgentIdxStart + i)
            );
        }

        return registeringAgents;
    }

    /// @inheritdoc IAgentManager
    function getAllAllowedAgents()
        external
        view
        override
        returns (address[] memory)
    {
        return s_allowedAgents.values();
    }

    /// @inheritdoc IAgentManager
    function getAllowedAgentsCount() external view override returns (uint64) {
        return uint64(s_allowedAgents.length());
    }

    /// @inheritdoc IAgentManager
    function getAllowedAgentsInRange(
        uint64 allowedAgentIdxStart,
        uint64 allowedAgentIdxEnd
    ) external view override returns (address[] memory allowedAgents) {
        if (
            allowedAgentIdxStart > allowedAgentIdxEnd ||
            allowedAgentIdxEnd >= s_allowedAgents.length()
        ) {
            revert InvalidCallData();
        }

        allowedAgents = new address[](
            (allowedAgentIdxEnd - allowedAgentIdxStart) + 1
        );
        for (
            uint256 i = 0;
            i <= allowedAgentIdxEnd - allowedAgentIdxStart;
            ++i
        ) {
            allowedAgents[i] = s_allowedAgents.at(
                uint256(allowedAgentIdxStart + i)
            );
        }

        return allowedAgents;
    }

    /// @inheritdoc IAgentManager
    function getAgentConfigs(
        address agent
    ) external view override returns (Common.AgentConfig[] memory) {
        if (false == s_allowedAgents.contains(agent))
            revert InvalidAllowedAgent();

        return s_agentConfigStates[agent].configs;
    }

    /// @inheritdoc IAgentManager
    function getAgentConfigsCount(
        address agent
    ) external view override returns (uint64) {
        if (false == s_allowedAgents.contains(agent))
            revert InvalidAllowedAgent();

        return uint64(s_agentConfigStates[agent].configs.length);
    }

    /// @inheritdoc IAgentManager
    function getAgentConfigsInRange(
        address agent,
        uint64 agentConfigIdxStart,
        uint64 agentConfigIdxEnd
    )
        external
        view
        override
        returns (Common.AgentConfig[] memory agentConfigs)
    {
        if (false == s_allowedAgents.contains(agent))
            revert InvalidAllowedAgent();

        if (
            agentConfigIdxStart > agentConfigIdxEnd ||
            agentConfigIdxEnd >= s_agentConfigStates[agent].configs.length
        ) {
            revert InvalidCallData();
        }

        agentConfigs = new Common.AgentConfig[](
            (agentConfigIdxEnd - agentConfigIdxStart) + 1
        );
        for (uint256 i = 0; i <= agentConfigIdxEnd - agentConfigIdxStart; ++i) {
            agentConfigs[i] = s_agentConfigStates[agent].configs[
                uint256(agentConfigIdxStart + i)
            ];
        }

        return agentConfigs;
    }

    /// @inheritdoc IAgentManager
    function getAgentConfig(
        address agent,
        bytes32 settingDigest
    ) external view override returns (Common.AgentConfig memory) {
        if (false == s_allowedAgents.contains(agent))
            revert InvalidAllowedAgent();

        return _getAgentConfigByDigest(agent, settingDigest);
    }

    /// @inheritdoc IAgentManager
    function allowedAgent(address agent) external view override returns (bool) {
        return s_allowedAgents.contains(agent);
    }

    /// @inheritdoc IAgentManager
    function allowedSigner(
        address agent,
        bytes32 settingDigest,
        address signer
    ) external view override returns (bool) {
        if (true == s_allowedAgents.contains(agent)) {
            Common.AgentConfig memory config = _getAgentConfigByDigest(
                agent,
                settingDigest
            );

            for (uint256 i = 0; i < config.settings.signers.length; ++i) {
                if (config.settings.signers[i] == signer) {
                    return true;
                }
            }
            return false;
        } else {
            revert InvalidAllowedAgent();
        }
    }

    /// @inheritdoc IAgentManager
    function signerThreshold(
        address agent,
        bytes32 settingDigest
    ) external view override returns (uint8) {
        if (true == s_allowedAgents.contains(agent)) {
            Common.AgentConfig memory config = _getAgentConfigByDigest(
                agent,
                settingDigest
            );

            return config.settings.threshold;
        } else {
            revert InvalidAllowedAgent();
        }
    }

    /// @inheritdoc IAgentManager
    function validateDataConversion(
        address agent,
        bytes calldata data
    ) external view override returns (bytes memory) {
        if (true == s_allowedAgents.contains(agent)) {
            if (s_agentSettings[agent].converterAddress != address(0)) {
                Common.AgentConfig memory config = _getAgentConfigByDigest(
                    agent,
                    s_agentConfigStates[agent].latestConfigDigest
                );

                return
                    IConverter(config.settings.converterAddress).converter(
                        data
                    );
            } else {
                return data;
            }
        } else {
            revert InvalidAllowedAgent();
        }
    }

    /// @inheritdoc IAgentManager
    function agentVersion() external pure override returns (string memory) {
        return s_agentVersion;
    }

    /// @inheritdoc IAgentManager
    function isValidMessageId(
        string memory messageId
    ) external pure override returns (bool) {
        return _isValidUUID(messageId);
    }

    /// @inheritdoc IAgentManager
    function isValidSourceAgentId(
        string memory sourceAgentId
    ) external view override returns (bool) {
        return
            _isValidUUID(sourceAgentId) &&
            !s_agentSourceIds.contains(
                keccak256(abi.encodePacked(sourceAgentId))
            );
    }

    /// @inheritdoc IAgentManager
    function agentProxy() external view override returns (address) {
        return address(i_agentProxy);
    }

    /// @inheritdoc IAgentManager
    function setAgentProxy(address proxy) external override onlyOwner {
        address old = address(i_agentProxy);
        i_agentProxy = IAgentProxy(proxy);

        emit AgentProxySet(old, proxy);
    }

    // ================================================================
    // |                         Remove methods                        |
    // ================================================================
    /// @inheritdoc IAgentManager
    function removeAgent(address agent) external override onlyOwner {
        s_registeringAgents.remove(agent);
        s_allowedAgents.remove(agent);
        s_agentNewSettings[agent] = false;

        emit AgentRemoved(agent);
    }

    // ================================================================
    // |                       Internal Methods                       |
    // ================================================================
    function _settingDigestFromSettingsData(
        address agent,
        Common.AgentSettings memory settings
    ) internal pure returns (bytes32) {
        uint256 h = uint256(
            keccak256(
                abi.encode(
                    agent,
                    settings.signers,
                    settings.threshold,
                    settings.converterAddress,
                    settings.agentHeader.version,
                    settings.agentHeader.messageId,
                    settings.agentHeader.sourceAgentId,
                    settings.agentHeader.sourceAgentName,
                    settings.agentHeader.targetAgentId,
                    settings.agentHeader.timestamp,
                    settings.agentHeader.messageType,
                    settings.agentHeader.priority
                )
            )
        );
        uint256 prefixMask = type(uint256).max << (256 - 16); // 0xFFFF00..00
        // 0x0100 represents version v1.0
        uint256 prefix = 0x0100 << (256 - 16); // 0x010000..00
        return bytes32((prefix & prefixMask) | (h & ~prefixMask));
    }

    function _isAgentConfigExists(
        address agent,
        bytes32 digest
    ) internal view returns (bool) {
        for (uint i = 0; i < s_agentConfigStates[agent].configs.length; i++) {
            if (s_agentConfigStates[agent].configs[i].configDigest == digest) {
                return true;
            }
        }
        return false;
    }

    function _getAgentConfigByDigest(
        address agent,
        bytes32 digest
    ) internal view returns (Common.AgentConfig memory) {
        for (uint i = 0; i < s_agentConfigStates[agent].configs.length; i++) {
            if (s_agentConfigStates[agent].configs[i].configDigest == digest) {
                return s_agentConfigStates[agent].configs[i];
            }
        }

        revert InvalidAgentConfig();
    }

    function _validateAgentHeader(
        Common.AgentHeader memory header
    ) internal pure {
        if (
            keccak256(abi.encodePacked(header.version)) !=
            keccak256(abi.encodePacked(s_agentVersion))
        ) revert InvalidAgentHeaderVersion();
        if (!_isValidUUID(header.sourceAgentId))
            revert InvalidAgentHeaderAgentId();
        if (!_isValidUUID(header.messageId))
            revert InvalidAgentHeaderMessageId();
        if (!_isValidMessageType(header.messageType))
            revert InvalidAgentHeaderMessageType();
        if (!_isValidPriority(header.priority))
            revert InvalidAgentHeaderPriority();
    }

    function _isValidMessageType(
        Common.MessageType index
    ) internal pure returns (bool) {
        return
            index >= Common.MessageType.Request &&
            index <= Common.MessageType.Event;
    }

    function _isValidPriority(
        Common.Priority index
    ) internal pure returns (bool) {
        return index >= Common.Priority.High && index <= Common.Priority.Low;
    }

    // Check if the input is a valid UUID v4 format
    function _isValidUUID(string memory uuid) internal pure returns (bool) {
        bytes memory b = bytes(uuid);

        // UUID v4 length must be 36 and include fixed '4' and 8-9/A-B positions.
        if (b.length != 36) return false;

        // Validate format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
        for (uint256 i = 0; i < b.length; i++) {
            if (i == 8 || i == 13 || i == 18 || i == 23) {
                if (b[i] != "-") return false;
            } else if (i == 14) {
                if (b[i] != "4") return false; // The 13th character must be '4'
            } else if (i == 19) {
                if (
                    !(b[i] == "8" || b[i] == "9" || b[i] == "a" || b[i] == "b")
                ) {
                    return false; // The 17th character must start with 10xx
                }
            } else {
                if (
                    !(b[i] >= "0" && b[i] <= "9") &&
                    !(b[i] >= "a" && b[i] <= "f") &&
                    !(b[i] >= "A" && b[i] <= "F")
                ) return false;
            }
        }

        return true;
    }
}
