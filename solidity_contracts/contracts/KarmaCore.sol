/// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./AgentRegistry.sol";

contract KarmaCore {
    struct Rating {
        bytes32 id;
        address raterAddress;
        address ratedAddress;
        uint8 score;
        string feedback;
        bytes32 interactionHash;
        uint256 timestamp;
        uint64 blockHeight;
    }

    struct KarmaFactors {
        string averageRating;
        uint64 ratingCount;
        uint256 interactionFrequency;
        string timeDecay;
        uint256 externalFactors;
    }

    struct KarmaCalculation {
        address agentAddress;
        uint256 currentScore;
        uint256 previousScore;
        KarmaFactors factors;
        uint256 lastUpdated;
        bytes32 calculationHash;
    }

    struct KarmaConfig {
        uint256 minKarmaForRating;
        uint256 minKarmaForVoting;
        uint256 minKarmaForProposal;
        uint64 ratingWindow;
        uint8 maxRatingsPerInteraction;
        uint256 ratingFee;
    }

    AgentRegistry public agentRegistry;
    mapping(address => uint256) public karmaScores;
    mapping(bytes32 => Rating) public ratings;
    mapping(address => mapping(bytes32 => bool)) public hasRatedInteraction;
    mapping(address => Rating[]) public agentRatings;
    mapping(address => KarmaCalculation[]) public karmaHistory;
    
    KarmaConfig public config;

    event RatingSubmitted(address indexed rater, address indexed ratedAgent, uint8 score, bytes32 interactionHash);
    event KarmaScoreUpdated(address indexed agentAddress, uint256 newScore, uint256 oldScore);

    constructor(address _agentRegistry) {
        agentRegistry = AgentRegistry(_agentRegistry);
        config = KarmaConfig({
            minKarmaForRating: 0,
            minKarmaForVoting: 50,
            minKarmaForProposal: 100,
            ratingWindow: 86400, // 24 hours
            maxRatingsPerInteraction: 10,
            ratingFee: 1
        });
    }

    function submitRating(
        address ratedAgent,
        uint8 score,
        string calldata feedback,
        bytes32 interactionHash
    ) external {
        require(agentRegistry.isAgentRegistered(msg.sender), "Rater not registered");
        require(agentRegistry.isAgentRegistered(ratedAgent), "Rated agent not registered");
        require(score >= 1 && score <= 10, "Score must be between 1 and 10");
        require(!hasRatedInteraction[msg.sender][interactionHash], "Already rated this interaction");
        require(karmaScores[msg.sender] >= config.minKarmaForRating, "Insufficient karma to rate");

        bytes32 ratingId = keccak256(abi.encodePacked(msg.sender, ratedAgent, interactionHash, block.timestamp));
        
        Rating memory newRating = Rating({
            id: ratingId,
            raterAddress: msg.sender,
            ratedAddress: ratedAgent,
            score: score,
            feedback: feedback,
            interactionHash: interactionHash,
            timestamp: block.timestamp,
            blockHeight: uint64(block.number)
        });

        ratings[ratingId] = newRating;
        hasRatedInteraction[msg.sender][interactionHash] = true;
        agentRatings[ratedAgent].push(newRating);

        // Update karma score
        uint256 oldScore = karmaScores[ratedAgent];
        uint256 newScore = _calculateNewKarmaScore(ratedAgent);
        karmaScores[ratedAgent] = newScore;

        emit RatingSubmitted(msg.sender, ratedAgent, score, interactionHash);
        emit KarmaScoreUpdated(ratedAgent, newScore, oldScore);
    }

    function _calculateNewKarmaScore(address agentAddress) internal view returns (uint256) {
        Rating[] memory ratings = agentRatings[agentAddress];
        if (ratings.length == 0) return 0;

        uint256 totalScore = 0;
        uint256 validRatings = 0;

        for (uint i = 0; i < ratings.length; i++) {
            // Apply time decay (simplified version)
            uint256 timeDiff = block.timestamp - ratings[i].timestamp;
            if (timeDiff < 30 days) {
                totalScore += ratings[i].score; // No scaling to avoid overflow
                validRatings++;
            }
        }

        if (validRatings == 0) return 0;
        
        uint256 averageScore = totalScore / validRatings;
        // Apply interaction frequency bonus
        uint256 frequencyBonus = validRatings > 10 ? 10 : validRatings;
        
        return averageScore + frequencyBonus;
    }

    function calculateKarma(address agentAddress) public view returns (KarmaCalculation memory) {
        require(agentRegistry.isAgentRegistered(agentAddress), "Agent not registered");
        
        Rating[] memory ratings = agentRatings[agentAddress];
        uint256 currentScore = karmaScores[agentAddress];
        
        // Calculate factors
        uint256 totalScore = 0;
        uint256 validRatings = 0;
        
        for (uint i = 0; i < ratings.length; i++) {
            uint256 timeDiff = block.timestamp - ratings[i].timestamp;
            if (timeDiff < 30 days) {
                totalScore += ratings[i].score;
                validRatings++;
            }
        }
        
        string memory averageRating = validRatings > 0 ? 
            _uint256ToString((totalScore * 100) / validRatings) : "0";
        
        KarmaFactors memory factors = KarmaFactors({
            averageRating: averageRating,
            ratingCount: uint64(validRatings),
            interactionFrequency: validRatings,
            timeDecay: "100", // Simplified
            externalFactors: 0
        });
        
        bytes32 calculationHash = keccak256(abi.encodePacked(
            agentAddress,
            currentScore,
            block.timestamp
        ));
        
        return KarmaCalculation({
            agentAddress: agentAddress,
            currentScore: currentScore,
            previousScore: 0, // Would need to track this
            factors: factors,
            lastUpdated: block.timestamp,
            calculationHash: calculationHash
        });
    }

    function getKarmaScore(address agentAddress) public view returns (uint256) {
        return karmaScores[agentAddress];
    }

    function getKarmaHistory(address agentAddress, uint32 limit) public view returns (KarmaCalculation[] memory) {
        KarmaCalculation[] memory history = karmaHistory[agentAddress];
        if (limit == 0 || limit >= history.length) {
            return history;
        }
        
        KarmaCalculation[] memory limitedHistory = new KarmaCalculation[](limit);
        uint256 startIndex = history.length - limit;
        
        for (uint i = 0; i < limit; i++) {
            limitedHistory[i] = history[startIndex + i];
        }
        
        return limitedHistory;
    }

    function getAgentRatings(address agentAddress, uint32 limit) public view returns (Rating[] memory) {
        Rating[] memory ratings = agentRatings[agentAddress];
        if (limit == 0 || limit >= ratings.length) {
            return ratings;
        }
        
        Rating[] memory limitedRatings = new Rating[](limit);
        uint256 startIndex = ratings.length - limit;
        
        for (uint i = 0; i < limit; i++) {
            limitedRatings[i] = ratings[startIndex + i];
        }
        
        return limitedRatings;
    }

    function _uint256ToString(uint256 value) internal pure returns (string memory) {
        if (value == 0) {
            return "0";
        }
        uint256 temp = value;
        uint256 digits;
        while (temp != 0) {
            digits++;
            temp /= 10;
        }
        bytes memory buffer = new bytes(digits);
        while (value != 0) {
            digits -= 1;
            buffer[digits] = bytes1(uint8(48 + uint256(value % 10)));
            value /= 10;
        }
        return string(buffer);
    }
}


