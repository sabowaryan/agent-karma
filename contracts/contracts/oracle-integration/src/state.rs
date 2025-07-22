//! State management for Oracle Integration contract

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use crate::msg::{Config, OracleDataEntry, DisputeInfo, OracleProvider};

/// Contract configuration
pub const CONFIG: Item<Config> = Item::new("config");

/// Oracle data storage - maps data hash to oracle data entry
pub const ORACLE_DATA: Map<String, OracleDataEntry> = Map::new("oracle_data");

/// Oracle providers - maps provider address to provider info
pub const ORACLE_PROVIDERS: Map<Addr, OracleProvider> = Map::new("oracle_providers");

/// Disputes - maps data hash to dispute information
pub const DISPUTES: Map<String, DisputeInfo> = Map::new("disputes");

/// Data type index - maps data type to list of data hashes
pub const DATA_TYPE_INDEX: Map<String, Vec<String>> = Map::new("data_type_index");

/// Timestamp index - maps timestamp (as string) to list of data hashes
pub const TIMESTAMP_INDEX: Map<String, Vec<String>> = Map::new("timestamp_index");

/// Provider data count - tracks how much data each provider has submitted
pub const PROVIDER_DATA_COUNT: Map<Addr, u64> = Map::new("provider_data_count");

/// Consensus tracking - maps data hash to consensus status
pub const CONSENSUS_STATUS: Map<String, bool> = Map::new("consensus_status");