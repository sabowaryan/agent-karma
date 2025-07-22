use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use agent_karma_contracts::types::Agent;

/// Contract configuration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Administrator address
    pub admin: Addr,
    /// Maximum number of agents that can be registered
    pub max_agents: Option<u64>,
    /// Whether registration is currently enabled
    pub registration_enabled: bool,
}

/// Agent registration status
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum AgentStatus {
    Active,
    Deactivated,
}

/// Extended agent information stored in the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StoredAgent {
    /// Core agent information
    pub agent: Agent,
    /// Registration status
    pub status: AgentStatus,
    /// Who registered this agent (for audit purposes)
    pub registered_by: Addr,
    /// Last metadata update timestamp
    pub last_updated: Timestamp,
}

/// Contract configuration storage
pub const CONFIG: Item<Config> = Item::new("config");

/// Map of agent address to agent data
/// Key: agent address (String), Value: StoredAgent
pub const AGENTS: Map<&str, StoredAgent> = Map::new("agents");

/// Map to track agents by framework for efficient querying
/// Key: framework name (String), Value: Vec<agent addresses>
pub const AGENTS_BY_FRAMEWORK: Map<&str, Vec<String>> = Map::new("agents_by_framework");

/// Counter for total number of registered agents
pub const AGENT_COUNT: Item<u64> = Item::new("agent_count");

/// Map to store agent registration order for pagination
/// Key: registration order (u64), Value: agent address (String)
pub const AGENT_ORDER: Map<u64, String> = Map::new("agent_order");

/// Reverse mapping for efficient lookups
/// Key: agent address (String), Value: registration order (u64)
pub const AGENT_ORDER_REVERSE: Map<&str, u64> = Map::new("agent_order_reverse");