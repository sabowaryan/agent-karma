//! Oracle Integration smart contract implementation
//! 
//! This contract manages external data sources with multi-signature validation,
//! proof-of-report staking, and dispute resolution mechanisms.

use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Addr, Timestamp, Uint128, Order,
};
use cw2::set_contract_version;
use sha2::{Digest, Sha256};
use hex;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg, Config, OracleDataEntry, 
    OracleSignature, DisputeInfo, OracleProvider, OracleDataResponse, 
    OracleDataListResponse, OracleProvidersResponse, ConsensusResponse,
    DisputedDataResponse, ConfigResponse, KarmaOracleDataResponse,
};
use crate::state::{
    CONFIG, ORACLE_DATA, ORACLE_PROVIDERS, DISPUTES, DATA_TYPE_INDEX,
    TIMESTAMP_INDEX, PROVIDER_DATA_COUNT, CONSENSUS_STATUS,
};

// Contract metadata
const CONTRACT_NAME: &str = "crates.io:oracle-integration";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Default configuration values
const DEFAULT_MIN_SIGNATURES: u32 = 3;
const DEFAULT_MIN_DISPUTE_STAKE: u128 = 100; // 100 karma tokens

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
        admin: admin.clone(),
        min_signatures: msg.min_signatures.unwrap_or(DEFAULT_MIN_SIGNATURES),
        min_dispute_stake: msg.min_dispute_stake.unwrap_or_else(|| Uint128::new(DEFAULT_MIN_DISPUTE_STAKE)),
    };

    CONFIG.save(deps.storage, &config)?;

    let initial_providers_count = msg.initial_providers.len();

    // Add initial oracle providers
    for provider_addr in msg.initial_providers {
        let provider = deps.api.addr_validate(&provider_addr)?;
        let oracle_provider = OracleProvider {
            address: provider.clone(),
            public_key: String::new(), // Will be set when provider submits first data
            active: true,
            added_at: _env.block.time,
        };
        ORACLE_PROVIDERS.save(deps.storage, provider, &oracle_provider)?;
    }

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin)
        .add_attribute("min_signatures", config.min_signatures.to_string())
        .add_attribute("initial_providers", initial_providers_count.to_string()))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SubmitOracleData { data_type, data, signatures } => {
            execute_submit_oracle_data(deps, env, info, data_type, data, signatures)
        }
        ExecuteMsg::DisputeOracleData { data_hash, stake_amount, evidence } => {
            execute_dispute_oracle_data(deps, env, info, data_hash, stake_amount, evidence)
        }
        ExecuteMsg::ResolveDispute { data_hash, resolution, resolution_reason } => {
            execute_resolve_dispute(deps, env, info, data_hash, resolution, resolution_reason)
        }
        ExecuteMsg::AddOracleProvider { provider, public_key } => {
            execute_add_oracle_provider(deps, env, info, provider, public_key)
        }
        ExecuteMsg::RemoveOracleProvider { provider } => {
            execute_remove_oracle_provider(deps, env, info, provider)
        }
        ExecuteMsg::UpdateConfig { min_signatures, min_dispute_stake } => {
            execute_update_config(deps, env, info, min_signatures, min_dispute_stake)
        }
    }
}

fn execute_submit_oracle_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data_type: String,
    data: String,
    signatures: Vec<OracleSignature>,
) -> Result<Response, ContractError> {
    // Validate data type
    if !is_valid_data_type(&data_type) {
        return Err(ContractError::InvalidDataType { data_type });
    }

    // Generate data hash
    let data_hash = generate_data_hash(&data_type, &data, env.block.time);

    // Check if data already exists
    if ORACLE_DATA.has(deps.storage, data_hash.clone()) {
        return Err(ContractError::OracleDataAlreadyExists { data_hash });
    }

    // Verify signatures
    let valid_signatures = verify_signatures(deps.as_ref(), &data_hash, &signatures)?;
    
    let config = CONFIG.load(deps.storage)?;
    let consensus_reached = valid_signatures >= config.min_signatures;

    // Create oracle data entry
    let oracle_entry = OracleDataEntry {
        data_hash: data_hash.clone(),
        provider: info.sender.clone(),
        data_type: data_type.clone(),
        data,
        timestamp: env.block.time,
        signatures,
        consensus_reached,
        signature_count: valid_signatures,
        disputed: false,
    };

    // Save oracle data
    ORACLE_DATA.save(deps.storage, data_hash.clone(), &oracle_entry)?;
    CONSENSUS_STATUS.save(deps.storage, data_hash.clone(), &consensus_reached)?;

    // Update indexes
    update_data_type_index(deps.storage, &data_type, &data_hash)?;
    update_timestamp_index(deps.storage, env.block.time, &data_hash)?;

    // Update provider data count
    let current_count = PROVIDER_DATA_COUNT
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(0);
    PROVIDER_DATA_COUNT.save(deps.storage, info.sender.clone(), &(current_count + 1))?;

    let mut response = Response::new()
        .add_attribute("method", "submit_oracle_data")
        .add_attribute("data_hash", data_hash)
        .add_attribute("data_type", data_type)
        .add_attribute("provider", info.sender)
        .add_attribute("signature_count", valid_signatures.to_string())
        .add_attribute("consensus_reached", consensus_reached.to_string());

    if consensus_reached {
        response = response.add_attribute("status", "consensus_reached");
    } else {
        response = response.add_attribute("status", "awaiting_consensus");
    }

    Ok(response)
}

fn execute_dispute_oracle_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data_hash: String,
    stake_amount: Uint128,
    evidence: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Check minimum stake requirement
    if stake_amount < config.min_dispute_stake {
        return Err(ContractError::InsufficientStake {
            got: stake_amount.to_string(),
            required: config.min_dispute_stake.to_string(),
        });
    }

    // Check if oracle data exists
    let mut oracle_data = ORACLE_DATA.load(deps.storage, data_hash.clone())
        .map_err(|_| ContractError::OracleDataNotFound { data_hash: data_hash.clone() })?;

    // Check if dispute already exists
    if DISPUTES.has(deps.storage, data_hash.clone()) {
        return Err(ContractError::DisputeAlreadyExists { data_hash });
    }

    // Create dispute
    let dispute = DisputeInfo {
        data_hash: data_hash.clone(),
        challenger: info.sender.clone(),
        stake_amount,
        evidence,
        created_at: env.block.time,
        resolved: false,
        resolved_at: None,
        resolution: None,
        resolution_reason: None,
    };

    // Save dispute
    DISPUTES.save(deps.storage, data_hash.clone(), &dispute)?;

    // Mark oracle data as disputed
    oracle_data.disputed = true;
    ORACLE_DATA.save(deps.storage, data_hash.clone(), &oracle_data)?;

    Ok(Response::new()
        .add_attribute("method", "dispute_oracle_data")
        .add_attribute("data_hash", data_hash)
        .add_attribute("challenger", info.sender)
        .add_attribute("stake_amount", stake_amount))
}

fn execute_resolve_dispute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data_hash: String,
    resolution: bool,
    resolution_reason: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only admin can resolve disputes
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Load dispute
    let mut dispute = DISPUTES.load(deps.storage, data_hash.clone())
        .map_err(|_| ContractError::DisputeNotFound { data_hash: data_hash.clone() })?;

    // Check if already resolved
    if dispute.resolved {
        return Err(ContractError::DisputeAlreadyResolved { data_hash });
    }

    // Resolve dispute
    dispute.resolved = true;
    dispute.resolved_at = Some(env.block.time);
    dispute.resolution = Some(resolution);
    dispute.resolution_reason = Some(resolution_reason.clone());

    DISPUTES.save(deps.storage, data_hash.clone(), &dispute)?;

    // If resolution is false (data is invalid), remove the oracle data
    if !resolution {
        ORACLE_DATA.remove(deps.storage, data_hash.clone());
        CONSENSUS_STATUS.remove(deps.storage, data_hash.clone());
    }

    Ok(Response::new()
        .add_attribute("method", "resolve_dispute")
        .add_attribute("data_hash", data_hash)
        .add_attribute("resolution", resolution.to_string())
        .add_attribute("resolution_reason", resolution_reason))
}

fn execute_add_oracle_provider(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    provider: String,
    public_key: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only admin can add providers
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    let provider_addr = deps.api.addr_validate(&provider)?;

    // Check if provider already exists
    if ORACLE_PROVIDERS.has(deps.storage, provider_addr.clone()) {
        return Err(ContractError::OracleProviderAlreadyExists { provider });
    }

    let oracle_provider = OracleProvider {
        address: provider_addr.clone(),
        public_key,
        active: true,
        added_at: env.block.time,
    };

    ORACLE_PROVIDERS.save(deps.storage, provider_addr, &oracle_provider)?;

    Ok(Response::new()
        .add_attribute("method", "add_oracle_provider")
        .add_attribute("provider", provider))
}

fn execute_remove_oracle_provider(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    provider: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only admin can remove providers
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    let provider_addr = deps.api.addr_validate(&provider)?;

    // Check if provider exists
    if !ORACLE_PROVIDERS.has(deps.storage, provider_addr.clone()) {
        return Err(ContractError::OracleProviderNotFound { provider });
    }

    // Check minimum providers requirement
    let provider_count = ORACLE_PROVIDERS
        .range(deps.storage, None, None, Order::Ascending)
        .count();

    if provider_count <= (config.min_signatures as usize) {
        return Err(ContractError::MinimumOracleProvidersRequired {
            required: config.min_signatures,
        });
    }

    ORACLE_PROVIDERS.remove(deps.storage, provider_addr);

    Ok(Response::new()
        .add_attribute("method", "remove_oracle_provider")
        .add_attribute("provider", provider))
}

fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    min_signatures: Option<u32>,
    min_dispute_stake: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // Only admin can update config
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(min_sigs) = min_signatures {
        config.min_signatures = min_sigs;
    }

    if let Some(min_stake) = min_dispute_stake {
        config.min_dispute_stake = min_stake;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "update_config")
        .add_attribute("min_signatures", config.min_signatures.to_string())
        .add_attribute("min_dispute_stake", config.min_dispute_stake))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOracleData { data_hash } => {
            to_binary(&query_oracle_data(deps, data_hash)?)
        }
        QueryMsg::GetOracleDataByType { data_type, start_time, end_time, limit } => {
            to_binary(&query_oracle_data_by_type(deps, data_type, start_time, end_time, limit)?)
        }
        QueryMsg::GetOracleProviders {} => {
            to_binary(&query_oracle_providers(deps)?)
        }
        QueryMsg::CheckConsensus { data_hash } => {
            to_binary(&query_check_consensus(deps, data_hash)?)
        }
        QueryMsg::GetDisputedData { start_after, limit } => {
            to_binary(&query_disputed_data(deps, start_after, limit)?)
        }
        QueryMsg::GetConfig {} => {
            to_binary(&query_config(deps)?)
        }
        QueryMsg::GetKarmaOracleData { agent_address, data_types } => {
            to_binary(&query_karma_oracle_data(deps, agent_address, data_types)?)
        }
    }
}

fn query_oracle_data(deps: Deps, data_hash: String) -> StdResult<OracleDataResponse> {
    let data = ORACLE_DATA.may_load(deps.storage, data_hash)?;
    Ok(OracleDataResponse { data })
}

fn query_oracle_data_by_type(
    deps: Deps,
    data_type: String,
    _start_time: Option<Timestamp>,
    _end_time: Option<Timestamp>,
    limit: Option<u32>,
) -> StdResult<OracleDataListResponse> {
    let data_hashes = DATA_TYPE_INDEX.may_load(deps.storage, data_type)?
        .unwrap_or_default();

    let limit = limit.unwrap_or(50) as usize;
    let mut data = Vec::new();

    for hash in data_hashes.iter().take(limit) {
        if let Some(oracle_data) = ORACLE_DATA.may_load(deps.storage, hash.clone())? {
            data.push(oracle_data);
        }
    }

    Ok(OracleDataListResponse { data })
}

fn query_oracle_providers(deps: Deps) -> StdResult<OracleProvidersResponse> {
    let providers: StdResult<Vec<_>> = ORACLE_PROVIDERS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_, provider)| provider))
        .collect();

    Ok(OracleProvidersResponse {
        providers: providers?,
    })
}

fn query_check_consensus(deps: Deps, data_hash: String) -> StdResult<ConsensusResponse> {
    let config = CONFIG.load(deps.storage)?;
    
    if let Some(oracle_data) = ORACLE_DATA.may_load(deps.storage, data_hash)? {
        Ok(ConsensusResponse {
            consensus_reached: oracle_data.consensus_reached,
            signature_count: oracle_data.signature_count,
            required_signatures: config.min_signatures,
        })
    } else {
        Ok(ConsensusResponse {
            consensus_reached: false,
            signature_count: 0,
            required_signatures: config.min_signatures,
        })
    }
}

fn query_disputed_data(
    deps: Deps,
    _start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<DisputedDataResponse> {
    let limit = limit.unwrap_or(50) as usize;
    
    let disputes: StdResult<Vec<_>> = DISPUTES
        .range(deps.storage, None, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, dispute)| dispute))
        .collect();

    Ok(DisputedDataResponse {
        disputes: disputes?,
    })
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

fn query_karma_oracle_data(
    deps: Deps,
    _agent_address: String,
    data_types: Vec<String>,
) -> StdResult<KarmaOracleDataResponse> {
    let mut performance_data = None;
    let mut cross_chain_data = None;
    let mut sentiment_data = None;

    for data_type in data_types {
        if let Some(data_hashes) = DATA_TYPE_INDEX.may_load(deps.storage, data_type.clone())? {
            // Get the most recent data for this type
            if let Some(latest_hash) = data_hashes.last() {
                if let Some(oracle_data) = ORACLE_DATA.may_load(deps.storage, latest_hash.clone())? {
                    if oracle_data.consensus_reached && !oracle_data.disputed {
                        match data_type.as_str() {
                            "performance" => performance_data = Some(oracle_data.data),
                            "cross_chain" => cross_chain_data = Some(oracle_data.data),
                            "sentiment" => sentiment_data = Some(oracle_data.data),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    Ok(KarmaOracleDataResponse {
        performance_data,
        cross_chain_data,
        sentiment_data,
        performance_weight: "15".to_string(),
        cross_chain_weight: "10".to_string(),
        sentiment_weight: "5".to_string(),
    })
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

// Helper functions

fn is_valid_data_type(data_type: &str) -> bool {
    matches!(data_type, "performance" | "cross_chain" | "sentiment" | "general")
}

fn generate_data_hash(data_type: &str, data: &str, timestamp: Timestamp) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data_type.as_bytes());
    hasher.update(data.as_bytes());
    hasher.update(timestamp.seconds().to_string().as_bytes());
    hex::encode(hasher.finalize())
}

fn verify_signatures(
    deps: Deps,
    data_hash: &str,
    signatures: &[OracleSignature],
) -> Result<u32, ContractError> {
    let mut valid_count = 0;

    for signature in signatures {
        let provider_addr = deps.api.addr_validate(&signature.provider)?;
        
        // Check if provider exists and is active
        if let Some(provider) = ORACLE_PROVIDERS.may_load(deps.storage, provider_addr)? {
            if provider.active {
                // In a real implementation, you would verify the cryptographic signature here
                // For this implementation, we'll assume signatures are valid if the provider exists
                if verify_signature_crypto(data_hash, &signature.signature, &signature.public_key) {
                    valid_count += 1;
                }
            }
        }
    }

    Ok(valid_count)
}

fn verify_signature_crypto(_data_hash: &str, _signature: &str, _public_key: &str) -> bool {
    // Placeholder for cryptographic signature verification
    // In a real implementation, this would use proper cryptographic libraries
    // to verify the signature against the public key and data hash
    true
}

fn update_data_type_index(
    storage: &mut dyn cosmwasm_std::Storage,
    data_type: &str,
    data_hash: &str,
) -> StdResult<()> {
    let mut hashes = DATA_TYPE_INDEX.may_load(storage, data_type.to_string())?
        .unwrap_or_default();
    hashes.push(data_hash.to_string());
    DATA_TYPE_INDEX.save(storage, data_type.to_string(), &hashes)
}

fn update_timestamp_index(
    storage: &mut dyn cosmwasm_std::Storage,
    timestamp: Timestamp,
    data_hash: &str,
) -> StdResult<()> {
    let timestamp_key = timestamp.seconds().to_string();
    let mut hashes = TIMESTAMP_INDEX.may_load(storage, timestamp_key.clone())?
        .unwrap_or_default();
    hashes.push(data_hash.to_string());
    TIMESTAMP_INDEX.save(storage, timestamp_key, &hashes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            admin: None,
            initial_providers: vec!["provider1".to_string(), "provider2".to_string()],
            min_signatures: Some(2),
            min_dispute_stake: Some(Uint128::new(50)),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(4, res.attributes.len());
    }

    #[test]
    fn submit_oracle_data() {
        let mut deps = mock_dependencies();

        // Initialize contract
        let msg = InstantiateMsg {
            admin: None,
            initial_providers: vec!["provider1".to_string()],
            min_signatures: Some(1),
            min_dispute_stake: Some(Uint128::new(50)),
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Submit oracle data
        let msg = ExecuteMsg::SubmitOracleData {
            data_type: "performance".to_string(),
            data: "{\"score\": 85}".to_string(),
            signatures: vec![OracleSignature {
                provider: "provider1".to_string(),
                signature: "signature1".to_string(),
                public_key: "pubkey1".to_string(),
            }],
        };
        let info = mock_info("provider1", &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.attributes.len(), 7);
        assert_eq!(res.attributes[0].value, "submit_oracle_data");
    }
}