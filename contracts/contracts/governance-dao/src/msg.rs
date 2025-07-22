use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use agent_karma_contracts::{Proposal, Vote};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Minimum karma required to create proposals
    pub min_karma_for_proposal: Uint128,
    /// Minimum karma required to vote
    pub min_karma_for_voting: Uint128,
    /// Default voting period in seconds
    pub default_voting_period: u64,
    /// Quorum threshold as percentage (0-100)
    pub quorum_threshold: u8,
    /// Execution delay in seconds
    pub execution_delay: u64,
    /// Address of the karma core contract
    pub karma_core_address: String,
    /// Address of the agent registry contract
    pub agent_registry_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Create a new governance proposal
    CreateProposal {
        title: String,
        description: String,
        calldata: String,
        voting_period: Option<u64>,
    },
    /// Vote on a proposal
    VoteProposal {
        proposal_id: u64,
        support: bool,
    },
    /// Finalize a proposal after voting period
    FinalizeProposal {
        proposal_id: u64,
    },
    /// Execute a passed proposal
    ExecuteProposal {
        proposal_id: u64,
    },
    /// Cancel a proposal (proposer or admin only)
    CancelProposal {
        proposal_id: u64,
    },
    /// Update governance configuration (admin only)
    UpdateConfig {
        min_karma_for_proposal: Option<Uint128>,
        min_karma_for_voting: Option<Uint128>,
        default_voting_period: Option<u64>,
        quorum_threshold: Option<u8>,
        execution_delay: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Get proposal details
    GetProposal {
        proposal_id: u64,
    },
    /// Get all active proposals
    GetActiveProposals {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    /// Get all proposals (including inactive)
    GetAllProposals {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    /// Get voting power for an agent
    GetVotingPower {
        agent_address: String,
    },
    /// Get vote details for a proposal
    GetVote {
        proposal_id: u64,
        voter: String,
    },
    /// Get all votes for a proposal
    GetProposalVotes {
        proposal_id: u64,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Get governance configuration
    GetConfig {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProposalResponse {
    pub proposal: Option<Proposal>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProposalsResponse {
    pub proposals: Vec<Proposal>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VotingPowerResponse {
    pub voting_power: Uint128,
    pub karma_score: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoteResponse {
    pub vote: Option<Vote>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VotesResponse {
    pub votes: Vec<Vote>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GovernanceConfig {
    pub min_karma_for_proposal: Uint128,
    pub min_karma_for_voting: Uint128,
    pub default_voting_period: u64,
    pub quorum_threshold: u8,
    pub execution_delay: u64,
    pub karma_core_address: Addr,
    pub agent_registry_address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub config: GovernanceConfig,
}