use cosmwasm_std::{StdError, OverflowError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Agent not found: {address}")]
    AgentNotFound { address: String },

    #[error("Invalid rating score: {score}. Must be between 1 and 10")]
    InvalidRatingScore { score: u8 },

    #[error("Rating already submitted for interaction: {interaction_hash}")]
    RatingAlreadySubmitted { interaction_hash: String },

    #[error("Rating window expired for interaction: {interaction_hash}")]
    RatingWindowExpired { interaction_hash: String },

    #[error("Interaction not found: {interaction_hash}")]
    InteractionNotFound { interaction_hash: String },

    #[error("Cannot rate yourself")]
    CannotRateSelf {},

    #[error("Insufficient karma: required {required}, current {current}")]
    InsufficientKarma { required: u128, current: u128 },

    #[error("Invalid karma configuration: {reason}")]
    InvalidKarmaConfig { reason: String },

    #[error("Karma calculation failed: {reason}")]
    KarmaCalculationFailed { reason: String },

    #[error("Oracle data verification failed")]
    OracleDataVerificationFailed {},

    #[error("Invalid time decay factor: {factor}")]
    InvalidTimeDecayFactor { factor: String },

    #[error("Admin required for this operation")]
    AdminRequired {},

    #[error("Rating fee payment required: {amount}")]
    RatingFeeRequired { amount: u128 },

    #[error("Duplicate prevention check failed")]
    DuplicatePreventionFailed {},

    #[error("Historical tracking error: {reason}")]
    HistoricalTrackingError { reason: String },

    #[error("Minimum requirements not met: {reason}")]
    MinimumRequirementsNotMet { reason: String },

    // Compliance and abuse detection errors
    #[error("Abuse detected: {violation_type}")]
    AbuseDetected { violation_type: String },

    #[error("Rate limit exceeded for action: {action}")]
    RateLimitExceeded { action: String },

    #[error("Compliance violation: {reason}")]
    ComplianceViolation { reason: String },

    #[error("Dispute case not found: {case_id}")]
    DisputeCaseNotFound { case_id: String },

    #[error("Dispute already resolved: {case_id}")]
    DisputeAlreadyResolved { case_id: String },

    #[error("Insufficient stake for dispute: required {required}, available {available}")]
    InsufficientStake { required: u128, available: u128 },

    #[error("Spam detection triggered: {evidence}")]
    SpamDetected { evidence: String },

    #[error("Bot behavior detected: {pattern}")]
    BotBehaviorDetected { pattern: String },

    #[error("Rating manipulation detected: {details}")]
    RatingManipulationDetected { details: String },

    #[error("Penalty application failed: {reason}")]
    PenaltyApplicationFailed { reason: String },
}