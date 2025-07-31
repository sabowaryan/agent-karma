/// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract AgentRegistry {
    struct AgentMetadata {
        string name;
        string description;
        string framework;
        string version;
        string ipfsHash;
    }

    struct Agent {
        address agentAddress;
        uint256 registrationDate;
        AgentMetadata metadata;
        uint256 karmaScore;
        uint64 interactionCount;
        uint64 ratingsReceived;
    }

    mapping(address => Agent) public agents;
    mapping(address => bool) public isRegistered;

    event AgentRegistered(address indexed agentAddress, string name, string framework);
    event AgentMetadataUpdated(address indexed agentAddress, string name);

    function registerAgent(AgentMetadata calldata metadata) external {
        require(!isRegistered[msg.sender], "Agent already registered");
        require(bytes(metadata.name).length > 0, "Name cannot be empty");

        agents[msg.sender] = Agent({
            agentAddress: msg.sender,
            registrationDate: block.timestamp,
            metadata: metadata,
            karmaScore: 0,
            interactionCount: 0,
            ratingsReceived: 0
        });
        isRegistered[msg.sender] = true;

        emit AgentRegistered(msg.sender, metadata.name, metadata.framework);
    }

    function getAgentInfo(address agentAddress) public view returns (Agent memory) {
        require(isRegistered[agentAddress], "Agent not registered");
        return agents[agentAddress];
    }

    function isAgentRegistered(address agentAddress) public view returns (bool) {
        return isRegistered[agentAddress];
    }

    function updateAgentMetadata(AgentMetadata calldata metadata) external {
        require(isRegistered[msg.sender], "Agent not registered");
        require(bytes(metadata.name).length > 0, "Name cannot be empty");

        agents[msg.sender].metadata = metadata;

        emit AgentMetadataUpdated(msg.sender, metadata.name);
    }
}


