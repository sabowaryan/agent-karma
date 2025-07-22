//! Comprehensive test suite for Oracle Integration contract

use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    coins, from_binary, Addr, Uint128,
};

use crate::contract::{execute, instantiate, query};
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, OracleSignature, 
    OracleDataResponse, OracleProvidersResponse, ConsensusResponse,
    ConfigResponse, KarmaOracleDataResponse,
};
use crate::error::ContractError;

#[test]
fn test_proper_initialization() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        initial_providers: vec![
            "provider1".to_string(),
            "provider2".to_string(),
            "provider3".to_string(),
        ],
        min_signatures: Some(3),
        min_dispute_stake: Some(Uint128::new(100)),
    };
    let info = mock_info("creator", &coins(1000, "earth"));

    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    
    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.attributes[0].value, "instantiate");
    assert_eq!(res.attributes[1].value, "admin");
    assert_eq!(res.attributes[2].value, "3");
    assert_eq!(res.attributes[3].value, "3");

    // Query config to verify initialization
    let query_msg = QueryMsg::GetConfig {};
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    
    assert_eq!(config_response.config.admin, Addr::unchecked("admin"));
    assert_eq!(config_response.config.min_signatures, 3);
    assert_eq!(config_response.config.min_dispute_stake, Uint128::new(100));
}

#[test]
fn test_submit_oracle_data_success() {
    let mut deps = mock_dependencies();

    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        initial_providers: vec!["provider1".to_string(), "provider2".to_string(), "provider3".to_string()],
        min_signatures: Some(2),
        min_dispute_stake: Some(Uint128::new(50)),
    };
    let info = mock_info("creator", &[]);
    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    // Submit oracle data with sufficient signatures
    let msg = ExecuteMsg::SubmitOracleData {
        data_type: "performance".to_string(),
        data: "{\"agent_id\": \"agent1\", \"performance_score\": 85}".to_string(),
        signatures: vec![
            OracleSignature {
                provider: "provider1".to_string(),
                signature: "sig1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            OracleSignature {
                provider: "provider2".to_string(),
                signature: "sig2".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ],
    };
    let info = mock_info("provider1", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(res.attributes.len(), 7);
    assert_eq!(res.attributes[0].value, "submit_oracle_data");
    assert_eq!(res.attributes[2].value, "performance");
    assert_eq!(res.attributes[4].value, "2");
    assert_eq!(res.attributes[5].value, "true"); // consensus_reached
    assert_eq!(res.attributes[6].value, "consensus_reached");
}

#[test]
fn test_submit_oracle_data_insufficient_signatures() {
    let mut deps = mock_dependencies();

    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        initial_providers: vec!["provider1".to_string(), "provider2".to_string(), "provider3".to_string()],
        min_signatures: Some(3),
        min_dispute_stake: Some(Uint128::new(50)),
    };
    let info = mock_info("creator", &[]);
    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    // Submit oracle data with insufficient signatures
    let msg = ExecuteMsg::SubmitOracleData {
        data_type: "performance".to_string(),
        data: "{\"agent_id\": \"agent1\", \"performance_score\": 85}".to_string(),
        signatures: vec![
            OracleSignature {
                provider: "provider1".to_string(),
                signature: "sig1".to_string(),
                public_key: "pubkey1".to_string(),
            },
        ],
    };
    let info = mock_info("provider1", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(res.attributes.len(), 7);
    assert_eq!(res.attributes[5].value, "false"); // consensus_reached
    assert_eq!(res.attributes[6].value, "awaiting_consensus");
}

#[test]
fn test_submit_oracle_data_invalid_data_type() {
    let mut deps = mock_dependencies();

    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        initial_providers: vec!["provider1".to_string()],
        min_signatures: Some(1),
        min_dispute_stake: Some(Uint128::new(50)),
    };
    let info = mock_info("creator", &[]);
    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    // Submit oracle data with invalid data type
    let msg = ExecuteMsg::SubmitOracleData {
        data_type: "invalid_type".to_string(),
        data: "{\"data\": \"test\"}".to_string(),
        signatures: vec![
            OracleSignature {
                provider: "provider1".to_string(),
                signature: "sig1".to_string(),
                public_key: "pubkey1".to_string(),
            },
        ],
    };
    let info = mock_info("provider1", &[]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    match err {
        ContractError::InvalidDataType { data_type } => {
            assert_eq!(data_type, "invalid_type");
        }
        _ => panic!("Expected InvalidDataType error"),
    }
}

#[test]
fn test_dispute_oracle_data_success() {
    let mut deps = mock_dependencies();

    // Initialize and submit oracle data
    setup_oracle_data(&mut deps);

    // Dispute the oracle data
    let msg = ExecuteMsg::DisputeOracleData {
        data_hash: "test_hash".to_string(),
        stake_amount: Uint128::new(100),
        evidence: "This data appears to be incorrect based on external verification".to_string(),
    };
    let info = mock_info("challenger", &[]);
    
    // First, we need to get the actual data hash from a submitted oracle data
    let submit_msg = ExecuteMsg::SubmitOracleData {
        data_type: "performance".to_string(),
        data: "{\"test\": \"data\"}".to_string(),
        signatures: vec![
            OracleSignature {
                provider: "provider1".to_string(),
                signature: "sig1".to_string(),
                public_key: "pubkey1".to_string(),
            },
        ],
    };
    let submit_info = mock_info("provider1", &[]);
    execute(deps.as_mut(), mock_env(), submit_info, submit_msg).unwrap();

    // Generate the actual hash using the mock environment timestamp
    let env = mock_env();
    let data_hash = generate_actual_hash(
        "performance",
        "{\"test\": \"data\"}",
        env.block.time.seconds()
    );
    
    let dispute_msg = ExecuteMsg::DisputeOracleData {
        data_hash,
        stake_amount: Uint128::new(100),
        evidence: "Incorrect data detected".to_string(),
    };
    let dispute_info = mock_info("challenger", &[]);
    let res = execute(deps.as_mut(), env, dispute_info, dispute_msg).unwrap();

    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.attributes[0].value, "dispute_oracle_data");
}

#[test]
fn test_dispute_oracle_data_insufficient_stake() {
    let mut deps = mock_dependencies();
    setup_oracle_data(&mut deps);

    // First submit oracle data to get a valid hash
    let env = mock_env();
    let submit_msg = ExecuteMsg::SubmitOracleData {
        data_type: "performance".to_string(),
        data: "{\"agent_id\": \"agent1\", \"performance_score\": 85}".to_string(),
        signatures: vec![
            OracleSignature {
                provider: "provider1".to_string(),
                signature: "sig1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            OracleSignature {
                provider: "provider2".to_string(),
                signature: "sig2".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ],
    };
    let submit_info = mock_info("provider1", &[]);
    execute(deps.as_mut(), env.clone(), submit_info, submit_msg).unwrap();

    // Generate the actual hash using the mock environment timestamp
    let data_hash = generate_actual_hash(
        "performance",
        "{\"agent_id\": \"agent1\", \"performance_score\": 85}",
        env.block.time.seconds()
    );

    let msg = ExecuteMsg::DisputeOracleData {
        data_hash,
        stake_amount: Uint128::new(25), // Below minimum of 50
        evidence: "Incorrect data".to_string(),
    };
    let info = mock_info("challenger", &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();

    match err {
        ContractError::InsufficientStake { got, required } => {
            assert_eq!(got, "25");
            assert_eq!(required, "50");
        }
        _ => panic!("Expected InsufficientStake error"),
    }
}

#[test]
fn test_resolve_dispute_success() {
    let mut deps = mock_dependencies();
    setup_oracle_data(&mut deps);

    // Submit and dispute oracle data
    let data_hash = submit_and_dispute_data(&mut deps);

    // Resolve dispute as admin
    let msg = ExecuteMsg::ResolveDispute {
        data_hash: data_hash.clone(),
        resolution: false, // Data is invalid
        resolution_reason: "Data verification failed against external sources".to_string(),
    };
    let info = mock_info("admin", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.attributes[0].value, "resolve_dispute");
    assert_eq!(res.attributes[2].value, "false");
}

#[test]
fn test_resolve_dispute_unauthorized() {
    let mut deps = mock_dependencies();
    setup_oracle_data(&mut deps);

    let data_hash = submit_and_dispute_data(&mut deps);

    // Try to resolve dispute as non-admin
    let msg = ExecuteMsg::ResolveDispute {
        data_hash,
        resolution: true,
        resolution_reason: "Data is valid".to_string(),
    };
    let info = mock_info("non_admin", &[]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    match err {
        ContractError::Unauthorized {} => {}
        _ => panic!("Expected Unauthorized error"),
    }
}

#[test]
fn test_add_oracle_provider_success() {
    let mut deps = mock_dependencies();
    setup_oracle_data(&mut deps);

    let msg = ExecuteMsg::AddOracleProvider {
        provider: "new_provider".to_string(),
        public_key: "new_pubkey".to_string(),
    };
    let info = mock_info("admin", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(res.attributes.len(), 2);
    assert_eq!(res.attributes[0].value, "add_oracle_provider");
    assert_eq!(res.attributes[1].value, "new_provider");

    // Verify provider was added
    let query_msg = QueryMsg::GetOracleProviders {};
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let providers_response: OracleProvidersResponse = from_binary(&res).unwrap();
    
    assert!(providers_response.providers.len() >= 4); // 3 initial + 1 new
}

#[test]
fn test_add_oracle_provider_unauthorized() {
    let mut deps = mock_dependencies();
    setup_oracle_data(&mut deps);

    let msg = ExecuteMsg::AddOracleProvider {
        provider: "new_provider".to_string(),
        public_key: "new_pubkey".to_string(),
    };
    let info = mock_info("non_admin", &[]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    match err {
        ContractError::Unauthorized {} => {}
        _ => panic!("Expected Unauthorized error"),
    }
}

#[test]
fn test_remove_oracle_provider_success() {
    let mut deps = mock_dependencies();
    setup_oracle_data(&mut deps);

    let msg = ExecuteMsg::RemoveOracleProvider {
        provider: "provider3".to_string(),
    };
    let info = mock_info("admin", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(res.attributes.len(), 2);
    assert_eq!(res.attributes[0].value, "remove_oracle_provider");
    assert_eq!(res.attributes[1].value, "provider3");
}

#[test]
fn test_remove_oracle_provider_minimum_required() {
    let mut deps = mock_dependencies();

    // Initialize with minimum providers
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        initial_providers: vec!["provider1".to_string(), "provider2".to_string()],
        min_signatures: Some(2),
        min_dispute_stake: Some(Uint128::new(50)),
    };
    let info = mock_info("creator", &[]);
    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    // Try to remove provider when at minimum
    let msg = ExecuteMsg::RemoveOracleProvider {
        provider: "provider1".to_string(),
    };
    let info = mock_info("admin", &[]);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    match err {
        ContractError::MinimumOracleProvidersRequired { required } => {
            assert_eq!(required, 2);
        }
        _ => panic!("Expected MinimumOracleProvidersRequired error"),
    }
}

#[test]
fn test_query_oracle_data() {
    let mut deps = mock_dependencies();
    setup_oracle_data(&mut deps);

    let data_hash = submit_test_data(&mut deps);

    let query_msg = QueryMsg::GetOracleData { data_hash: data_hash.clone() };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let data_response: OracleDataResponse = from_binary(&res).unwrap();

    assert!(data_response.data.is_some());
    let oracle_data = data_response.data.unwrap();
    assert_eq!(oracle_data.data_hash, data_hash);
    assert_eq!(oracle_data.data_type, "performance");
}

#[test]
fn test_query_oracle_data_by_type() {
    let mut deps = mock_dependencies();
    setup_oracle_data(&mut deps);

    submit_test_data(&mut deps);

    let query_msg = QueryMsg::GetOracleDataByType {
        data_type: "performance".to_string(),
        start_time: None,
        end_time: None,
        limit: Some(10),
    };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let data_response: crate::msg::OracleDataListResponse = from_binary(&res).unwrap();

    assert!(!data_response.data.is_empty());
    assert_eq!(data_response.data[0].data_type, "performance");
}

#[test]
fn test_query_check_consensus() {
    let mut deps = mock_dependencies();
    setup_oracle_data(&mut deps);

    let data_hash = submit_test_data(&mut deps);

    let query_msg = QueryMsg::CheckConsensus { data_hash };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let consensus_response: ConsensusResponse = from_binary(&res).unwrap();

    assert!(consensus_response.consensus_reached);
    assert_eq!(consensus_response.required_signatures, 2);
}

#[test]
fn test_query_karma_oracle_data() {
    let mut deps = mock_dependencies();
    setup_oracle_data(&mut deps);

    // Submit different types of oracle data
    submit_karma_test_data(&mut deps);

    let query_msg = QueryMsg::GetKarmaOracleData {
        agent_address: "agent1".to_string(),
        data_types: vec!["performance".to_string(), "cross_chain".to_string(), "sentiment".to_string()],
    };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let karma_response: KarmaOracleDataResponse = from_binary(&res).unwrap();

    assert!(karma_response.performance_data.is_some());
    assert_eq!(karma_response.performance_weight, "15");
    assert_eq!(karma_response.cross_chain_weight, "10");
    assert_eq!(karma_response.sentiment_weight, "5");
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies();
    setup_oracle_data(&mut deps);

    let msg = ExecuteMsg::UpdateConfig {
        min_signatures: Some(4),
        min_dispute_stake: Some(Uint128::new(200)),
    };
    let info = mock_info("admin", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[1].value, "4");
    assert_eq!(res.attributes[2].value, "200");

    // Verify config was updated
    let query_msg = QueryMsg::GetConfig {};
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    
    assert_eq!(config_response.config.min_signatures, 4);
    assert_eq!(config_response.config.min_dispute_stake, Uint128::new(200));
}

// Helper functions for tests

fn setup_oracle_data(deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>) {
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        initial_providers: vec![
            "provider1".to_string(),
            "provider2".to_string(),
            "provider3".to_string(),
        ],
        min_signatures: Some(2),
        min_dispute_stake: Some(Uint128::new(50)),
    };
    let info = mock_info("creator", &[]);
    instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
}

fn submit_test_data(deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>) -> String {
    let env = mock_env();
    let msg = ExecuteMsg::SubmitOracleData {
        data_type: "performance".to_string(),
        data: "{\"agent_id\": \"agent1\", \"performance_score\": 85}".to_string(),
        signatures: vec![
            OracleSignature {
                provider: "provider1".to_string(),
                signature: "sig1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            OracleSignature {
                provider: "provider2".to_string(),
                signature: "sig2".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ],
    };
    let info = mock_info("provider1", &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Generate hash using actual timestamp from mock environment
    generate_actual_hash(
        "performance",
        "{\"agent_id\": \"agent1\", \"performance_score\": 85}",
        env.block.time.seconds()
    )
}

fn submit_karma_test_data(deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>) {
    // Submit performance data
    let perf_msg = ExecuteMsg::SubmitOracleData {
        data_type: "performance".to_string(),
        data: "{\"agent_id\": \"agent1\", \"score\": 85}".to_string(),
        signatures: vec![
            OracleSignature {
                provider: "provider1".to_string(),
                signature: "sig1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            OracleSignature {
                provider: "provider2".to_string(),
                signature: "sig2".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ],
    };
    let info = mock_info("provider1", &[]);
    execute(deps.as_mut(), mock_env(), info, perf_msg).unwrap();

    // Submit cross-chain data
    let cross_msg = ExecuteMsg::SubmitOracleData {
        data_type: "cross_chain".to_string(),
        data: "{\"agent_id\": \"agent1\", \"reputation\": 75}".to_string(),
        signatures: vec![
            OracleSignature {
                provider: "provider1".to_string(),
                signature: "sig3".to_string(),
                public_key: "pubkey1".to_string(),
            },
            OracleSignature {
                provider: "provider2".to_string(),
                signature: "sig4".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ],
    };
    let info = mock_info("provider1", &[]);
    execute(deps.as_mut(), mock_env(), info, cross_msg).unwrap();
}

fn submit_and_dispute_data(deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>) -> String {
    let env = mock_env();
    
    // Submit oracle data first
    let submit_msg = ExecuteMsg::SubmitOracleData {
        data_type: "performance".to_string(),
        data: "{\"agent_id\": \"agent1\", \"performance_score\": 85}".to_string(),
        signatures: vec![
            OracleSignature {
                provider: "provider1".to_string(),
                signature: "sig1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            OracleSignature {
                provider: "provider2".to_string(),
                signature: "sig2".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ],
    };
    let submit_info = mock_info("provider1", &[]);
    execute(deps.as_mut(), env.clone(), submit_info, submit_msg).unwrap();
    
    // Generate the actual hash using the mock environment timestamp
    let data_hash = generate_actual_hash(
        "performance", 
        "{\"agent_id\": \"agent1\", \"performance_score\": 85}", 
        env.block.time.seconds()
    );

    let dispute_msg = ExecuteMsg::DisputeOracleData {
        data_hash: data_hash.clone(),
        stake_amount: Uint128::new(100),
        evidence: "Data appears incorrect".to_string(),
    };
    let info = mock_info("challenger", &[]);
    execute(deps.as_mut(), env, info, dispute_msg).unwrap();

    data_hash
}



fn generate_actual_hash(data_type: &str, data: &str, timestamp_seconds: u64) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data_type.as_bytes());
    hasher.update(data.as_bytes());
    hasher.update(timestamp_seconds.to_string().as_bytes());
    hex::encode(hasher.finalize())
}