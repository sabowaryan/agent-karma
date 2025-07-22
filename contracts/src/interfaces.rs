//! Smart contract interfaces for Agent-Karma system
//! 
//! This module defines the core traits that each smart contract
//! must implement to ensure consistent behavior across the system.

use cosmwasm_std::{Addr, Response, StdResult, Timestamp, Uint128};
use crate::types::*;

/// Core interface for Agent Registry contract
/// 
/// Manages agent registration and identity verification
pub trait IAgentRegistry {
    /// Register a new agent in the system
    /// 
    /// # Arguments
    /// * `agent_address` - Blockchain address of the agent
    /// * `metadata` - Agent metadata including name, description, framework
    /// 
    /// # Returns
    /// * `StdResult<Response>` - Success response or error
    /// 
    /// # Errors
    /// * Returns error if agent is already registered
    /// * Returns error if metadata is invalid
    fn register_agent(
        &self,
        agent_address: Addr,
        metadata: AgentMetadata,
    ) -> StdResult<Response>;

    /// Retrieve agent information
    /// 
    /// # Arguments
    /// * `agent_address` - Address of the agent to query
    /// 
    /// # Returns
    /// * `StdResult<Option<Agent>>` - Agent data if found, None otherwise
    fn get_agent_info(&self, agent_address: Addr) -> StdResult<Option<Agent>>;

    /// Check if an agent is registered
    /// 
    /// # Arguments
    /// * `agent_address` - Address to check
    /// 
    /// # Returns
    /// * `StdResult<bool>` - true if registered, false otherwise
    fn is_registered_agent(&self, agent_address: Addr) -> StdResult<bool>;

    /// Update agent metadata
    /// 
    /// # Arguments
    /// * `agent_address` - Address of the agent
    /// * `metadata` - New metadata to set
    /// 
    /// # Returns
    /// * `StdResult<Response>` - Success response or error
    /// 
    /// # Errors
    /// * Returns error if agent is not registered
    /// * Returns error if caller is not the agent owner
    fn update_agent_metadata(
        &self,
        agent_address: Addr,
        metadata: AgentMetadata,
    ) -> StdResult<Response>;
}

/// Core interface for Karma calculation and management
/// 
/// Handles karma scoring, rating submission, and score queries
pub trait IKarmaCore {
    /// Submit a rating for another agent
    /// 
    /// # Arguments
    /// * `rater` - Address of the agent giving the rating
    /// * `rated_agent` - Address of the agent being rated
    /// * `score` - Rating score (1-10)
    /// * `feedback` - Optional feedback text
    /// * `interaction_hash` - Hash of the interaction being rated
    /// 
    /// # Returns
    /// * `StdResult<Response>` - Success response or error
    /// 
    /// # Errors
    /// * Returns error if rating is outside valid range (1-10)
    /// * Returns error if interaction has already been rated by this agent
    /// * Returns error if rating window has expired
    fn submit_rating(
        &self,
        rater: Addr,
        rated_agent: Addr,
        score: u8,
        feedback: Option<String>,
        interaction_hash: String,
    ) -> StdResult<Response>;

    /// Calculate karma score for an agent
    /// 
    /// # Arguments
    /// * `agent_address` - Address of the agent
    /// 
    /// # Returns
    /// * `StdResult<KarmaCalculation>` - Detailed karma calculation
    /// 
    /// # Errors
    /// * Returns error if agent is not registered
    fn calculate_karma(&self, agent_address: Addr) -> StdResult<KarmaCalculation>;

    /// Get current karma score for an agent
    /// 
    /// # Arguments
    /// * `agent_address` - Address of the agent
    /// 
    /// # Returns
    /// * `StdResult<Uint128>` - Current karma score
    fn get_karma_score(&self, agent_address: Addr) -> StdResult<Uint128>;

    /// Get karma history for an agent
    /// 
    /// # Arguments
    /// * `agent_address` - Address of the agent
    /// * `limit` - Maximum number of records to return
    /// 
    /// # Returns
    /// * `StdResult<Vec<KarmaCalculation>>` - Historical karma data
    fn get_karma_history(
        &self,
        agent_address: Addr,
        limit: Option<u32>,
    ) -> StdResult<Vec<KarmaCalculation>>;

    /// Get all ratings for an agent
    /// 
    /// # Arguments
    /// * `agent_address` - Address of the agent
    /// * `limit` - Maximum number of ratings to return
    /// 
    /// # Returns
    /// * `StdResult<Vec<Rating>>` - List of ratings received
    fn get_agent_ratings(
        &self,
        agent_address: Addr,
        limit: Option<u32>,
    ) -> StdResult<Vec<Rating>>;
}

/// Interface for interaction logging and audit trails
/// 
/// Records all agent interactions for transparency and verification
pub trait IInteractionLogger {
    /// Log an interaction between agents
    /// 
    /// # Arguments
    /// * `participants` - Addresses of agents involved
    /// * `interaction_type` - Type of interaction
    /// * `metadata` - Additional interaction data
    /// 
    /// # Returns
    /// * `StdResult<String>` - Unique interaction ID
    /// 
    /// # Errors
    /// * Returns error if any participant is not registered
    fn log_interaction(
        &self,
        participants: Vec<Addr>,
        interaction_type: String,
        metadata: InteractionMetadata,
    ) -> StdResult<String>;

    /// Retrieve interaction history for an agent
    /// 
    /// # Arguments
    /// * `agent_address` - Address of the agent
    /// * `limit` - Maximum number of interactions to return
    /// 
    /// # Returns
    /// * `StdResult<Vec<Interaction>>` - List of interactions
    fn get_interaction_history(
        &self,
        agent_address: Addr,
        limit: Option<u32>,
    ) -> StdResult<Vec<Interaction>>;

    /// Verify an interaction exists and is valid
    /// 
    /// # Arguments
    /// * `interaction_hash` - Hash of the interaction to verify
    /// 
    /// # Returns
    /// * `StdResult<bool>` - true if interaction is valid
    fn verify_interaction(&self, interaction_hash: String) -> StdResult<bool>;

    /// Get interaction details by hash
    /// 
    /// # Arguments
    /// * `interaction_hash` - Hash of the interaction
    /// 
    /// # Returns
    /// * `StdResult<Option<Interaction>>` - Interaction data if found
    fn get_interaction_by_hash(
        &self,
        interaction_hash: String,
    ) -> StdResult<Option<Interaction>>;
}

/// Interface for decentralized governance
/// 
/// Manages proposals, voting, and governance execution
pub trait IGovernanceDAO {
    /// Create a new governance proposal
    /// 
    /// # Arguments
    /// * `proposer` - Address of the proposer
    /// * `title` - Title of the proposal
    /// * `description` - Detailed description
    /// * `calldata` - Encoded function call to execute
    /// * `voting_period` - Duration of voting period in seconds
    /// 
    /// # Returns
    /// * `StdResult<u64>` - Unique proposal ID
    /// 
    /// # Errors
    /// * Returns error if proposer doesn't have minimum karma
    /// * Returns error if proposal data is invalid
    fn create_proposal(
        &self,
        proposer: Addr,
        title: String,
        description: String,
        calldata: String,
        voting_period: u64,
    ) -> StdResult<u64>;

    /// Vote on a governance proposal
    /// 
    /// # Arguments
    /// * `voter` - Address of the voter
    /// * `proposal_id` - ID of the proposal
    /// * `support` - true for yes, false for no
    /// 
    /// # Returns
    /// * `StdResult<Response>` - Success response or error
    /// 
    /// # Errors
    /// * Returns error if voter doesn't have minimum karma
    /// * Returns error if proposal doesn't exist or voting has ended
    /// * Returns error if voter has already voted
    fn vote_proposal(
        &self,
        voter: Addr,
        proposal_id: u64,
        support: bool,
    ) -> StdResult<Response>;

    /// Finalize a proposal after voting period ends
    /// 
    /// # Arguments
    /// * `proposal_id` - ID of the proposal to finalize
    /// 
    /// # Returns
    /// * `StdResult<Response>` - Success response or error
    /// 
    /// # Errors
    /// * Returns error if voting period hasn't ended
    /// * Returns error if proposal has already been finalized
    fn finalize_proposal(&self, proposal_id: u64) -> StdResult<Response>;

    /// Get proposal details
    /// 
    /// # Arguments
    /// * `proposal_id` - ID of the proposal
    /// 
    /// # Returns
    /// * `StdResult<Option<Proposal>>` - Proposal data if found
    fn get_proposal(&self, proposal_id: u64) -> StdResult<Option<Proposal>>;

    /// Calculate voting power for an agent
    /// 
    /// # Arguments
    /// * `voter` - Address of the voter
    /// 
    /// # Returns
    /// * `StdResult<Uint128>` - Karma-based voting power
    fn calculate_voting_power(&self, voter: Addr) -> StdResult<Uint128>;

    /// Get all active proposals
    /// 
    /// # Returns
    /// * `StdResult<Vec<Proposal>>` - List of active proposals
    fn get_active_proposals(&self) -> StdResult<Vec<Proposal>>;
}

/// Interface for oracle data integration
/// 
/// Manages external data sources and validation
pub trait IOracleIntegration {
    /// Submit external data from oracle provider
    /// 
    /// # Arguments
    /// * `provider` - Address of the oracle provider
    /// * `data_type` - Type of data being provided
    /// * `data` - The actual data payload
    /// * `signatures` - Validator signatures
    /// 
    /// # Returns
    /// * `StdResult<Response>` - Success response or error
    fn submit_oracle_data(
        &self,
        provider: Addr,
        data_type: String,
        data: String,
        signatures: Vec<String>,
    ) -> StdResult<Response>;

    /// Verify oracle data consensus
    /// 
    /// # Arguments
    /// * `data_hash` - Hash of the data to verify
    /// 
    /// # Returns
    /// * `StdResult<bool>` - true if consensus is reached
    fn verify_oracle_consensus(&self, data_hash: String) -> StdResult<bool>;

    /// Get verified oracle data
    /// 
    /// # Arguments
    /// * `data_type` - Type of data to retrieve
    /// * `timestamp` - Optional timestamp filter
    /// 
    /// # Returns
    /// * `StdResult<Vec<OracleData>>` - List of verified oracle data
    fn get_oracle_data(
        &self,
        data_type: String,
        timestamp: Option<Timestamp>,
    ) -> StdResult<Vec<OracleData>>;
}