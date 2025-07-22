use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Admin required for this operation")]
    AdminRequired {},

    #[error("Interaction not found: {interaction_hash}")]
    InteractionNotFound { interaction_hash: String },

    #[error("Invalid interaction hash format")]
    InvalidInteractionHash {},

    #[error("Interaction already exists: {interaction_hash}")]
    InteractionAlreadyExists { interaction_hash: String },

    #[error("Invalid participants list: {reason}")]
    InvalidParticipants { reason: String },

    #[error("Participant not registered: {address}")]
    ParticipantNotRegistered { address: String },

    #[error("Storage operation failed after {attempts} attempts: {reason}")]
    StorageOperationFailed { attempts: u32, reason: String },

    #[error("Invalid interaction type: {interaction_type}")]
    InvalidInteractionType { interaction_type: String },

    #[error("Interaction verification failed: {reason}")]
    VerificationFailed { reason: String },

    #[error("Pagination limit exceeded: max {max}, requested {requested}")]
    PaginationLimitExceeded { max: u32, requested: u32 },

    #[error("Invalid timestamp: {reason}")]
    InvalidTimestamp { reason: String },

    #[error("Metadata validation failed: {reason}")]
    MetadataValidationFailed { reason: String },
}