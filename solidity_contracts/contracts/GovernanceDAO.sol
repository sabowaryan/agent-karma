/// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./AgentRegistry.sol";
import "./KarmaCore.sol";

contract GovernanceDAO {
    enum ProposalStatus {
        Active,
        Passed,
        Failed,
        Executed
    }

    struct Proposal {
        uint256 id;
        string title;
        string description;
        address proposer;
        bytes calldataToExecute;
        uint256 createdAt;
        uint256 votingDeadline;
        bool executed;
        uint256 votesFor;
        uint256 votesAgainst;
        uint256 quorumRequired;
        ProposalStatus status;
    }

    struct Vote {
        uint256 proposalId;
        address voter;
        bool support;
        uint256 votingPower;
        uint256 timestamp;
        uint64 blockHeight;
    }

    AgentRegistry public agentRegistry;
    KarmaCore public karmaCore;
    
    uint256 public nextProposalId = 1;
    mapping(uint256 => Proposal) public proposals;
    mapping(uint256 => mapping(address => Vote)) public votes;
    mapping(uint256 => mapping(address => bool)) public hasVoted;
    
    uint256 public constant MIN_VOTING_PERIOD = 1 days;
    uint256 public constant MAX_VOTING_PERIOD = 30 days;
    uint256 public constant QUORUM_PERCENTAGE = 10; // 10% of total karma

    event ProposalCreated(uint256 indexed proposalId, address indexed proposer, string title);
    event VoteCast(uint256 indexed proposalId, address indexed voter, bool support, uint256 votingPower);
    event ProposalFinalized(uint256 indexed proposalId, bool passed, bool executed);

    constructor(address _agentRegistry, address _karmaCore) {
        agentRegistry = AgentRegistry(_agentRegistry);
        karmaCore = KarmaCore(_karmaCore);
    }

    function createProposal(
        string calldata title,
        string calldata description,
        bytes calldata calldataToExecute,
        uint64 votingPeriod
    ) external returns (uint256) {
        require(agentRegistry.isAgentRegistered(msg.sender), "Proposer not registered");
        require(bytes(title).length > 0, "Title cannot be empty");
        require(votingPeriod >= MIN_VOTING_PERIOD && votingPeriod <= MAX_VOTING_PERIOD, "Invalid voting period");
        
        uint256 proposerKarma = karmaCore.getKarmaScore(msg.sender);
        require(proposerKarma >= 100, "Insufficient karma to create proposal"); // MIN_KARMA_FOR_PROPOSAL

        uint256 proposalId = nextProposalId++;
        uint256 deadline = block.timestamp + votingPeriod;
        
        proposals[proposalId] = Proposal({
            id: proposalId,
            title: title,
            description: description,
            proposer: msg.sender,
            calldataToExecute: calldataToExecute,
            createdAt: block.timestamp,
            votingDeadline: deadline,
            executed: false,
            votesFor: 0,
            votesAgainst: 0,
            quorumRequired: _calculateQuorum(),
            status: ProposalStatus.Active
        });

        emit ProposalCreated(proposalId, msg.sender, title);
        return proposalId;
    }

    function voteProposal(uint256 proposalId, bool support) external {
        require(agentRegistry.isAgentRegistered(msg.sender), "Voter not registered");
        require(proposals[proposalId].id == proposalId, "Proposal does not exist");
        require(block.timestamp <= proposals[proposalId].votingDeadline, "Voting period ended");
        require(!hasVoted[proposalId][msg.sender], "Already voted");
        
        uint256 votingPower = calculateVotingPower(msg.sender);
        require(votingPower >= 50, "Insufficient karma to vote"); // MIN_KARMA_FOR_VOTING

        Vote memory newVote = Vote({
            proposalId: proposalId,
            voter: msg.sender,
            support: support,
            votingPower: votingPower,
            timestamp: block.timestamp,
            blockHeight: uint64(block.number)
        });

        votes[proposalId][msg.sender] = newVote;
        hasVoted[proposalId][msg.sender] = true;

        if (support) {
            proposals[proposalId].votesFor += votingPower;
        } else {
            proposals[proposalId].votesAgainst += votingPower;
        }

        emit VoteCast(proposalId, msg.sender, support, votingPower);
    }

    function finalizeProposal(uint256 proposalId) external {
        require(proposals[proposalId].id == proposalId, "Proposal does not exist");
        require(block.timestamp > proposals[proposalId].votingDeadline, "Voting period not ended");
        require(proposals[proposalId].status == ProposalStatus.Active, "Proposal already finalized");

        Proposal storage proposal = proposals[proposalId];
        uint256 totalVotes = proposal.votesFor + proposal.votesAgainst;
        
        bool passed = false;
        if (totalVotes >= proposal.quorumRequired && proposal.votesFor > proposal.votesAgainst) {
            passed = true;
            proposal.status = ProposalStatus.Passed;
        } else {
            proposal.status = ProposalStatus.Failed;
        }

        bool executed = false;
        if (passed && proposal.calldataToExecute.length > 0) {
            // Execute the proposal
            (bool success,) = address(this).call(proposal.calldataToExecute);
            if (success) {
                proposal.executed = true;
                proposal.status = ProposalStatus.Executed;
                executed = true;
            }
        }

        emit ProposalFinalized(proposalId, passed, executed);
    }

    function getProposal(uint256 proposalId) public view returns (Proposal memory) {
        require(proposals[proposalId].id == proposalId, "Proposal does not exist");
        return proposals[proposalId];
    }

    function calculateVotingPower(address voter) public view returns (uint256) {
        return karmaCore.getKarmaScore(voter);
    }

    function getActiveProposals() public view returns (Proposal[] memory) {
        uint256 activeCount = 0;
        
        // Count active proposals
        for (uint256 i = 1; i < nextProposalId; i++) {
            if (proposals[i].status == ProposalStatus.Active && 
                block.timestamp <= proposals[i].votingDeadline) {
                activeCount++;
            }
        }
        
        Proposal[] memory activeProposals = new Proposal[](activeCount);
        uint256 index = 0;
        
        // Populate active proposals
        for (uint256 i = 1; i < nextProposalId; i++) {
            if (proposals[i].status == ProposalStatus.Active && 
                block.timestamp <= proposals[i].votingDeadline) {
                activeProposals[index] = proposals[i];
                index++;
            }
        }
        
        return activeProposals;
    }

    function _calculateQuorum() internal pure returns (uint256) {
        // Simplified quorum calculation
        // In a real implementation, this would be based on total karma in the system
        return 1000; // Minimum 1000 karma points needed for quorum
    }

    function getVote(uint256 proposalId, address voter) public view returns (Vote memory) {
        require(hasVoted[proposalId][voter], "Vote not found");
        return votes[proposalId][voter];
    }
}

