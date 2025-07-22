use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use agent_karma_contracts::types::{Interaction, InteractionMetadata};

/// Contract configuration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Administrator address
    pub admin: Addr,
    /// Maximum number of participants per interaction
    pub max_participants: u32,
    /// Maximum pagination limit
    pub max_pagination_limit: u32,
    /// Retry attempts for storage operations
    pub max_retry_attempts: u32,
    /// Valid interaction types
    pub valid_interaction_types: Vec<String>,
}

/// Stored interaction data with additional indexing fields
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StoredInteraction {
    /// The core interaction data
    pub interaction: Interaction,
    /// Hash of the interaction for verification
    pub hash: String,
    /// Whether the interaction has been verified
    pub verified: bool,
    /// Number of storage retry attempts
    pub retry_count: u32,
}

/// Storage for contract configuration
pub const CONFIG: Item<Config> = Item::new("config");

/// Counter for generating unique interaction IDs
pub const INTERACTION_COUNTER: Item<u64> = Item::new("interaction_counter");

/// Primary storage for interactions by hash
pub const INTERACTIONS: Map<&str, StoredInteraction> = Map::new("interactions");

/// Index structure for interactions
pub struct InteractionIndexes<'a> {
    /// Index by participant address
    pub participant: MultiIndex<'a, String, StoredInteraction, String>,
    /// Index by timestamp for chronological queries
    pub timestamp: MultiIndex<'a, u64, StoredInteraction, String>,
    /// Index by interaction type
    pub interaction_type: MultiIndex<'a, String, StoredInteraction, String>,
    /// Index by block height
    pub block_height: MultiIndex<'a, u64, StoredInteraction, String>,
}

impl<'a> IndexList<StoredInteraction> for InteractionIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<StoredInteraction>> + '_> {
        let v: Vec<&dyn Index<StoredInteraction>> = vec![
            &self.participant,
            &self.timestamp,
            &self.interaction_type,
            &self.block_height,
        ];
        Box::new(v.into_iter())
    }
}

/// Indexed map for efficient interaction queries
pub fn interactions<'a>() -> IndexedMap<'a, &'a str, StoredInteraction, InteractionIndexes<'a>> {
    let indexes = InteractionIndexes {
        participant: MultiIndex::new(
            |_pk: &[u8], d: &StoredInteraction| {
                d.interaction.participants.iter().map(|p| p.to_string()).collect::<Vec<_>>()
            },
            "interactions",
            "interactions__participant",
        ),
        timestamp: MultiIndex::new(
            |_pk: &[u8], d: &StoredInteraction| d.interaction.timestamp.seconds(),
            "interactions",
            "interactions__timestamp",
        ),
        interaction_type: MultiIndex::new(
            |_pk: &[u8], d: &StoredInteraction| d.interaction.interaction_type.clone(),
            "interactions",
            "interactions__type",
        ),
        block_height: MultiIndex::new(
            |_pk: &[u8], d: &StoredInteraction| d.interaction.block_height,
            "interactions",
            "interactions__block_height",
        ),
    };
    IndexedMap::new("interactions", indexes)
}

/// Storage for interaction verification status
pub const VERIFICATION_STATUS: Map<&str, bool> = Map::new("verification_status");

/// Storage for failed storage operations (for retry mechanism)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FailedOperation {
    /// Operation type
    pub operation_type: String,
    /// Data to retry
    pub data: String,
    /// Number of attempts made
    pub attempts: u32,
    /// Last error message
    pub last_error: String,
    /// Timestamp of last attempt
    pub last_attempt: Timestamp,
}

pub const FAILED_OPERATIONS: Map<&str, FailedOperation> = Map::new("failed_operations");

/// Default configuration values
impl Default for Config {
    fn default() -> Self {
        Self {
            admin: Addr::unchecked(""),
            max_participants: 10,
            max_pagination_limit: 100,
            max_retry_attempts: 3,
            valid_interaction_types: vec![
                "conversation".to_string(),
                "task".to_string(),
                "collaboration".to_string(),
                "transaction".to_string(),
                "negotiation".to_string(),
                "information_exchange".to_string(),
                "service_request".to_string(),
                "feedback".to_string(),
            ],
        }
    }
}