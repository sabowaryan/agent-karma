use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, Order,
};
use cw_storage_plus::Bound;
use cw2::set_contract_version;

use agent_karma_contracts::{
    messages::{agent_registry::*, InstantiateMsg, MigrateMsg},
    types::{Agent, AgentMetadata},
};

use crate::error::ContractError;
use crate::state::{
    Config, StoredAgent, AgentStatus, CONFIG, AGENTS, AGENTS_BY_FRAMEWORK, 
    AGENT_COUNT, AGENT_ORDER, AGENT_ORDER_REVERSE,
};

// Contract name and version for migration
const CONTRACT_NAME: &str = "agent-registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Constants for validation
const MAX_AGENT_NAME_LENGTH: usize = 64;
const MAX_AGENT_DESCRIPTION_LENGTH: usize = 512;
const MAX_FRAMEWORK_NAME_LENGTH: usize = 32;
const MAX_VERSION_LENGTH: usize = 16;
const VALID_FRAMEWORKS: &[&str] = &["ElizaOS", "MCP", "AIDN", "Custom"];

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = match msg.admin {
        Some(admin_str) => deps.api.addr_validate(&admin_str)?,
        None => info.sender.clone(),
    };

    let config = Config {
        admin,
        max_agents: None, // No limit by default
        registration_enabled: true,
    };

    CONFIG.save(deps.storage, &config)?;
    AGENT_COUNT.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", config.admin)
        .add_attribute("registration_enabled", "true"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterAgent { metadata } => {
            execute_register_agent(deps, env, info, metadata)
        }
        ExecuteMsg::UpdateAgentMetadata { metadata } => {
            execute_update_agent_metadata(deps, env, info, metadata)
        }
        ExecuteMsg::DeactivateAgent { agent_address } => {
            execute_deactivate_agent(deps, env, info, agent_address)
        }
    }
}

pub fn execute_register_agent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    metadata: AgentMetadata,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    // Check if registration is enabled
    if !config.registration_enabled {
        return Err(ContractError::Unauthorized {});
    }

    let agent_address = info.sender.clone();
    let agent_address_str = agent_address.to_string();

    // Check if agent is already registered
    if AGENTS.has(deps.storage, &agent_address_str) {
        return Err(ContractError::AgentAlreadyRegistered {
            address: agent_address_str,
        });
    }

    // Validate metadata
    validate_agent_metadata(&metadata)?;

    // Check max agents limit if set
    let current_count = AGENT_COUNT.load(deps.storage)?;
    if let Some(max_agents) = config.max_agents {
        if current_count >= max_agents {
            return Err(ContractError::Unauthorized {});
        }
    }

    // Create agent record
    let agent = Agent {
        address: agent_address.clone(),
        registration_date: env.block.time,
        metadata: metadata.clone(),
        karma_score: Uint128::zero(),
        interaction_count: 0,
        ratings_received: 0,
    };

    let stored_agent = StoredAgent {
        agent,
        status: AgentStatus::Active,
        registered_by: agent_address.clone(),
        last_updated: env.block.time,
    };

    // Save agent data
    AGENTS.save(deps.storage, &agent_address_str, &stored_agent)?;

    // Update agent count and order tracking
    let new_count = current_count + 1;
    AGENT_COUNT.save(deps.storage, &new_count)?;
    AGENT_ORDER.save(deps.storage, new_count, &agent_address_str)?;
    AGENT_ORDER_REVERSE.save(deps.storage, &agent_address_str, &new_count)?;

    // Update framework index
    update_framework_index(deps.storage, &metadata.framework, &agent_address_str, true)?;

    Ok(Response::new()
        .add_attribute("method", "register_agent")
        .add_attribute("agent_address", agent_address_str)
        .add_attribute("agent_name", metadata.name)
        .add_attribute("framework", metadata.framework)
        .add_attribute("registration_date", env.block.time.to_string()))
}

pub fn execute_update_agent_metadata(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    metadata: AgentMetadata,
) -> Result<Response, ContractError> {
    let agent_address_str = info.sender.to_string();

    // Load existing agent
    let mut stored_agent = AGENTS.load(deps.storage, &agent_address_str)
        .map_err(|_| ContractError::AgentNotFound {
            address: agent_address_str.clone(),
        })?;

    // Check if agent is active
    if stored_agent.status != AgentStatus::Active {
        return Err(ContractError::AgentDeactivated {
            address: agent_address_str,
        });
    }

    // Only the agent owner can update metadata
    if stored_agent.agent.address != info.sender {
        return Err(ContractError::OnlyOwnerCanUpdate {});
    }

    // Validate new metadata
    validate_agent_metadata(&metadata)?;

    // Update framework index if framework changed
    if stored_agent.agent.metadata.framework != metadata.framework {
        update_framework_index(
            deps.storage,
            &stored_agent.agent.metadata.framework,
            &agent_address_str,
            false,
        )?;
        update_framework_index(deps.storage, &metadata.framework, &agent_address_str, true)?;
    }

    // Update agent metadata
    stored_agent.agent.metadata = metadata.clone();
    stored_agent.last_updated = env.block.time;

    AGENTS.save(deps.storage, &agent_address_str, &stored_agent)?;

    Ok(Response::new()
        .add_attribute("method", "update_agent_metadata")
        .add_attribute("agent_address", agent_address_str)
        .add_attribute("agent_name", metadata.name)
        .add_attribute("framework", metadata.framework)
        .add_attribute("updated_at", env.block.time.to_string()))
}

pub fn execute_deactivate_agent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    agent_address: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only admin can deactivate agents
    if info.sender != config.admin {
        return Err(ContractError::AdminRequired {});
    }

    // Validate agent address
    let _validated_address = deps.api.addr_validate(&agent_address)?;

    // Load existing agent
    let mut stored_agent = AGENTS.load(deps.storage, &agent_address)
        .map_err(|_| ContractError::AgentNotFound {
            address: agent_address.clone(),
        })?;

    // Check if already deactivated
    if stored_agent.status == AgentStatus::Deactivated {
        return Err(ContractError::AgentDeactivated {
            address: agent_address,
        });
    }

    // Deactivate agent
    stored_agent.status = AgentStatus::Deactivated;
    stored_agent.last_updated = env.block.time;

    AGENTS.save(deps.storage, &agent_address, &stored_agent)?;

    // Remove from framework index
    update_framework_index(
        deps.storage,
        &stored_agent.agent.metadata.framework,
        &agent_address,
        false,
    )?;

    Ok(Response::new()
        .add_attribute("method", "deactivate_agent")
        .add_attribute("agent_address", agent_address)
        .add_attribute("deactivated_by", info.sender)
        .add_attribute("deactivated_at", env.block.time.to_string()))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAgent { agent_address } => to_json_binary(&query_get_agent(deps, agent_address)?),
        QueryMsg::IsRegistered { agent_address } => {
            to_json_binary(&query_is_registered(deps, agent_address)?)
        }
        QueryMsg::GetAllAgents { start_after, limit } => {
            to_json_binary(&query_get_all_agents(deps, start_after, limit)?)
        }
        QueryMsg::GetAgentsByFramework {
            framework,
            start_after,
            limit,
        } => to_json_binary(&query_get_agents_by_framework(
            deps, framework, start_after, limit,
        )?),
    }
}

pub fn query_get_agent(deps: Deps, agent_address: String) -> StdResult<AgentResponse> {
    let stored_agent = AGENTS.may_load(deps.storage, &agent_address)?;
    
    let agent = match stored_agent {
        Some(stored) if stored.status == AgentStatus::Active => Some(stored.agent),
        _ => None,
    };

    Ok(AgentResponse { agent })
}

pub fn query_is_registered(deps: Deps, agent_address: String) -> StdResult<IsRegisteredResponse> {
    let stored_agent = AGENTS.may_load(deps.storage, &agent_address)?;
    
    let registered = match stored_agent {
        Some(stored) => stored.status == AgentStatus::Active,
        None => false,
    };

    Ok(IsRegisteredResponse { registered })
}

pub fn query_get_all_agents(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AgentsResponse> {
    let limit = limit.unwrap_or(30);
    let limit = if limit == 0 { 30 } else { limit.min(100) } as usize;
    
    let start = start_after
        .as_ref()
        .and_then(|addr| AGENT_ORDER_REVERSE.may_load(deps.storage, addr).ok().flatten())
        .map(|order| order + 1)
        .unwrap_or(1);

    let agents: StdResult<Vec<Agent>> = AGENT_ORDER
        .range(deps.storage, Some(Bound::inclusive(start)), None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, agent_address) = item?;
            let stored_agent = AGENTS.load(deps.storage, &agent_address)?;
            
            // Only return active agents
            if stored_agent.status == AgentStatus::Active {
                Ok(Some(stored_agent.agent))
            } else {
                Ok(None)
            }
        })
        .filter_map(|result| match result {
            Ok(Some(agent)) => Some(Ok(agent)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        })
        .collect();

    Ok(AgentsResponse { agents: agents? })
}

pub fn query_get_agents_by_framework(
    deps: Deps,
    framework: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AgentsResponse> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    
    let agent_addresses = AGENTS_BY_FRAMEWORK
        .may_load(deps.storage, &framework)?
        .unwrap_or_default();

    let start_index = start_after
        .as_ref()
        .and_then(|addr| agent_addresses.iter().position(|a| a == addr))
        .map(|pos| pos + 1)
        .unwrap_or(0);

    let agents: StdResult<Vec<Agent>> = agent_addresses
        .iter()
        .skip(start_index)
        .take(limit)
        .map(|agent_address| {
            let stored_agent = AGENTS.load(deps.storage, agent_address)?;
            
            // Only return active agents
            if stored_agent.status == AgentStatus::Active {
                Ok(Some(stored_agent.agent))
            } else {
                Ok(None)
            }
        })
        .filter_map(|result| match result {
            Ok(Some(agent)) => Some(Ok(agent)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        })
        .collect();

    Ok(AgentsResponse { agents: agents? })
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

// Helper functions

fn validate_agent_metadata(metadata: &AgentMetadata) -> Result<(), ContractError> {
    // Validate name length
    if metadata.name.len() > MAX_AGENT_NAME_LENGTH {
        return Err(ContractError::AgentNameTooLong {});
    }

    // Validate description length
    if metadata.description.len() > MAX_AGENT_DESCRIPTION_LENGTH {
        return Err(ContractError::AgentDescriptionTooLong {});
    }

    // Validate framework
    if metadata.framework.len() > MAX_FRAMEWORK_NAME_LENGTH {
        return Err(ContractError::InvalidFramework {
            framework: metadata.framework.clone(),
        });
    }

    // Check if framework is in allowed list (optional validation)
    if !VALID_FRAMEWORKS.contains(&metadata.framework.as_str()) && metadata.framework != "Custom" {
        return Err(ContractError::InvalidFramework {
            framework: metadata.framework.clone(),
        });
    }

    // Validate version length
    if metadata.version.len() > MAX_VERSION_LENGTH {
        return Err(ContractError::InvalidMetadata {
            reason: "Version too long".to_string(),
        });
    }

    // Validate IPFS hash format if provided
    if let Some(ref ipfs_hash) = metadata.ipfs_hash {
        if !is_valid_ipfs_hash(ipfs_hash) {
            return Err(ContractError::InvalidIpfsHash {
                hash: ipfs_hash.clone(),
            });
        }
    }

    // Validate that required fields are not empty
    if metadata.name.trim().is_empty() {
        return Err(ContractError::InvalidMetadata {
            reason: "Name cannot be empty".to_string(),
        });
    }

    if metadata.framework.trim().is_empty() {
        return Err(ContractError::InvalidMetadata {
            reason: "Framework cannot be empty".to_string(),
        });
    }

    if metadata.version.trim().is_empty() {
        return Err(ContractError::InvalidMetadata {
            reason: "Version cannot be empty".to_string(),
        });
    }

    Ok(())
}

pub fn is_valid_ipfs_hash(hash: &str) -> bool {
    // Basic IPFS hash validation
    // IPFS hashes typically start with "Qm" and are 46 characters long (CIDv0)
    // or start with "bafy" and are longer (CIDv1)
    if hash.starts_with("Qm") && hash.len() == 46 {
        hash.chars().all(|c| c.is_ascii_alphanumeric())
    } else if hash.starts_with("bafy") && hash.len() >= 50 {
        hash.chars().all(|c| c.is_ascii_alphanumeric())
    } else {
        false
    }
}

fn update_framework_index(
    storage: &mut dyn cosmwasm_std::Storage,
    framework: &str,
    agent_address: &str,
    add: bool,
) -> StdResult<()> {
    let mut agents = AGENTS_BY_FRAMEWORK
        .may_load(storage, framework)?
        .unwrap_or_default();

    if add {
        if !agents.contains(&agent_address.to_string()) {
            agents.push(agent_address.to_string());
        }
    } else {
        agents.retain(|addr| addr != agent_address);
    }

    if agents.is_empty() {
        AGENTS_BY_FRAMEWORK.remove(storage, framework);
    } else {
        AGENTS_BY_FRAMEWORK.save(storage, framework, &agents)?;
    }

    Ok(())
}