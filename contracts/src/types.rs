//! Core data types for Agent-Karma smart contracts
//! 
//! This module defines all the shared data structures used across
//! the Agent-Karma smart contract ecosystem.

use cosmwasm_std::{Addr, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents an AI agent registered in the system
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Agent {
    /// Blockchain address of the agent
    pub address: Addr,
    /// Timestamp when the agent was registered
    pub registration_date: Timestamp,
    /// Agent metadata information
    pub metadata: AgentMetadata,
    /// Current karma score
    pub karma_score: Uint128,
    /// Total number of interactions
    pub interaction_count: u64,
    /// Total number of ratings received
    pub ratings_received: u64,
}

/// Metadata associated with an agent
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AgentMetadata {
    /// Display name of the agent
    pub name: String,
    /// Description of the agent's purpose
    pub description: String,
    /// AI framework used (ElizaOS, MCP, AIDN, etc.)
    pub framework: String,
    /// Version of the agent
    pub version: String,
    /// Optional IPFS hash for extended metadata
    pub ipfs_hash: Option<String>,
}

/// Represents a rating given by one agent to another
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Rating {
    /// Unique identifier for the rating
    pub id: String,
    /// Address of the agent giving the rating
    pub rater_address: Addr,
    /// Address of the agent being rated
    pub rated_address: Addr,
    /// Rating score (1-10)
    pub score: u8,
    /// Optional feedback text
    pub feedback: Option<String>,
    /// Hash of the interaction this rating refers to
    pub interaction_hash: String,
    /// Timestamp when the rating was submitted
    pub timestamp: Timestamp,
    /// Sei blockchain block height
    pub block_height: u64,
}

/// Represents an interaction between agents
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Interaction {
    /// Unique identifier for the interaction
    pub id: String,
    /// Addresses of agents involved in the interaction
    pub participants: Vec<Addr>,
    /// Type of interaction (e.g., "conversation", "task", "collaboration")
    pub interaction_type: String,
    /// Timestamp when the interaction occurred
    pub timestamp: Timestamp,
    /// Sei blockchain block height
    pub block_height: u64,
    /// Additional metadata about the interaction
    pub metadata: InteractionMetadata,
}

/// Metadata for interactions
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InteractionMetadata {
    /// Duration of the interaction in seconds
    pub duration: Option<u64>,
    /// Outcome of the interaction
    pub outcome: Option<String>,
    /// Context or additional information
    pub context: Option<String>,
}

/// Karma calculation details
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct KarmaCalculation {
    /// Address of the agent
    pub agent_address: Addr,
    /// Current karma score
    pub current_score: Uint128,
    /// Previous karma score
    pub previous_score: Uint128,
    /// Factors that contributed to the calculation
    pub factors: KarmaFactors,
    /// Timestamp of last update
    pub last_updated: Timestamp,
    /// Hash for verification purposes
    pub calculation_hash: String,
}

/// Factors used in karma calculation
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct KarmaFactors {
    /// Average rating received
    pub average_rating: String, // Using String to avoid floating point issues
    /// Total number of ratings
    pub rating_count: u64,
    /// Interaction frequency score
    pub interaction_frequency: Uint128,
    /// Time decay factor
    pub time_decay: String,
    /// External factors from oracles
    pub external_factors: Option<Uint128>,
}

/// Governance proposal
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Proposal {
    /// Unique proposal ID
    pub id: u64,
    /// Title of the proposal
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Address of the proposer
    pub proposer: Addr,
    /// Encoded function call to execute if passed
    pub calldata: String,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Voting deadline
    pub voting_deadline: Timestamp,
    /// Whether the proposal has been executed
    pub executed: bool,
    /// Karma-weighted votes in favor
    pub votes_for: Uint128,
    /// Karma-weighted votes against
    pub votes_against: Uint128,
    /// Minimum karma required for quorum
    pub quorum_required: Uint128,
    /// Current status of the proposal
    pub status: ProposalStatus,
}

/// Status of a governance proposal
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ProposalStatus {
    Active,
    Passed,
    Failed,
    Executed,
}

/// Vote on a governance proposal
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Vote {
    /// ID of the proposal being voted on
    pub proposal_id: u64,
    /// Address of the voter
    pub voter: Addr,
    /// Support for the proposal (true = yes, false = no)
    pub support: bool,
    /// Voting power at the time of vote
    pub voting_power: Uint128,
    /// Timestamp of the vote
    pub timestamp: Timestamp,
    /// Block height when vote was cast
    pub block_height: u64,
}

/// Oracle data submission
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleData {
    /// Address of the oracle provider
    pub provider: Addr,
    /// Type of data being provided
    pub data_type: String,
    /// The actual data payload
    pub data: String,
    /// Timestamp of data submission
    pub timestamp: Timestamp,
    /// Signatures from validator nodes
    pub signatures: Vec<String>,
    /// Whether the data has been verified
    pub verified: bool,
}

/// Configuration for karma calculation parameters
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct KarmaConfig {
    /// Minimum karma required for rating submission
    pub min_karma_for_rating: Uint128,
    /// Minimum karma required for governance voting
    pub min_karma_for_voting: Uint128,
    /// Minimum karma required for proposal creation
    pub min_karma_for_proposal: Uint128,
    /// Time window for rating submission after interaction (in seconds)
    pub rating_window: u64,
    /// Maximum number of ratings per interaction
    pub max_ratings_per_interaction: u8,
    /// Karma fee for rating submission
    pub rating_fee: Uint128,
}