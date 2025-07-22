use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, Addr, WasmQuery, QueryRequest, StdError, Timestamp,
};
use cw2::set_contract_version;
use agent_karma_contracts::{Proposal, ProposalStatus, Vote};

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, GovernanceConfig, ProposalResponse, ProposalsResponse,
    VotingPowerResponse, VoteResponse, VotesResponse, ConfigResponse,
};
use crate::state::{CONFIG, PROPOSAL_COUNT, PROPOSALS, VOTES, VOTE_TALLIES, VoteTally};

// Contract name and version for migration info
const CONTRACT_NAME: &str = "governance-dao";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Validate configuration
    if msg.quorum_threshold > 100 {
        return Err(ContractError::InvalidProposalData {});
    }

    let config = GovernanceConfig {
        min_karma_for_proposal: msg.min_karma_for_proposal,
        min_karma_for_voting: msg.min_karma_for_voting,
        default_voting_period: msg.default_voting_period,
        quorum_threshold: msg.quorum_threshold,
        execution_delay: msg.execution_delay,
        karma_core_address: deps.api.addr_validate(&msg.karma_core_address)?,
        agent_registry_address: deps.api.addr_validate(&msg.agent_registry_address)?,
    };

    CONFIG.save(deps.storage, &config)?;
    PROPOSAL_COUNT.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateProposal {
            title,
            description,
            calldata,
            voting_period,
        } => execute_create_proposal(deps, env, info, title, description, calldata, voting_period),
        ExecuteMsg::VoteProposal { proposal_id, support } => {
            execute_vote_proposal(deps, env, info, proposal_id, support)
        }
        ExecuteMsg::FinalizeProposal { proposal_id } => {
            execute_finalize_proposal(deps, env, info, proposal_id)
        }
        ExecuteMsg::ExecuteProposal { proposal_id } => {
            execute_execute_proposal(deps, env, info, proposal_id)
        }
        ExecuteMsg::CancelProposal { proposal_id } => {
            execute_cancel_proposal(deps, env, info, proposal_id)
        }
        ExecuteMsg::UpdateConfig {
            min_karma_for_proposal,
            min_karma_for_voting,
            default_voting_period,
            quorum_threshold,
            execution_delay,
        } => execute_update_config(
            deps,
            env,
            info,
            min_karma_for_proposal,
            min_karma_for_voting,
            default_voting_period,
            quorum_threshold,
            execution_delay,
        ),
    }
}

pub fn execute_create_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    description: String,
    calldata: String,
    voting_period: Option<u64>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    // Check if agent is registered
    let is_registered = query_agent_registered(deps.as_ref(), &info.sender, &config.agent_registry_address)?;
    if !is_registered {
        return Err(ContractError::AgentNotRegistered {});
    }

    // Check karma requirement for proposal creation
    let karma_score = query_karma_score(deps.as_ref(), &info.sender, &config.karma_core_address)?;
    if karma_score < config.min_karma_for_proposal {
        return Err(ContractError::InsufficientKarma {
            required: config.min_karma_for_proposal.u128(),
            current: karma_score.u128(),
        });
    }

    // Validate proposal data
    if title.is_empty() || description.is_empty() {
        return Err(ContractError::InvalidProposalData {});
    }

    // Determine voting period
    let voting_period_seconds = voting_period.unwrap_or(config.default_voting_period);
    if voting_period_seconds < 3600 || voting_period_seconds > 604800 {
        // Between 1 hour and 1 week
        return Err(ContractError::InvalidVotingPeriod {
            min: 3600,
            max: 604800,
        });
    }

    // Get next proposal ID
    let proposal_id = PROPOSAL_COUNT.load(deps.storage)?;
    let next_id = proposal_id + 1;
    PROPOSAL_COUNT.save(deps.storage, &next_id)?;

    // Create proposal
    let proposal = Proposal {
        id: next_id,
        title: title.clone(),
        description: description.clone(),
        proposer: info.sender.clone(),
        calldata,
        created_at: env.block.time,
        voting_deadline: env.block.time.plus_seconds(voting_period_seconds),
        executed: false,
        votes_for: Uint128::zero(),
        votes_against: Uint128::zero(),
        quorum_required: calculate_quorum_requirement(deps.as_ref(), &config)?,
        status: ProposalStatus::Active,
    };

    // Save proposal and initialize vote tally
    PROPOSALS.save(deps.storage, next_id, &proposal)?;
    VOTE_TALLIES.save(
        deps.storage,
        next_id,
        &VoteTally {
            votes_for: Uint128::zero(),
            votes_against: Uint128::zero(),
            total_voting_power: Uint128::zero(),
            unique_voters: 0,
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "create_proposal")
        .add_attribute("proposal_id", next_id.to_string())
        .add_attribute("proposer", info.sender)
        .add_attribute("title", title))
}

pub fn execute_vote_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    support: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    // Check if agent is registered
    let is_registered = query_agent_registered(deps.as_ref(), &info.sender, &config.agent_registry_address)?;
    if !is_registered {
        return Err(ContractError::AgentNotRegistered {});
    }

    // Check karma requirement for voting
    let karma_score = query_karma_score(deps.as_ref(), &info.sender, &config.karma_core_address)?;
    if karma_score < config.min_karma_for_voting {
        return Err(ContractError::InsufficientKarma {
            required: config.min_karma_for_voting.u128(),
            current: karma_score.u128(),
        });
    }

    // Load proposal
    let mut proposal = PROPOSALS
        .load(deps.storage, proposal_id)
        .map_err(|_| ContractError::ProposalNotFound { id: proposal_id })?;

    // Check if voting period is still active
    if env.block.time > proposal.voting_deadline {
        return Err(ContractError::VotingPeriodEnded {});
    }

    // Check if already voted
    if VOTES.has(deps.storage, (proposal_id, &info.sender)) {
        return Err(ContractError::AlreadyVoted {});
    }

    // Calculate voting power (square root of karma)
    let voting_power = calculate_voting_power(karma_score);

    // Create vote record
    let vote = Vote {
        proposal_id,
        voter: info.sender.clone(),
        support,
        voting_power,
        timestamp: env.block.time,
        block_height: env.block.height,
    };

    // Save vote
    VOTES.save(deps.storage, (proposal_id, &info.sender), &vote)?;

    // Update vote tally
    let mut tally = VOTE_TALLIES.load(deps.storage, proposal_id)?;
    if support {
        tally.votes_for += voting_power;
        proposal.votes_for += voting_power;
    } else {
        tally.votes_against += voting_power;
        proposal.votes_against += voting_power;
    }
    tally.total_voting_power += voting_power;
    tally.unique_voters += 1;

    // Save updated tally and proposal
    VOTE_TALLIES.save(deps.storage, proposal_id, &tally)?;
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(Response::new()
        .add_attribute("method", "vote_proposal")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("voter", info.sender)
        .add_attribute("support", support.to_string())
        .add_attribute("voting_power", voting_power.to_string()))
}

pub fn execute_finalize_proposal(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    // Load proposal
    let mut proposal = PROPOSALS
        .load(deps.storage, proposal_id)
        .map_err(|_| ContractError::ProposalNotFound { id: proposal_id })?;

    // Check if voting period has ended
    if env.block.time <= proposal.voting_deadline {
        return Err(ContractError::VotingPeriodNotEnded {});
    }

    // Check if already finalized
    if proposal.status != ProposalStatus::Active {
        return Err(ContractError::ProposalAlreadyExecuted {});
    }

    let tally = VOTE_TALLIES.load(deps.storage, proposal_id)?;

    // Check quorum
    let quorum_met = tally.total_voting_power >= proposal.quorum_required;
    
    // Determine if proposal passed
    let passed = quorum_met && proposal.votes_for > proposal.votes_against;

    // Update proposal status
    proposal.status = if passed {
        ProposalStatus::Passed
    } else {
        ProposalStatus::Failed
    };

    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(Response::new()
        .add_attribute("method", "finalize_proposal")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("status", format!("{:?}", proposal.status))
        .add_attribute("quorum_met", quorum_met.to_string())
        .add_attribute("votes_for", proposal.votes_for.to_string())
        .add_attribute("votes_against", proposal.votes_against.to_string()))
}

pub fn execute_execute_proposal(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    // Load proposal
    let mut proposal = PROPOSALS
        .load(deps.storage, proposal_id)
        .map_err(|_| ContractError::ProposalNotFound { id: proposal_id })?;

    // Check if proposal passed
    if proposal.status != ProposalStatus::Passed {
        return Err(ContractError::ProposalNotPassed {});
    }

    // Check execution delay
    let config = CONFIG.load(deps.storage)?;
    let execution_time = proposal.voting_deadline.plus_seconds(config.execution_delay);
    if env.block.time < execution_time {
        return Err(ContractError::VotingPeriodNotEnded {}); // Reusing error for execution delay
    }

    // Mark as executed
    proposal.executed = true;
    proposal.status = ProposalStatus::Executed;
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    // Note: In a real implementation, you would decode and execute the calldata here
    // For this implementation, we'll just mark it as executed

    Ok(Response::new()
        .add_attribute("method", "execute_proposal")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("executed", "true"))
}

pub fn execute_cancel_proposal(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    // Load proposal
    let mut proposal = PROPOSALS
        .load(deps.storage, proposal_id)
        .map_err(|_| ContractError::ProposalNotFound { id: proposal_id })?;

    // Check if caller is proposer (in a real implementation, also check for admin)
    if proposal.proposer != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Check if proposal is still active
    if proposal.status != ProposalStatus::Active {
        return Err(ContractError::ProposalAlreadyExecuted {});
    }

    // Cancel proposal
    proposal.status = ProposalStatus::Failed;
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(Response::new()
        .add_attribute("method", "cancel_proposal")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("cancelled_by", info.sender))
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    min_karma_for_proposal: Option<Uint128>,
    min_karma_for_voting: Option<Uint128>,
    default_voting_period: Option<u64>,
    quorum_threshold: Option<u8>,
    execution_delay: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // In a real implementation, check if sender is admin
    // For now, we'll allow any registered agent with high karma
    let karma_score = query_karma_score(deps.as_ref(), &info.sender, &config.karma_core_address)?;
    if karma_score < Uint128::from(1000u128) {
        return Err(ContractError::Unauthorized {});
    }

    // Update configuration
    if let Some(min_karma) = min_karma_for_proposal {
        config.min_karma_for_proposal = min_karma;
    }
    if let Some(min_karma) = min_karma_for_voting {
        config.min_karma_for_voting = min_karma;
    }
    if let Some(period) = default_voting_period {
        config.default_voting_period = period;
    }
    if let Some(threshold) = quorum_threshold {
        if threshold > 100 {
            return Err(ContractError::InvalidProposalData {});
        }
        config.quorum_threshold = threshold;
    }
    if let Some(delay) = execution_delay {
        config.execution_delay = delay;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "update_config")
        .add_attribute("updated_by", info.sender))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetProposal { proposal_id } => to_binary(&query_proposal(deps, proposal_id)?),
        QueryMsg::GetActiveProposals { start_after, limit } => {
            to_binary(&query_active_proposals(deps, env, start_after, limit)?)
        }
        QueryMsg::GetAllProposals { start_after, limit } => {
            to_binary(&query_all_proposals(deps, start_after, limit)?)
        }
        QueryMsg::GetVotingPower { agent_address } => {
            to_binary(&query_voting_power(deps, agent_address)?)
        }
        QueryMsg::GetVote { proposal_id, voter } => {
            to_binary(&query_vote(deps, proposal_id, voter)?)
        }
        QueryMsg::GetProposalVotes {
            proposal_id,
            start_after,
            limit,
        } => to_binary(&query_proposal_votes(deps, proposal_id, start_after, limit)?),
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
    }
}

fn query_proposal(deps: Deps, proposal_id: u64) -> StdResult<ProposalResponse> {
    let proposal = PROPOSALS.may_load(deps.storage, proposal_id)?;
    Ok(ProposalResponse { proposal })
}

fn query_active_proposals(
    deps: Deps,
    env: Env,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ProposalsResponse> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    let start = start_after.unwrap_or(0);

    let proposals: StdResult<Vec<Proposal>> = PROPOSALS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter(|item| {
            if let Ok((id, proposal)) = item {
                *id > start
                    && proposal.status == ProposalStatus::Active
                    && env.block.time <= proposal.voting_deadline
            } else {
                false
            }
        })
        .take(limit)
        .map(|item| item.map(|(_, proposal)| proposal))
        .collect();

    Ok(ProposalsResponse {
        proposals: proposals?,
    })
}

fn query_all_proposals(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ProposalsResponse> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    let start = start_after.unwrap_or(0);

    let proposals: StdResult<Vec<Proposal>> = PROPOSALS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter(|item| {
            if let Ok((id, _)) = item {
                *id > start
            } else {
                false
            }
        })
        .take(limit)
        .map(|item| item.map(|(_, proposal)| proposal))
        .collect();

    Ok(ProposalsResponse {
        proposals: proposals?,
    })
}

fn query_voting_power(deps: Deps, agent_address: String) -> StdResult<VotingPowerResponse> {
    let config = CONFIG.load(deps.storage)?;
    let agent_addr = deps.api.addr_validate(&agent_address)?;
    
    let karma_score = query_karma_score(deps, &agent_addr, &config.karma_core_address)?;
    let voting_power = calculate_voting_power(karma_score);

    Ok(VotingPowerResponse {
        voting_power,
        karma_score,
    })
}

fn query_vote(deps: Deps, proposal_id: u64, voter: String) -> StdResult<VoteResponse> {
    let voter_addr = deps.api.addr_validate(&voter)?;
    let vote = VOTES.may_load(deps.storage, (proposal_id, &voter_addr))?;
    Ok(VoteResponse { vote })
}

fn query_proposal_votes(
    deps: Deps,
    proposal_id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<VotesResponse> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    
    let start_addr = if let Some(addr_str) = start_after {
        Some(deps.api.addr_validate(&addr_str)?)
    } else {
        None
    };

    let votes: StdResult<Vec<Vote>> = VOTES
        .prefix(proposal_id)
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter(|item| {
            if let (Some(start), Ok((addr, _))) = (&start_addr, item) {
                addr > start
            } else {
                true
            }
        })
        .take(limit)
        .map(|item| item.map(|(_, vote)| vote))
        .collect();

    Ok(VotesResponse { votes: votes? })
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

// Helper functions

pub fn calculate_voting_power(karma_score: Uint128) -> Uint128 {
    // Square root voting power to prevent excessive concentration
    let karma_u128 = karma_score.u128();
    let sqrt_karma = (karma_u128 as f64).sqrt() as u128;
    Uint128::from(sqrt_karma)
}

fn calculate_quorum_requirement(deps: Deps, config: &GovernanceConfig) -> StdResult<Uint128> {
    // In a real implementation, this would query total karma in the system
    // For now, we'll use a fixed value
    let total_karma = Uint128::from(10000u128); // Placeholder
    let quorum_karma = total_karma.multiply_ratio(config.quorum_threshold as u128, 100u128);
    Ok(quorum_karma)
}

fn query_karma_score(deps: Deps, agent_addr: &Addr, karma_core_addr: &Addr) -> StdResult<Uint128> {
    // In a real implementation, this would query the karma core contract
    // For now, we'll return a placeholder value
    // This would be something like:
    // let query_msg = karma_core::QueryMsg::GetKarmaScore { 
    //     agent_address: agent_addr.to_string() 
    // };
    // let response: karma_core::KarmaScoreResponse = deps.querier.query_wasm_smart(
    //     karma_core_addr,
    //     &query_msg,
    // )?;
    // Ok(response.score)
    
    // Placeholder implementation
    Ok(Uint128::from(500u128))
}

fn query_agent_registered(deps: Deps, agent_addr: &Addr, registry_addr: &Addr) -> StdResult<bool> {
    // In a real implementation, this would query the agent registry contract
    // For now, we'll return true as placeholder
    // This would be something like:
    // let query_msg = agent_registry::QueryMsg::IsRegistered { 
    //     agent_address: agent_addr.to_string() 
    // };
    // let response: agent_registry::IsRegisteredResponse = deps.querier.query_wasm_smart(
    //     registry_addr,
    //     &query_msg,
    // )?;
    // Ok(response.registered)
    
    // Placeholder implementation
    Ok(true)
}