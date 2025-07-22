use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Agent already registered: {address}")]
    AgentAlreadyRegistered { address: String },

    #[error("Agent not found: {address}")]
    AgentNotFound { address: String },

    #[error("Invalid metadata: {reason}")]
    InvalidMetadata { reason: String },

    #[error("Invalid agent address: {address}")]
    InvalidAgentAddress { address: String },

    #[error("Agent name too long: maximum 64 characters")]
    AgentNameTooLong {},

    #[error("Agent description too long: maximum 512 characters")]
    AgentDescriptionTooLong {},

    #[error("Invalid framework name: {framework}")]
    InvalidFramework { framework: String },

    #[error("Invalid IPFS hash format: {hash}")]
    InvalidIpfsHash { hash: String },

    #[error("Agent is deactivated: {address}")]
    AgentDeactivated { address: String },

    #[error("Only agent owner can update metadata")]
    OnlyOwnerCanUpdate {},

    #[error("Admin privileges required")]
    AdminRequired {},
}