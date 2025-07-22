//! Error types for Agent-Karma smart contracts
//! 
//! This module defines all the custom error types used across
//! the Agent-Karma smart contract ecosystem.

use cosmwasm_std::StdError;
use thiserror::Error;

/// Main error type for Agent-Karma contracts
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    /// Standard CosmWasm errors
    #[error("{0}")]
    Std(#[from] StdError),

    /// Agent Registry Errors
    #[error("Agent already registered: {address}")]
    AgentAlreadyRegistered { address: String },

    #[error("Agent not found: {address}")]
    AgentNotFound { address: String },

    #[error("Invalid agent metadata: {reason}")]
    InvalidAgentMetadata { reason: String },

    #[error("Unauthorized: only agent owner can perform this action")]
    UnauthorizedAgentUpdate,

    /// Karma Core Errors
    #[error("Invalid rating score: {score}. Must be between 1 and 10")]
    InvalidRatingScore { score: u8 },

    #[error("Rating already submitted for interaction: {interaction_hash}")]
    DuplicateRating { interaction_hash: String },

    #[error("Rating window expired for interaction: {interaction_hash}")]
    RatingWindowExpired { interaction_hash: String },

    #[error("Insufficient karma: required {required}, available {available}")]
    InsufficientKarma { required: u128, available: u128 },

    #[error("Cannot rate yourself")]
    SelfRatingNotAllowed,

    #[error("Karma calculation failed: {reason}")]
    KarmaCalculationError { reason: String },

    /// Interaction Logger Errors
    #[error("Invalid interaction: {reason}")]
    InvalidInteraction { reason: String },

    #[error("Interaction not found: {interaction_hash}")]
    InteractionNotFound { interaction_hash: String },

    #[error("Interaction verification failed: {interaction_hash}")]
    InteractionVerificationFailed { interaction_hash: String },

    #[error("Duplicate interaction hash: {interaction_hash}")]
    DuplicateInteractionHash { interaction_hash: String },

    /// Governance DAO Errors
    #[error("Proposal not found: {proposal_id}")]
    ProposalNotFound { proposal_id: u64 },

    #[error("Voting period has ended for proposal: {proposal_id}")]
    VotingPeriodEnded { proposal_id: u64 },

    #[error("Voting period has not ended for proposal: {proposal_id}")]
    VotingPeriodNotEnded { proposal_id: u64 },

    #[error("Already voted on proposal: {proposal_id}")]
    AlreadyVoted { proposal_id: u64 },

    #[error("Proposal already executed: {proposal_id}")]
    ProposalAlreadyExecuted { proposal_id: u64 },

    #[error("Proposal execution failed: {reason}")]
    ProposalExecutionFailed { reason: String },

    #[error("Quorum not reached for proposal: {proposal_id}")]
    QuorumNotReached { proposal_id: u64 },

    #[error("Invalid proposal: {reason}")]
    InvalidProposal { reason: String },

    /// Oracle Integration Errors
    #[error("Oracle data not found: {data_hash}")]
    OracleDataNotFound { data_hash: String },

    #[error("Oracle consensus not reached: {data_hash}")]
    OracleConsensusNotReached { data_hash: String },

    #[error("Invalid oracle signature: {provider}")]
    InvalidOracleSignature { provider: String },

    #[error("Oracle provider not authorized: {provider}")]
    UnauthorizedOracleProvider { provider: String },

    #[error("Oracle data already disputed: {data_hash}")]
    OracleDataAlreadyDisputed { data_hash: String },

    /// System Configuration Errors
    #[error("Invalid configuration: {key} = {value}")]
    InvalidConfiguration { key: String, value: String },

    #[error("Configuration not found: {key}")]
    ConfigurationNotFound { key: String },

    #[error("Unauthorized configuration update")]
    UnauthorizedConfigUpdate,

    /// Access Control Errors
    #[error("Unauthorized: caller does not have required permissions")]
    Unauthorized,

    #[error("Admin privileges required")]
    AdminRequired,

    #[error("Contract is paused")]
    ContractPaused,

    /// Validation Errors
    #[error("Invalid address format: {address}")]
    InvalidAddress { address: String },

    #[error("Invalid timestamp: {timestamp}")]
    InvalidTimestamp { timestamp: String },

    #[error("Invalid amount: {amount}")]
    InvalidAmount { amount: String },

    #[error("String too long: maximum {max_length} characters")]
    StringTooLong { max_length: usize },

    #[error("String too short: minimum {min_length} characters")]
    StringTooShort { min_length: usize },

    /// Rate Limiting Errors
    #[error("Rate limit exceeded: {action}")]
    RateLimitExceeded { action: String },

    #[error("Cooldown period active: {remaining_seconds} seconds remaining")]
    CooldownActive { remaining_seconds: u64 },

    /// Economic Errors
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: String, available: String },

    #[error("Fee payment failed: {reason}")]
    FeePaymentFailed { reason: String },

    #[error("Staking requirement not met: required {required}")]
    StakingRequirementNotMet { required: String },

    /// Integration Errors
    #[error("External service unavailable: {service}")]
    ExternalServiceUnavailable { service: String },

    #[error("Integration error: {framework} - {reason}")]
    IntegrationError { framework: String, reason: String },

    #[error("API version mismatch: expected {expected}, got {actual}")]
    ApiVersionMismatch { expected: String, actual: String },

    /// Performance Errors
    #[error("Operation timeout: {operation}")]
    OperationTimeout { operation: String },

    #[error("Response time exceeded: {actual_ms}ms > {limit_ms}ms")]
    ResponseTimeExceeded { actual_ms: u64, limit_ms: u64 },

    #[error("Resource limit exceeded: {resource}")]
    ResourceLimitExceeded { resource: String },

    /// Data Integrity Errors
    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Data corruption detected: {details}")]
    DataCorruption { details: String },

    /// Migration Errors
    #[error("Migration not supported from version {from} to {to}")]
    MigrationNotSupported { from: String, to: String },

    #[error("Migration failed: {reason}")]
    MigrationFailed { reason: String },
}

/// Result type alias for Agent-Karma contracts
pub type ContractResult<T> = Result<T, ContractError>;

/// Helper functions for common error scenarios
impl ContractError {
    /// Create an insufficient karma error
    pub fn insufficient_karma(required: u128, available: u128) -> Self {
        ContractError::InsufficientKarma { required, available }
    }

    /// Create an agent not found error
    pub fn agent_not_found(address: impl Into<String>) -> Self {
        ContractError::AgentNotFound {
            address: address.into(),
        }
    }

    /// Create an invalid rating score error
    pub fn invalid_rating_score(score: u8) -> Self {
        ContractError::InvalidRatingScore { score }
    }

    /// Create a proposal not found error
    pub fn proposal_not_found(proposal_id: u64) -> Self {
        ContractError::ProposalNotFound { proposal_id }
    }

    /// Create an interaction not found error
    pub fn interaction_not_found(interaction_hash: impl Into<String>) -> Self {
        ContractError::InteractionNotFound {
            interaction_hash: interaction_hash.into(),
        }
    }

    /// Create a rate limit exceeded error
    pub fn rate_limit_exceeded(action: impl Into<String>) -> Self {
        ContractError::RateLimitExceeded {
            action: action.into(),
        }
    }

    /// Create a response time exceeded error
    pub fn response_time_exceeded(actual_ms: u64, limit_ms: u64) -> Self {
        ContractError::ResponseTimeExceeded { actual_ms, limit_ms }
    }
}

/// Validation helper functions
pub mod validation {
    use super::ContractError;
    use cosmwasm_std::Addr;

    /// Validate that a string is within length limits
    pub fn validate_string_length(
        value: &str,
        min_length: usize,
        max_length: usize,
    ) -> Result<(), ContractError> {
        if value.len() < min_length {
            return Err(ContractError::StringTooShort { min_length });
        }
        if value.len() > max_length {
            return Err(ContractError::StringTooLong { max_length });
        }
        Ok(())
    }

    /// Validate that a rating score is within valid range (1-10)
    pub fn validate_rating_score(score: u8) -> Result<(), ContractError> {
        if score < 1 || score > 10 {
            return Err(ContractError::invalid_rating_score(score));
        }
        Ok(())
    }

    /// Validate that an address is not empty
    pub fn validate_address(addr: &Addr) -> Result<(), ContractError> {
        if addr.as_str().is_empty() {
            return Err(ContractError::InvalidAddress {
                address: addr.to_string(),
            });
        }
        Ok(())
    }

    /// Validate that participants list is not empty and contains valid addresses
    pub fn validate_participants(participants: &[Addr]) -> Result<(), ContractError> {
        if participants.is_empty() {
            return Err(ContractError::InvalidInteraction {
                reason: "Participants list cannot be empty".to_string(),
            });
        }

        for addr in participants {
            validate_address(addr)?;
        }

        Ok(())
    }

    /// Validate that karma amount is not zero
    pub fn validate_karma_amount(amount: u128) -> Result<(), ContractError> {
        if amount == 0 {
            return Err(ContractError::InvalidAmount {
                amount: amount.to_string(),
            });
        }
        Ok(())
    }
}