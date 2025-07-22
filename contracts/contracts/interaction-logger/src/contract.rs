use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Order, Addr, Timestamp,
};
use cw2::set_contract_version;

use agent_karma_contracts::{
    messages::{interaction_logger::*, InstantiateMsg},
    types::{Interaction, InteractionMetadata},
};

use crate::error::ContractError;
use crate::helpers::*;
use crate::state::{
    Config, StoredInteraction, CONFIG, INTERACTION_COUNTER, interactions, VERIFICATION_STATUS,
};

// Contract name and version for migration info
const CONTRACT_NAME: &str = "interaction-logger";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Contract instantiation
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    let admin = msg.admin
        .map(|addr| deps.api.addr_validate(&addr))
        .transpose()?
        .unwrap_or_else(|| info.sender.clone());
    
    let config = Config {
        admin,
        ..Default::default()
    };
    
    CONFIG.save(deps.storage, &config)?;
    INTERACTION_COUNTER.save(deps.storage, &0u64)?;
    
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", config.admin.to_string())
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

/// Contract execution entry point
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::LogInteraction {
            participants,
            interaction_type,
            metadata,
        } => execute_log_interaction(deps, env, info, participants, interaction_type, metadata),
        ExecuteMsg::VerifyInteraction { interaction_hash } => {
            execute_verify_interaction(deps, env, info, interaction_hash)
        }
        ExecuteMsg::UpdateInteractionMetadata {
            interaction_hash,
            metadata,
        } => execute_update_interaction_metadata(deps, env, info, interaction_hash, metadata),
    }
}

/// Log a new interaction
pub fn execute_log_interaction(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    participants: Vec<String>,
    interaction_type: String,
    metadata: InteractionMetadata,
) -> Result<Response, ContractError> {
    let config = get_config(deps.as_ref())?;
    
    // Validate inputs
    let validated_participants = validate_participants(&participants, &config)?;
    validate_interaction_type(&interaction_type, &config)?;
    validate_metadata(&metadata)?;
    validate_timestamp(&env.block.time, &env.block.time)?;
    
    // Check that the sender is one of the participants
    if !validated_participants.contains(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }
    
    // Verify all participants are registered agents
    for participant in &validated_participants {
        check_agent_registered(deps.as_ref(), participant)?;
    }
    
    // Generate unique interaction ID
    let counter = INTERACTION_COUNTER.load(deps.storage)?;
    let new_counter = counter + 1;
    let interaction_id = generate_interaction_id(new_counter, &env.block.time);
    
    // Generate interaction hash
    let interaction_hash = generate_interaction_hash(
        &validated_participants,
        &interaction_type,
        &env.block.time,
        &metadata,
    );
    
    // Check if interaction already exists
    if interaction_exists(deps.as_ref(), &interaction_hash)? {
        return Err(ContractError::InteractionAlreadyExists { interaction_hash });
    }
    
    // Create interaction object
    let interaction = Interaction {
        id: interaction_id,
        participants: validated_participants,
        interaction_type: sanitize_string(&interaction_type),
        timestamp: env.block.time,
        block_height: env.block.height,
        metadata: InteractionMetadata {
            duration: metadata.duration,
            outcome: metadata.outcome.map(|s| sanitize_string(&s)),
            context: metadata.context.map(|s| sanitize_string(&s)),
        },
    };
    
    let stored_interaction = StoredInteraction {
        interaction: interaction.clone(),
        hash: interaction_hash.clone(),
        verified: false, // Will be verified separately
        retry_count: 0,
    };
    
    // Store interaction with retry mechanism
    let operation_id = format!("log_interaction_{}", interaction_hash);
    retry_storage_operation(
        deps,
        &env,
        &operation_id,
        move || {
            interactions().save(deps.storage, &interaction_hash, &stored_interaction)?;
            INTERACTION_COUNTER.save(deps.storage, &new_counter)?;
            Ok(())
        },
        config.max_retry_attempts,
    )?;
    
    Ok(Response::new()
        .add_attribute("method", "log_interaction")
        .add_attribute("interaction_id", interaction.id)
        .add_attribute("interaction_hash", interaction_hash)
        .add_attribute("participants", participants.join(","))
        .add_attribute("interaction_type", interaction.interaction_type)
        .add_attribute("timestamp", interaction.timestamp.to_string())
        .add_attribute("block_height", interaction.block_height.to_string()))
}

/// Verify an interaction
pub fn execute_verify_interaction(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    interaction_hash: String,
) -> Result<Response, ContractError> {
    validate_interaction_hash(&interaction_hash)?;
    
    // Get the stored interaction
    let stored_interaction = get_interaction_by_hash(deps.as_ref(), &interaction_hash)?
        .ok_or_else(|| ContractError::InteractionNotFound {
            interaction_hash: interaction_hash.clone(),
        })?;
    
    // Verify the interaction integrity
    let is_valid = verify_interaction_integrity(
        &stored_interaction.interaction,
        &stored_interaction.hash,
    )?;
    
    if !is_valid {
        return Err(ContractError::VerificationFailed {
            reason: "Interaction hash does not match stored data".to_string(),
        });
    }
    
    // Update verification status
    VERIFICATION_STATUS.save(deps.storage, &interaction_hash, &true)?;
    
    // Update stored interaction verification status
    let mut updated_interaction = stored_interaction;
    updated_interaction.verified = true;
    interactions().save(deps.storage, &interaction_hash, &updated_interaction)?;
    
    Ok(Response::new()
        .add_attribute("method", "verify_interaction")
        .add_attribute("interaction_hash", interaction_hash)
        .add_attribute("verified", "true")
        .add_attribute("integrity_check", "passed"))
}

/// Update interaction metadata (only by participants)
pub fn execute_update_interaction_metadata(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    interaction_hash: String,
    metadata: InteractionMetadata,
) -> Result<Response, ContractError> {
    validate_interaction_hash(&interaction_hash)?;
    validate_metadata(&metadata)?;
    
    // Get the stored interaction
    let mut stored_interaction = get_interaction_by_hash(deps.as_ref(), &interaction_hash)?
        .ok_or_else(|| ContractError::InteractionNotFound {
            interaction_hash: interaction_hash.clone(),
        })?;
    
    // Check that the sender is one of the participants
    if !stored_interaction.interaction.participants.contains(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }
    
    // Update metadata
    stored_interaction.interaction.metadata = InteractionMetadata {
        duration: metadata.duration,
        outcome: metadata.outcome.map(|s| sanitize_string(&s)),
        context: metadata.context.map(|s| sanitize_string(&s)),
    };
    
    // Recalculate hash with new metadata
    let new_hash = generate_interaction_hash(
        &stored_interaction.interaction.participants,
        &stored_interaction.interaction.interaction_type,
        &stored_interaction.interaction.timestamp,
        &stored_interaction.interaction.metadata,
    );
    
    stored_interaction.hash = new_hash.clone();
    stored_interaction.verified = false; // Need to re-verify after update
    
    // Store updated interaction with retry mechanism
    let config = get_config(deps.as_ref())?;
    let operation_id = format!("update_metadata_{}", interaction_hash);
    retry_storage_operation(
        deps,
        &env,
        &operation_id,
        || {
            interactions().save(deps.storage, &interaction_hash, &stored_interaction)?;
            Ok(())
        },
        config.max_retry_attempts,
    )?;
    
    Ok(Response::new()
        .add_attribute("method", "update_interaction_metadata")
        .add_attribute("interaction_hash", interaction_hash)
        .add_attribute("new_hash", new_hash)
        .add_attribute("updated_by", info.sender.to_string()))
}

/// Contract query entry point
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetInteraction { interaction_hash } => {
            to_json_binary(&query_get_interaction(deps, interaction_hash)?)
        }
        QueryMsg::GetInteractionHistory {
            agent_address,
            start_after,
            limit,
        } => to_json_binary(&query_get_interaction_history(
            deps,
            agent_address,
            start_after,
            limit,
        )?),
        QueryMsg::GetInteractionsBetween {
            agent1,
            agent2,
            start_after,
            limit,
        } => to_json_binary(&query_get_interactions_between(
            deps, agent1, agent2, start_after, limit,
        )?),
        QueryMsg::GetRecentInteractions { start_after, limit } => {
            to_json_binary(&query_get_recent_interactions(deps, start_after, limit)?)
        }
        QueryMsg::VerifyInteractionExists { interaction_hash } => {
            to_json_binary(&query_verify_interaction_exists(deps, interaction_hash)?)
        }
    }
}

/// Query interaction by hash
pub fn query_get_interaction(
    deps: Deps,
    interaction_hash: String,
) -> Result<InteractionResponse, ContractError> {
    validate_interaction_hash(&interaction_hash)?;
    
    let stored_interaction = get_interaction_by_hash(deps, &interaction_hash)?;
    
    Ok(InteractionResponse {
        interaction: stored_interaction.map(|si| si.interaction),
    })
}

/// Query interaction history for an agent
pub fn query_get_interaction_history(
    deps: Deps,
    agent_address: String,
    start_after: Option<Timestamp>,
    limit: Option<u32>,
) -> Result<InteractionsResponse, ContractError> {
    let config = get_config(deps)?;
    let limit = validate_pagination(limit, &config)?;
    
    let agent_addr = deps.api.addr_validate(&agent_address)?;
    
    let start_bound = start_after.map(|ts| ts.seconds());
    
    let interactions_result: Result<Vec<_>, _> = interactions()
        .idx
        .participant
        .prefix(agent_addr.to_string())
        .range(deps.storage, start_bound.map(cosmwasm_std::Bound::exclusive), None, Order::Descending)
        .take(limit as usize)
        .collect();
    
    let interactions_list = interactions_result
        .map_err(ContractError::Std)?
        .into_iter()
        .map(|(_, stored_interaction)| stored_interaction.interaction)
        .collect();
    
    Ok(InteractionsResponse {
        interactions: interactions_list,
    })
}

/// Query interactions between two specific agents
pub fn query_get_interactions_between(
    deps: Deps,
    agent1: String,
    agent2: String,
    start_after: Option<Timestamp>,
    limit: Option<u32>,
) -> Result<InteractionsResponse, ContractError> {
    let config = get_config(deps)?;
    let limit = validate_pagination(limit, &config)?;
    
    let agent1_addr = deps.api.addr_validate(&agent1)?;
    let agent2_addr = deps.api.addr_validate(&agent2)?;
    
    let start_bound = start_after.map(|ts| ts.seconds());
    
    // Get all interactions and filter for those involving both agents
    let interactions_result: Result<Vec<_>, _> = interactions()
        .range(deps.storage, start_bound.map(cosmwasm_std::Bound::exclusive), None, Order::Descending)
        .take(limit as usize * 2) // Take more to account for filtering
        .collect();
    
    let filtered_interactions: Vec<Interaction> = interactions_result
        .map_err(ContractError::Std)?
        .into_iter()
        .map(|(_, stored_interaction)| stored_interaction.interaction)
        .filter(|interaction| {
            interaction.participants.contains(&agent1_addr) && 
            interaction.participants.contains(&agent2_addr)
        })
        .take(limit as usize)
        .collect();
    
    Ok(InteractionsResponse {
        interactions: filtered_interactions,
    })
}

/// Query recent interactions (global feed)
pub fn query_get_recent_interactions(
    deps: Deps,
    start_after: Option<Timestamp>,
    limit: Option<u32>,
) -> Result<InteractionsResponse, ContractError> {
    let config = get_config(deps)?;
    let limit = validate_pagination(limit, &config)?;
    
    let start_bound = start_after.map(|ts| ts.seconds());
    
    let interactions_result: Result<Vec<_>, _> = interactions()
        .idx
        .timestamp
        .range(deps.storage, start_bound.map(cosmwasm_std::Bound::exclusive), None, Order::Descending)
        .take(limit as usize)
        .collect();
    
    let interactions_list = interactions_result
        .map_err(ContractError::Std)?
        .into_iter()
        .map(|(_, stored_interaction)| stored_interaction.interaction)
        .collect();
    
    Ok(InteractionsResponse {
        interactions: interactions_list,
    })
}

/// Verify if interaction exists and is valid
pub fn query_verify_interaction_exists(
    deps: Deps,
    interaction_hash: String,
) -> Result<VerificationResponse, ContractError> {
    validate_interaction_hash(&interaction_hash)?;
    
    let stored_interaction = get_interaction_by_hash(deps, &interaction_hash)?;
    
    match stored_interaction {
        Some(si) => {
            let is_valid = verify_interaction_integrity(&si.interaction, &si.hash)?;
            Ok(VerificationResponse {
                verified: is_valid && si.verified,
                details: Some(format!(
                    "Interaction found, integrity check: {}, verification status: {}",
                    if is_valid { "passed" } else { "failed" },
                    if si.verified { "verified" } else { "not verified" }
                )),
            })
        }
        None => Ok(VerificationResponse {
            verified: false,
            details: Some("Interaction not found".to_string()),
        }),
    }
}

/// Migration entry point
#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: agent_karma_contracts::messages::MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}