//! CosmWasm message structures for Agent-Karma smart contracts
//! 
//! This module defines all the message types used for contract instantiation,
//! execution, and queries across the Agent-Karma ecosystem.

use cosmwasm_std::{Addr, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::types::*;

/// Common instantiation message for all contracts
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Administrator address for the contract
    pub admin: Option<String>,
    /// Initial configuration parameters
    pub config: Option<KarmaConfig>,
}

/// Agent Registry Messages
pub mod agent_registry {
    use super::*;

    /// Execute messages for Agent Registry contract
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        /// Register a new agent
        RegisterAgent {
            metadata: AgentMetadata,
        },
        /// Update agent metadata (only by agent owner)
        UpdateAgentMetadata {
            metadata: AgentMetadata,
        },
        /// Deactivate an agent (admin only)
        DeactivateAgent {
            agent_address: String,
        },
    }

    /// Query messages for Agent Registry contract
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        /// Get agent information
        GetAgent {
            agent_address: String,
        },
        /// Check if agent is registered
        IsRegistered {
            agent_address: String,
        },
        /// Get all registered agents (paginated)
        GetAllAgents {
            start_after: Option<String>,
            limit: Option<u32>,
        },
        /// Get agents by framework
        GetAgentsByFramework {
            framework: String,
            start_after: Option<String>,
            limit: Option<u32>,
        },
    }

    /// Response types for Agent Registry queries
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct AgentResponse {
        pub agent: Option<Agent>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct IsRegisteredResponse {
        pub registered: bool,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct AgentsResponse {
        pub agents: Vec<Agent>,
    }
}

/// Karma Core Messages
pub mod karma_core {
    use super::*;

    /// Execute messages for Karma Core contract
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        /// Submit a rating for another agent
        SubmitRating {
            rated_agent: String,
            score: u8,
            feedback: Option<String>,
            interaction_hash: String,
        },
        /// Recalculate karma for an agent (can be called by anyone)
        RecalculateKarma {
            agent_address: String,
        },
        /// Update karma configuration (admin only)
        UpdateConfig {
            config: KarmaConfig,
        },
        /// Process oracle data for karma calculation
        ProcessOracleData {
            agent_address: String,
            oracle_data: Vec<OracleData>,
        },
        /// Run abuse detection on an agent
        RunAbuseDetection {
            agent_address: String,
        },
        /// Apply penalty for compliance violation
        ApplyCompliancePenalty {
            agent_address: String,
            violation_type: String,
            severity: u8,
            evidence: String,
        },
        /// Create dispute for false positive detection
        CreateDispute {
            violation_id: String,
            stake_amount: Uint128,
            evidence: String,
        },
        /// Resolve dispute case (admin only)
        ResolveDispute {
            case_id: String,
            resolution: String, // "confirmed", "overturned", "partial"
        },
    }

    /// Query messages for Karma Core contract
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        /// Get current karma score
        GetKarmaScore {
            agent_address: String,
        },
        /// Get detailed karma calculation
        GetKarmaCalculation {
            agent_address: String,
        },
        /// Get karma history
        GetKarmaHistory {
            agent_address: String,
            start_after: Option<Timestamp>,
            limit: Option<u32>,
        },
        /// Get ratings for an agent
        GetAgentRatings {
            agent_address: String,
            start_after: Option<String>,
            limit: Option<u32>,
        },
        /// Get leaderboard (top karma scores)
        GetLeaderboard {
            limit: Option<u32>,
        },
        /// Get karma configuration
        GetConfig {},
        /// Get compliance violations for an agent
        GetComplianceViolations {
            agent_address: String,
            start_after: Option<String>,
            limit: Option<u32>,
        },
        /// Get dispute cases
        GetDisputeCases {
            status: Option<String>, // "pending", "resolved", etc.
            start_after: Option<String>,
            limit: Option<u32>,
        },
        /// Get abuse detection results for an agent
        GetAbuseDetectionResults {
            agent_address: String,
        },
        /// Get rate limit status for an agent
        GetRateLimitStatus {
            agent_address: String,
            action_type: String,
        },
    }

    /// Response types for Karma Core queries
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct KarmaScoreResponse {
        pub score: Uint128,
        pub last_updated: Timestamp,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct KarmaCalculationResponse {
        pub calculation: KarmaCalculation,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct KarmaHistoryResponse {
        pub history: Vec<KarmaCalculation>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct RatingsResponse {
        pub ratings: Vec<Rating>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct LeaderboardEntry {
        pub agent_address: Addr,
        pub karma_score: Uint128,
        pub agent_name: String,
        pub framework: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct LeaderboardResponse {
        pub leaderboard: Vec<LeaderboardEntry>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct ConfigResponse {
        pub config: KarmaConfig,
    }

    /// Response types for compliance queries
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct ComplianceViolation {
        pub agent_address: Addr,
        pub violation_type: String,
        pub severity: u8,
        pub timestamp: Timestamp,
        pub evidence: String,
        pub penalty_applied: Uint128,
        pub disputed: bool,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct ComplianceViolationsResponse {
        pub violations: Vec<ComplianceViolation>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct DisputeCase {
        pub case_id: String,
        pub violation_id: String,
        pub challenger: Addr,
        pub stake_amount: Uint128,
        pub evidence: String,
        pub status: String,
        pub created_at: Timestamp,
        pub resolved_at: Option<Timestamp>,
        pub resolution: Option<String>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct DisputeCasesResponse {
        pub cases: Vec<DisputeCase>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct AbuseDetectionResult {
        pub is_suspicious: bool,
        pub violation_type: Option<String>,
        pub confidence_score: String, // Using string to avoid floating point
        pub evidence: Vec<String>,
        pub recommended_penalty: Uint128,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct AbuseDetectionResponse {
        pub results: Vec<AbuseDetectionResult>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct RateLimitStatus {
        pub agent_address: Addr,
        pub action_type: String,
        pub current_count: u32,
        pub limit: u32,
        pub window_start: Timestamp,
        pub window_end: Timestamp,
        pub remaining_actions: u32,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct RateLimitStatusResponse {
        pub status: RateLimitStatus,
    }
}

/// Interaction Logger Messages
pub mod interaction_logger {
    use super::*;

    /// Execute messages for Interaction Logger contract
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        /// Log a new interaction
        LogInteraction {
            participants: Vec<String>,
            interaction_type: String,
            metadata: InteractionMetadata,
        },
        /// Verify an interaction (for audit purposes)
        VerifyInteraction {
            interaction_hash: String,
        },
        /// Update interaction metadata (participants only)
        UpdateInteractionMetadata {
            interaction_hash: String,
            metadata: InteractionMetadata,
        },
    }

    /// Query messages for Interaction Logger contract
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        /// Get interaction by hash
        GetInteraction {
            interaction_hash: String,
        },
        /// Get interaction history for an agent
        GetInteractionHistory {
            agent_address: String,
            start_after: Option<Timestamp>,
            limit: Option<u32>,
        },
        /// Get interactions between specific agents
        GetInteractionsBetween {
            agent1: String,
            agent2: String,
            start_after: Option<Timestamp>,
            limit: Option<u32>,
        },
        /// Get recent interactions (global feed)
        GetRecentInteractions {
            start_after: Option<Timestamp>,
            limit: Option<u32>,
        },
        /// Verify interaction exists and is valid
        VerifyInteractionExists {
            interaction_hash: String,
        },
    }

    /// Response types for Interaction Logger queries
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InteractionResponse {
        pub interaction: Option<Interaction>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InteractionsResponse {
        pub interactions: Vec<Interaction>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct VerificationResponse {
        pub verified: bool,
        pub details: Option<String>,
    }
}

/// Governance DAO Messages
pub mod governance_dao {
    use super::*;

    /// Execute messages for Governance DAO contract
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        /// Create a new governance proposal
        CreateProposal {
            title: String,
            description: String,
            calldata: String,
            voting_period: u64,
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
    }

    /// Query messages for Governance DAO contract
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
        GetGovernanceConfig {},
    }

    /// Response types for Governance DAO queries
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
        pub quorum_threshold: String, // Percentage as string to avoid floating point
        pub execution_delay: u64,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct GovernanceConfigResponse {
        pub config: GovernanceConfig,
    }
}

/// Oracle Integration Messages
pub mod oracle_integration {
    use super::*;

    /// Oracle signature structure
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct OracleSignature {
        /// Address of the oracle provider
        pub provider: String,
        /// Cryptographic signature
        pub signature: String,
        /// Public key used for verification
        pub public_key: String,
    }

    /// Execute messages for Oracle Integration contract
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        /// Submit oracle data with multi-signature validation
        SubmitOracleData {
            data_type: String,
            data: String,
            signatures: Vec<OracleSignature>,
        },
        /// Dispute oracle data by staking karma
        DisputeOracleData {
            data_hash: String,
            stake_amount: Uint128,
            evidence: String,
        },
        /// Resolve oracle dispute (admin only)
        ResolveDispute {
            data_hash: String,
            resolution: bool, // true if data is valid, false if invalid
            resolution_reason: String,
        },
        /// Add oracle provider (admin only)
        AddOracleProvider {
            provider: String,
            public_key: String,
        },
        /// Remove oracle provider (admin only)
        RemoveOracleProvider {
            provider: String,
        },
        /// Update contract configuration (admin only)
        UpdateConfig {
            min_signatures: Option<u32>,
            min_dispute_stake: Option<Uint128>,
        },
    }

    /// Query messages for Oracle Integration contract
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        /// Get oracle data by hash
        GetOracleData {
            data_hash: String,
        },
        /// Get oracle data by type and time range
        GetOracleDataByType {
            data_type: String,
            start_time: Option<Timestamp>,
            end_time: Option<Timestamp>,
            limit: Option<u32>,
        },
        /// Get all oracle providers
        GetOracleProviders {},
        /// Check if data has reached consensus
        CheckConsensus {
            data_hash: String,
        },
        /// Get disputed data
        GetDisputedData {
            start_after: Option<String>,
            limit: Option<u32>,
        },
        /// Get contract configuration
        GetConfig {},
        /// Get oracle data for karma calculation
        GetKarmaOracleData {
            agent_address: String,
            data_types: Vec<String>, // performance, cross_chain, sentiment
        },
    }

    /// Oracle data entry
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct OracleDataEntry {
        /// Hash of the data for identification
        pub data_hash: String,
        /// Address of the submitting provider
        pub provider: Addr,
        /// Type of data (performance, cross_chain, sentiment)
        pub data_type: String,
        /// The actual data payload (JSON string)
        pub data: String,
        /// Timestamp of data submission
        pub timestamp: Timestamp,
        /// Signatures from validator nodes
        pub signatures: Vec<OracleSignature>,
        /// Whether consensus has been reached
        pub consensus_reached: bool,
        /// Number of valid signatures
        pub signature_count: u32,
        /// Whether the data has been disputed
        pub disputed: bool,
    }

    /// Dispute information
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct DisputeInfo {
        /// Hash of the disputed data
        pub data_hash: String,
        /// Address of the challenger
        pub challenger: Addr,
        /// Amount staked for the dispute
        pub stake_amount: Uint128,
        /// Evidence provided by challenger
        pub evidence: String,
        /// Timestamp when dispute was created
        pub created_at: Timestamp,
        /// Whether the dispute has been resolved
        pub resolved: bool,
        /// Resolution timestamp
        pub resolved_at: Option<Timestamp>,
        /// Resolution result (true = data valid, false = data invalid)
        pub resolution: Option<bool>,
        /// Reason for resolution
        pub resolution_reason: Option<String>,
    }

    /// Oracle provider information
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct OracleProvider {
        /// Address of the provider
        pub address: Addr,
        /// Public key for signature verification
        pub public_key: String,
        /// Whether the provider is active
        pub active: bool,
        /// Timestamp when provider was added
        pub added_at: Timestamp,
    }

    /// Response types for Oracle Integration queries
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct OracleDataResponse {
        pub data: Option<OracleDataEntry>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct OracleDataListResponse {
        pub data: Vec<OracleDataEntry>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct OracleProvidersResponse {
        pub providers: Vec<OracleProvider>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct ConsensusResponse {
        pub consensus_reached: bool,
        pub signature_count: u32,
        pub required_signatures: u32,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct DisputedDataResponse {
        pub disputes: Vec<DisputeInfo>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct ConfigResponse {
        pub config: OracleConfig,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct OracleConfig {
        /// Administrator address
        pub admin: Addr,
        /// Minimum signatures required for consensus
        pub min_signatures: u32,
        /// Minimum stake required for disputes
        pub min_dispute_stake: Uint128,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct KarmaOracleDataResponse {
        pub performance_data: Option<String>,
        pub cross_chain_data: Option<String>,
        pub sentiment_data: Option<String>,
        pub performance_weight: String, // "15" for 15%
        pub cross_chain_weight: String, // "10" for 10%
        pub sentiment_weight: String,   // "5" for 5%
    }
}

/// Migration messages for contract upgrades
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    /// Version being migrated to
    pub version: String,
    /// Optional migration parameters
    pub params: Option<serde_json::Value>,
}