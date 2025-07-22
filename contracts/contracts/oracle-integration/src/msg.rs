//! Message types for Oracle Integration contract

use cosmwasm_std::{Addr, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Instantiation message for Oracle Integration contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Administrator address for the contract
    pub admin: Option<String>,
    /// Initial oracle providers
    pub initial_providers: Vec<String>,
    /// Minimum signatures required for consensus (default: 3)
    pub min_signatures: Option<u32>,
    /// Minimum stake required for disputes
    pub min_dispute_stake: Option<Uint128>,
}

/// Execute messages for Oracle Integration contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Submit oracle data with multi-signature validation
    SubmitOracleData {
        data_type: String,
        data: String,
        signatures: Vec<OracleSignature>,
    },
    /// Dispute oracle data by staking karma
    DisputeOracleData {
        data_hash: String,
        stake_amount: Uint128,
        evidence: String,
    },
    /// Resolve oracle dispute (admin only)
    ResolveDispute {
        data_hash: String,
        resolution: bool, // true if data is valid, false if invalid
        resolution_reason: String,
    },
    /// Add oracle provider (admin only)
    AddOracleProvider {
        provider: String,
        public_key: String,
    },
    /// Remove oracle provider (admin only)
    RemoveOracleProvider {
        provider: String,
    },
    /// Update contract configuration (admin only)
    UpdateConfig {
        min_signatures: Option<u32>,
        min_dispute_stake: Option<Uint128>,
    },
}

/// Query messages for Oracle Integration contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Get oracle data by hash
    GetOracleData {
        data_hash: String,
    },
    /// Get oracle data by type and time range
    GetOracleDataByType {
        data_type: String,
        start_time: Option<Timestamp>,
        end_time: Option<Timestamp>,
        limit: Option<u32>,
    },
    /// Get all oracle providers
    GetOracleProviders {},
    /// Check if data has reached consensus
    CheckConsensus {
        data_hash: String,
    },
    /// Get disputed data
    GetDisputedData {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Get contract configuration
    GetConfig {},
    /// Get oracle data for karma calculation
    GetKarmaOracleData {
        agent_address: String,
        data_types: Vec<String>, // performance, cross_chain, sentiment
    },
}

/// Oracle signature structure
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleSignature {
    /// Address of the oracle provider
    pub provider: String,
    /// Cryptographic signature
    pub signature: String,
    /// Public key used for verification
    pub public_key: String,
}

/// Oracle data entry
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleDataEntry {
    /// Hash of the data for identification
    pub data_hash: String,
    /// Address of the submitting provider
    pub provider: Addr,
    /// Type of data (performance, cross_chain, sentiment)
    pub data_type: String,
    /// The actual data payload (JSON string)
    pub data: String,
    /// Timestamp of data submission
    pub timestamp: Timestamp,
    /// Signatures from validator nodes
    pub signatures: Vec<OracleSignature>,
    /// Whether consensus has been reached
    pub consensus_reached: bool,
    /// Number of valid signatures
    pub signature_count: u32,
    /// Whether the data has been disputed
    pub disputed: bool,
}

/// Dispute information
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DisputeInfo {
    /// Hash of the disputed data
    pub data_hash: String,
    /// Address of the challenger
    pub challenger: Addr,
    /// Amount staked for the dispute
    pub stake_amount: Uint128,
    /// Evidence provided by challenger
    pub evidence: String,
    /// Timestamp when dispute was created
    pub created_at: Timestamp,
    /// Whether the dispute has been resolved
    pub resolved: bool,
    /// Resolution timestamp
    pub resolved_at: Option<Timestamp>,
    /// Resolution result (true = data valid, false = data invalid)
    pub resolution: Option<bool>,
    /// Reason for resolution
    pub resolution_reason: Option<String>,
}

/// Oracle provider information
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleProvider {
    /// Address of the provider
    pub address: Addr,
    /// Public key for signature verification
    pub public_key: String,
    /// Whether the provider is active
    pub active: bool,
    /// Timestamp when provider was added
    pub added_at: Timestamp,
}

/// Contract configuration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Administrator address
    pub admin: Addr,
    /// Minimum signatures required for consensus
    pub min_signatures: u32,
    /// Minimum stake required for disputes
    pub min_dispute_stake: Uint128,
}

/// Response types for queries
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleDataResponse {
    pub data: Option<OracleDataEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleDataListResponse {
    pub data: Vec<OracleDataEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleProvidersResponse {
    pub providers: Vec<OracleProvider>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConsensusResponse {
    pub consensus_reached: bool,
    pub signature_count: u32,
    pub required_signatures: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DisputedDataResponse {
    pub disputes: Vec<DisputeInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub config: Config,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct KarmaOracleDataResponse {
    pub performance_data: Option<String>,
    pub cross_chain_data: Option<String>,
    pub sentiment_data: Option<String>,
    pub performance_weight: String, // "15" for 15%
    pub cross_chain_weight: String, // "10" for 10%
    pub sentiment_weight: String,   // "5" for 5%
}

/// Migration message
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub version: String,
}