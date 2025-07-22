use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map, MultiIndex, IndexList, IndexedMap, Index};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use agent_karma_contracts::types::{Rating, KarmaCalculation, KarmaConfig, KarmaFactors};

/// Contract configuration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Administrator address
    pub admin: Addr,
    /// Agent registry contract address
    pub agent_registry: Addr,
    /// Interaction logger contract address
    pub interaction_logger: Addr,
    /// Karma calculation configuration
    pub karma_config: KarmaConfig,
}

/// Stored rating with additional metadata
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StoredRating {
    /// Core rating information
    pub rating: Rating,
    /// Whether this rating has been processed in karma calculation
    pub processed: bool,
    /// Karma fee paid for this rating
    pub fee_paid: Uint128,
}

/// Karma score with historical tracking
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct KarmaScore {
    /// Current karma score
    pub current_score: Uint128,
    /// Previous karma score (for comparison)
    pub previous_score: Uint128,
    /// Last calculation timestamp
    pub last_updated: Timestamp,
    /// Total number of ratings received
    pub total_ratings: u64,
    /// Average rating score (stored as string to avoid floating point)
    pub average_rating: String,
    /// Interaction count at last calculation
    pub interaction_count: u64,
}

/// Rating duplicate prevention tracking
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RatingTracker {
    /// Interaction hash
    pub interaction_hash: String,
    /// Rater address
    pub rater: Addr,
    /// Timestamp when rating was submitted
    pub submitted_at: Timestamp,
}

/// Indexes for ratings
pub struct RatingIndexes<'a> {
    /// Index by rated agent address
    pub rated_agent: MultiIndex<'a, String, StoredRating, String>,
    /// Index by rater address
    pub rater: MultiIndex<'a, String, StoredRating, String>,
    /// Index by timestamp for chronological queries
    pub timestamp: MultiIndex<'a, u64, StoredRating, String>,
}

impl<'a> IndexList<StoredRating> for RatingIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<StoredRating>> + '_> {
        let v: Vec<&dyn Index<StoredRating>> = vec![&self.rated_agent, &self.rater, &self.timestamp];
        Box::new(v.into_iter())
    }
}

/// Contract configuration storage
pub const CONFIG: Item<Config> = Item::new("config");

/// Map of agent address to current karma score
/// Key: agent address (String), Value: KarmaScore
pub const KARMA_SCORES: Map<&str, KarmaScore> = Map::new("karma_scores");

/// Historical karma calculations for tracking
/// Key: (agent_address, timestamp), Value: KarmaCalculation
pub const KARMA_HISTORY: Map<(&str, u64), KarmaCalculation> = Map::new("karma_history");

/// Ratings storage with indexes
/// Key: rating ID (String), Value: StoredRating
pub fn ratings<'a>() -> IndexedMap<'a, &'a str, StoredRating, RatingIndexes<'a>> {
    let indexes = RatingIndexes {
        rated_agent: MultiIndex::new(
            |_pk: &[u8], r: &StoredRating| r.rating.rated_address.to_string(),
            "ratings",
            "ratings__rated_agent",
        ),
        rater: MultiIndex::new(
            |_pk: &[u8], r: &StoredRating| r.rating.rater_address.to_string(),
            "ratings",
            "ratings__rater",
        ),
        timestamp: MultiIndex::new(
            |_pk: &[u8], r: &StoredRating| r.rating.timestamp.seconds(),
            "ratings",
            "ratings__timestamp",
        ),
    };
    IndexedMap::new("ratings", indexes)
}

/// Rating duplicate prevention tracking
/// Key: (interaction_hash, rater_address), Value: RatingTracker
pub const RATING_TRACKERS: Map<(&str, &str), RatingTracker> = Map::new("rating_trackers");

/// Leaderboard storage for efficient queries
/// Key: karma score (u128), Value: agent address (String)
pub const LEADERBOARD: Map<u128, String> = Map::new("leaderboard");

/// Counter for rating IDs
pub const RATING_COUNTER: Item<u64> = Item::new("rating_counter");

/// Oracle data integration storage
/// Key: (agent_address, data_type), Value: oracle data hash
pub const ORACLE_DATA: Map<(&str, &str), String> = Map::new("oracle_data");

/// Karma earning/spending tracking
/// Key: agent_address, Value: (earned, spent)
pub const KARMA_BALANCE: Map<&str, (Uint128, Uint128)> = Map::new("karma_balance");

/// Compliance violations storage
/// Key: violation_id, Value: ComplianceViolation
pub const COMPLIANCE_VIOLATIONS: Map<&str, crate::compliance::ComplianceViolation> = Map::new("compliance_violations");

/// Dispute cases storage
/// Key: case_id, Value: DisputeCase
pub const DISPUTE_CASES: Map<&str, crate::compliance::DisputeCase> = Map::new("dispute_cases");

/// Rate limiting trackers
/// Key: agent_address, Value: RateLimitTracker
pub const RATE_LIMIT_TRACKERS: Map<&str, crate::compliance::RateLimitTracker> = Map::new("rate_limit_trackers");

/// Abuse pattern detection storage
/// Key: (agent_address, pattern_type), Value: detection_data
pub const ABUSE_PATTERNS: Map<(&str, &str), String> = Map::new("abuse_patterns");

/// Karma penalties tracking
/// Key: penalty_id, Value: penalty_amount
pub const KARMA_PENALTIES: Map<&str, Uint128> = Map::new("karma_penalties");