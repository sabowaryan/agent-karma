/// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./AgentRegistry.sol";

contract OracleIntegration {
    struct OracleData {
        address provider;
        string dataType;
        string data;
        uint256 timestamp;
        string[] signatures;
        bool verified;
    }

    AgentRegistry public agentRegistry;
    mapping(bytes32 => OracleData) public oracleData;
    mapping(string => bytes32[]) public dataTypeHashes;
    mapping(address => bool) public authorizedProviders;
    
    uint256 public constant MIN_SIGNATURES_REQUIRED = 3;
    address public owner;

    event OracleDataSubmitted(address indexed provider, string dataType, bytes32 indexed dataHash);
    event OracleDataVerified(bytes32 indexed dataHash);
    event ProviderAuthorized(address indexed provider);
    event ProviderRevoked(address indexed provider);

    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner can call this function");
        _;
    }

    modifier onlyAuthorizedProvider() {
        require(authorizedProviders[msg.sender], "Provider not authorized");
        _;
    }

    constructor(address _agentRegistry) {
        agentRegistry = AgentRegistry(_agentRegistry);
        owner = msg.sender;
    }

    function authorizeProvider(address provider) external onlyOwner {
        require(agentRegistry.isAgentRegistered(provider), "Provider must be registered agent");
        authorizedProviders[provider] = true;
        emit ProviderAuthorized(provider);
    }

    function revokeProvider(address provider) external onlyOwner {
        authorizedProviders[provider] = false;
        emit ProviderRevoked(provider);
    }

    function submitOracleData(
        string calldata dataType,
        string calldata data,
        string[] calldata signatures
    ) external onlyAuthorizedProvider {
        require(bytes(dataType).length > 0, "Data type cannot be empty");
        require(bytes(data).length > 0, "Data cannot be empty");
        require(signatures.length >= MIN_SIGNATURES_REQUIRED, "Insufficient signatures");

        bytes32 dataHash = keccak256(abi.encodePacked(
            msg.sender,
            dataType,
            data,
            block.timestamp
        ));

        OracleData memory newOracleData = OracleData({
            provider: msg.sender,
            dataType: dataType,
            data: data,
            timestamp: block.timestamp,
            signatures: signatures,
            verified: false
        });

        oracleData[dataHash] = newOracleData;
        dataTypeHashes[dataType].push(dataHash);

        // Auto-verify if enough signatures (simplified verification)
        if (signatures.length >= MIN_SIGNATURES_REQUIRED) {
            oracleData[dataHash].verified = true;
            emit OracleDataVerified(dataHash);
        }

        emit OracleDataSubmitted(msg.sender, dataType, dataHash);
    }

    function verifyOracleConsensus(bytes32 dataHash) public view returns (bool) {
        OracleData memory data = oracleData[dataHash];
        return data.verified && data.signatures.length >= MIN_SIGNATURES_REQUIRED;
    }

    function getOracleData(
        string calldata dataType,
        uint256 timestamp
    ) public view returns (OracleData[] memory) {
        bytes32[] memory hashes = dataTypeHashes[dataType];
        uint256 matchingCount = 0;

        // Count matching entries
        for (uint i = 0; i < hashes.length; i++) {
            OracleData memory data = oracleData[hashes[i]];
            if (data.verified && (timestamp == 0 || data.timestamp >= timestamp)) {
                matchingCount++;
            }
        }

        // Create result array
        OracleData[] memory result = new OracleData[](matchingCount);
        uint256 resultIndex = 0;

        // Populate result array
        for (uint i = 0; i < hashes.length; i++) {
            OracleData memory data = oracleData[hashes[i]];
            if (data.verified && (timestamp == 0 || data.timestamp >= timestamp)) {
                result[resultIndex] = data;
                resultIndex++;
            }
        }

        return result;
    }

    function getLatestOracleData(string calldata dataType) public view returns (OracleData memory) {
        bytes32[] memory hashes = dataTypeHashes[dataType];
        require(hashes.length > 0, "No data found for this type");

        bytes32 latestHash = hashes[0];
        uint256 latestTimestamp = 0;

        // Find the latest verified data
        for (uint i = 0; i < hashes.length; i++) {
            OracleData memory data = oracleData[hashes[i]];
            if (data.verified && data.timestamp > latestTimestamp) {
                latestHash = hashes[i];
                latestTimestamp = data.timestamp;
            }
        }

        require(latestTimestamp > 0, "No verified data found");
        return oracleData[latestHash];
    }

    function getDataHash(
        address provider,
        string calldata dataType,
        string calldata data,
        uint256 timestamp
    ) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(provider, dataType, data, timestamp));
    }

    function isProviderAuthorized(address provider) public view returns (bool) {
        return authorizedProviders[provider];
    }

    function getDataTypeCount(string calldata dataType) public view returns (uint256) {
        return dataTypeHashes[dataType].length;
    }
}

