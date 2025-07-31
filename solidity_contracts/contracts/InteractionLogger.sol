/// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./AgentRegistry.sol";

contract InteractionLogger {
    struct InteractionMetadata {
        uint64 duration;
        string outcome;
        string context;
    }

    struct Interaction {
        bytes32 id;
        address[] participants;
        string interactionType;
        uint256 timestamp;
        uint64 blockHeight;
        InteractionMetadata metadata;
    }

    AgentRegistry public agentRegistry;
    mapping(bytes32 => Interaction) public interactions;
    mapping(address => bytes32[]) public agentInteractions;

    event InteractionLogged(bytes32 indexed interactionHash, address[] participants, string interactionType);

    constructor(address _agentRegistry) {
        agentRegistry = AgentRegistry(_agentRegistry);
    }

    function logInteraction(
        address[] calldata participants,
        string calldata interactionType,
        InteractionMetadata calldata metadata
    ) external returns (bytes32) {
        require(participants.length > 0, "At least one participant required");
        require(bytes(interactionType).length > 0, "Interaction type cannot be empty");

        // Verify all participants are registered
        for (uint i = 0; i < participants.length; i++) {
            require(agentRegistry.isAgentRegistered(participants[i]), "Participant not registered");
        }

        // Generate a unique hash for the interaction
        bytes32 interactionHash = keccak256(abi.encode(
            participants,
            interactionType,
            block.timestamp,
            block.number
        ));

        Interaction memory newInteraction = Interaction({
            id: interactionHash,
            participants: participants,
            interactionType: interactionType,
            timestamp: block.timestamp,
            blockHeight: uint64(block.number),
            metadata: metadata
        });

        interactions[interactionHash] = newInteraction;

        // Add to each participant's interaction history
        for (uint i = 0; i < participants.length; i++) {
            agentInteractions[participants[i]].push(interactionHash);
        }

        emit InteractionLogged(interactionHash, participants, interactionType);
        return interactionHash;
    }

    function getInteractionHistory(address agentAddress, uint32 limit) public view returns (Interaction[] memory) {
        require(agentRegistry.isAgentRegistered(agentAddress), "Agent not registered");
        
        bytes32[] memory interactionHashes = agentInteractions[agentAddress];
        uint256 length = limit == 0 || limit >= interactionHashes.length ? 
            interactionHashes.length : limit;
        
        Interaction[] memory history = new Interaction[](length);
        uint256 startIndex = interactionHashes.length > length ? 
            interactionHashes.length - length : 0;
        
        for (uint i = 0; i < length; i++) {
            history[i] = interactions[interactionHashes[startIndex + i]];
        }
        
        return history;
    }

    function verifyInteraction(bytes32 interactionHash) public view returns (bool) {
        return interactions[interactionHash].id == interactionHash;
    }

    function getInteractionByHash(bytes32 interactionHash) public view returns (Interaction memory) {
        require(verifyInteraction(interactionHash), "Interaction not found");
        return interactions[interactionHash];
    }

    function getInteractionCount(address agentAddress) public view returns (uint256) {
        return agentInteractions[agentAddress].length;
    }
}


