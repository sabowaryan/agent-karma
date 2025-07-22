//! Error types for Oracle Integration contract

use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid oracle provider: {provider}")]
    InvalidOracleProvider { provider: String },

    #[error("Insufficient signatures: got {got}, required {required}")]
    InsufficientSignatures { got: u32, required: u32 },

    #[error("Invalid signature: {signature}")]
    InvalidSignature { signature: String },

    #[error("Oracle data not found: {data_hash}")]
    OracleDataNotFound { data_hash: String },

    #[error("Oracle data already exists: {data_hash}")]
    OracleDataAlreadyExists { data_hash: String },

    #[error("Consensus not reached: {current}/{required}")]
    ConsensusNotReached { current: u32, required: u32 },

    #[error("Insufficient stake: got {got}, required {required}")]
    InsufficientStake { got: String, required: String },

    #[error("Dispute already exists for data: {data_hash}")]
    DisputeAlreadyExists { data_hash: String },

    #[error("Dispute not found: {data_hash}")]
    DisputeNotFound { data_hash: String },

    #[error("Dispute already resolved: {data_hash}")]
    DisputeAlreadyResolved { data_hash: String },

    #[error("Invalid data type: {data_type}")]
    InvalidDataType { data_type: String },

    #[error("Data verification failed: {reason}")]
    DataVerificationFailed { reason: String },

    #[error("Oracle provider already exists: {provider}")]
    OracleProviderAlreadyExists { provider: String },

    #[error("Oracle provider not found: {provider}")]
    OracleProviderNotFound { provider: String },

    #[error("Minimum oracle providers required: {required}")]
    MinimumOracleProvidersRequired { required: u32 },
}