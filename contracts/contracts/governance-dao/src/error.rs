use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Insufficient karma: required {required}, have {current}")]
    InsufficientKarma { required: u128, current: u128 },

    #[error("Proposal not found: {id}")]
    ProposalNotFound { id: u64 },

    #[error("Voting period has ended")]
    VotingPeriodEnded {},

    #[error("Voting period has not ended")]
    VotingPeriodNotEnded {},

    #[error("Already voted on this proposal")]
    AlreadyVoted {},

    #[error("Proposal already executed")]
    ProposalAlreadyExecuted {},

    #[error("Proposal not passed")]
    ProposalNotPassed {},

    #[error("Quorum not reached")]
    QuorumNotReached {},

    #[error("Invalid voting period: must be between {min} and {max} seconds")]
    InvalidVotingPeriod { min: u64, max: u64 },

    #[error("Invalid proposal data")]
    InvalidProposalData {},

    #[error("Agent not registered")]
    AgentNotRegistered {},
}