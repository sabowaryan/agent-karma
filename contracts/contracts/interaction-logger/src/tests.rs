use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_json_binary, Addr, Timestamp, Uint128};

use agent_karma_contracts::{
    messages::{interaction_logger::*, InstantiateMsg},
    types::InteractionMetadata,
};

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::helpers::*;
use crate::state::{CONFIG, INTERACTION_COUNTER, interactions};

// Test constants
const ADMIN: &str = "admin";
const AGENT1: &str = "agent1";
const AGENT2: &str = "agent2";
const AGENT3: &str = "agent3";

fn setup_contract() -> (cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, cosmwasm_std::Env) {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    let msg = InstantiateMsg {
        admin: Some(ADMIN.to_string()),
        config: None,
    };
    
    let info = mock_info(ADMIN, &coins(1000, "token"));
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    (deps, env)
}

fn create_test_metadata() -> InteractionMetadata {
    InteractionMetadata {
        duration: Some(300), // 5 minutes
        outcome: Some("successful".to_string()),
        context: Some("test interaction".to_string()),
    }
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    let msg = InstantiateMsg {
        admin: Some(ADMIN.to_string()),
        config: None,
    };
    
    let info = mock_info(ADMIN, &coins(1000, "token"));
    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
    
    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.attributes[0].value, "instantiate");
    assert_eq!(res.attributes[1].value, ADMIN);
    
    // Check config was saved
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.admin, Addr::unchecked(ADMIN));
    assert_eq!(config.max_participants, 10);
    
    // Check counter was initialized
    let counter = INTERACTION_COUNTER.load(&deps.storage).unwrap();
    assert_eq!(counter, 0);
}

#[test]
fn test_log_interaction_success() {
    let (mut deps, env) = setup_contract();
    
    let participants = vec![AGENT1.to_string(), AGENT2.to_string()];
    let interaction_type = "conversation".to_string();
    let metadata = create_test_metadata();
    
    let msg = ExecuteMsg::LogInteraction {
        participants: participants.clone(),
        interaction_type: interaction_type.clone(),
        metadata: metadata.clone(),
    };
    
    let info = mock_info(AGENT1, &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    assert_eq!(res.attributes[0].value, "log_interaction");
    assert_eq!(res.attributes[2].value, participants.join(","));
    assert_eq!(res.attributes[3].value, interaction_type);
    
    // Check that interaction was saved
    let counter = INTERACTION_COUNTER.load(&deps.storage).unwrap();
    assert_eq!(counter, 1);
    
    // Check that we can query the interaction
    let interactions_result: Vec<_> = interactions()
        .range(&deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    
    assert_eq!(interactions_result.len(), 1);
    assert_eq!(interactions_result[0].1.interaction.participants.len(), 2);
    assert_eq!(interactions_result[0].1.interaction.interaction_type, interaction_type);
}

#[test]
fn test_log_interaction_unauthorized() {
    let (mut deps, env) = setup_contract();
    
    let participants = vec![AGENT1.to_string(), AGENT2.to_string()];
    let interaction_type = "conversation".to_string();
    let metadata = create_test_metadata();
    
    let msg = ExecuteMsg::LogInteraction {
        participants,
        interaction_type,
        metadata,
    };
    
    // Try to log interaction as AGENT3 who is not a participant
    let info = mock_info(AGENT3, &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::Unauthorized {} => {},
        _ => panic!("Expected Unauthorized error"),
    }
}

#[test]
fn test_log_interaction_invalid_participants() {
    let (mut deps, env) = setup_contract();
    
    // Test empty participants
    let msg = ExecuteMsg::LogInteraction {
        participants: vec![],
        interaction_type: "conversation".to_string(),
        metadata: create_test_metadata(),
    };
    
    let info = mock_info(AGENT1, &[]);
    let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    
    match err {
        ContractError::InvalidParticipants { reason } => {
            assert!(reason.contains("At least one participant"));
        },
        _ => panic!("Expected InvalidParticipants error"),
    }
    
    // Test duplicate participants
    let msg = ExecuteMsg::LogInteraction {
        participants: vec![AGENT1.to_string(), AGENT1.to_string()],
        interaction_type: "conversation".to_string(),
        metadata: create_test_metadata(),
    };
    
    let info = mock_info(AGENT1, &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::InvalidParticipants { reason } => {
            assert!(reason.contains("Duplicate participant"));
        },
        _ => panic!("Expected InvalidParticipants error"),
    }
}

#[test]
fn test_log_interaction_invalid_type() {
    let (mut deps, env) = setup_contract();
    
    let participants = vec![AGENT1.to_string(), AGENT2.to_string()];
    let interaction_type = "invalid_type".to_string();
    let metadata = create_test_metadata();
    
    let msg = ExecuteMsg::LogInteraction {
        participants,
        interaction_type: interaction_type.clone(),
        metadata,
    };
    
    let info = mock_info(AGENT1, &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::InvalidInteractionType { interaction_type: invalid_type } => {
            assert_eq!(invalid_type, interaction_type);
        },
        _ => panic!("Expected InvalidInteractionType error"),
    }
}

#[test]
fn test_log_interaction_invalid_metadata() {
    let (mut deps, env) = setup_contract();
    
    let participants = vec![AGENT1.to_string(), AGENT2.to_string()];
    let interaction_type = "conversation".to_string();
    
    // Test invalid duration (zero)
    let metadata = InteractionMetadata {
        duration: Some(0),
        outcome: None,
        context: None,
    };
    
    let msg = ExecuteMsg::LogInteraction {
        participants,
        interaction_type,
        metadata,
    };
    
    let info = mock_info(AGENT1, &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::MetadataValidationFailed { reason } => {
            assert!(reason.contains("Duration cannot be zero"));
        },
        _ => panic!("Expected MetadataValidationFailed error"),
    }
}

#[test]
fn test_verify_interaction() {
    let (mut deps, env) = setup_contract();
    
    // First log an interaction
    let participants = vec![AGENT1.to_string(), AGENT2.to_string()];
    let interaction_type = "conversation".to_string();
    let metadata = create_test_metadata();
    
    let msg = ExecuteMsg::LogInteraction {
        participants,
        interaction_type,
        metadata,
    };
    
    let info = mock_info(AGENT1, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Extract interaction hash from response
    let interaction_hash = res.attributes.iter()
        .find(|attr| attr.key == "interaction_hash")
        .unwrap()
        .value
        .clone();
    
    // Now verify the interaction
    let verify_msg = ExecuteMsg::VerifyInteraction {
        interaction_hash: interaction_hash.clone(),
    };
    
    let info = mock_info("anyone", &[]);
    let res = execute(deps.as_mut(), env, info, verify_msg).unwrap();
    
    assert_eq!(res.attributes[0].value, "verify_interaction");
    assert_eq!(res.attributes[1].value, interaction_hash);
    assert_eq!(res.attributes[2].value, "true");
    assert_eq!(res.attributes[3].value, "passed");
}

#[test]
fn test_verify_interaction_not_found() {
    let (mut deps, env) = setup_contract();
    
    let fake_hash = "a".repeat(64);
    let msg = ExecuteMsg::VerifyInteraction {
        interaction_hash: fake_hash.clone(),
    };
    
    let info = mock_info("anyone", &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::InteractionNotFound { interaction_hash } => {
            assert_eq!(interaction_hash, fake_hash);
        },
        _ => panic!("Expected InteractionNotFound error"),
    }
}

#[test]
fn test_verify_interaction_invalid_hash() {
    let (mut deps, env) = setup_contract();
    
    let invalid_hash = "invalid_hash".to_string();
    let msg = ExecuteMsg::VerifyInteraction {
        interaction_hash: invalid_hash,
    };
    
    let info = mock_info("anyone", &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::InvalidInteractionHash {} => {},
        _ => panic!("Expected InvalidInteractionHash error"),
    }
}

#[test]
fn test_update_interaction_metadata() {
    let (mut deps, env) = setup_contract();
    
    // First log an interaction
    let participants = vec![AGENT1.to_string(), AGENT2.to_string()];
    let interaction_type = "conversation".to_string();
    let metadata = create_test_metadata();
    
    let msg = ExecuteMsg::LogInteraction {
        participants,
        interaction_type,
        metadata,
    };
    
    let info = mock_info(AGENT1, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Extract interaction hash from response
    let interaction_hash = res.attributes.iter()
        .find(|attr| attr.key == "interaction_hash")
        .unwrap()
        .value
        .clone();
    
    // Update metadata
    let new_metadata = InteractionMetadata {
        duration: Some(600), // 10 minutes
        outcome: Some("updated outcome".to_string()),
        context: Some("updated context".to_string()),
    };
    
    let update_msg = ExecuteMsg::UpdateInteractionMetadata {
        interaction_hash: interaction_hash.clone(),
        metadata: new_metadata,
    };
    
    let info = mock_info(AGENT1, &[]);
    let res = execute(deps.as_mut(), env, info, update_msg).unwrap();
    
    assert_eq!(res.attributes[0].value, "update_interaction_metadata");
    assert_eq!(res.attributes[1].value, interaction_hash);
    assert_eq!(res.attributes[3].value, AGENT1);
}

#[test]
fn test_update_interaction_metadata_unauthorized() {
    let (mut deps, env) = setup_contract();
    
    // First log an interaction
    let participants = vec![AGENT1.to_string(), AGENT2.to_string()];
    let interaction_type = "conversation".to_string();
    let metadata = create_test_metadata();
    
    let msg = ExecuteMsg::LogInteraction {
        participants,
        interaction_type,
        metadata,
    };
    
    let info = mock_info(AGENT1, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Extract interaction hash from response
    let interaction_hash = res.attributes.iter()
        .find(|attr| attr.key == "interaction_hash")
        .unwrap()
        .value
        .clone();
    
    // Try to update metadata as non-participant
    let new_metadata = InteractionMetadata {
        duration: Some(600),
        outcome: Some("unauthorized update".to_string()),
        context: None,
    };
    
    let update_msg = ExecuteMsg::UpdateInteractionMetadata {
        interaction_hash,
        metadata: new_metadata,
    };
    
    let info = mock_info(AGENT3, &[]); // AGENT3 is not a participant
    let err = execute(deps.as_mut(), env, info, update_msg).unwrap_err();
    
    match err {
        ContractError::Unauthorized {} => {},
        _ => panic!("Expected Unauthorized error"),
    }
}

#[test]
fn test_query_get_interaction() {
    let (mut deps, env) = setup_contract();
    
    // First log an interaction
    let participants = vec![AGENT1.to_string(), AGENT2.to_string()];
    let interaction_type = "conversation".to_string();
    let metadata = create_test_metadata();
    
    let msg = ExecuteMsg::LogInteraction {
        participants: participants.clone(),
        interaction_type: interaction_type.clone(),
        metadata: metadata.clone(),
    };
    
    let info = mock_info(AGENT1, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Extract interaction hash from response
    let interaction_hash = res.attributes.iter()
        .find(|attr| attr.key == "interaction_hash")
        .unwrap()
        .value
        .clone();
    
    // Query the interaction
    let query_msg = QueryMsg::GetInteraction {
        interaction_hash: interaction_hash.clone(),
    };
    
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let response: InteractionResponse = from_json_binary(&res).unwrap();
    
    assert!(response.interaction.is_some());
    let interaction = response.interaction.unwrap();
    assert_eq!(interaction.participants.len(), 2);
    assert_eq!(interaction.interaction_type, interaction_type);
    assert_eq!(interaction.metadata.duration, metadata.duration);
}

#[test]
fn test_query_get_interaction_not_found() {
    let (deps, env) = setup_contract();
    
    let fake_hash = "a".repeat(64);
    let query_msg = QueryMsg::GetInteraction {
        interaction_hash: fake_hash,
    };
    
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let response: InteractionResponse = from_json_binary(&res).unwrap();
    
    assert!(response.interaction.is_none());
}

#[test]
fn test_query_get_interaction_history() {
    let (mut deps, env) = setup_contract();
    
    // Log multiple interactions for AGENT1
    for i in 0..3 {
        let participants = vec![AGENT1.to_string(), format!("agent{}", i + 2)];
        let interaction_type = "conversation".to_string();
        let metadata = create_test_metadata();
        
        let msg = ExecuteMsg::LogInteraction {
            participants,
            interaction_type,
            metadata,
        };
        
        let info = mock_info(AGENT1, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }
    
    // Query interaction history for AGENT1
    let query_msg = QueryMsg::GetInteractionHistory {
        agent_address: AGENT1.to_string(),
        start_after: None,
        limit: None,
    };
    
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let response: InteractionsResponse = from_json_binary(&res).unwrap();
    
    assert_eq!(response.interactions.len(), 3);
    
    // All interactions should involve AGENT1
    for interaction in response.interactions {
        assert!(interaction.participants.contains(&Addr::unchecked(AGENT1)));
    }
}

#[test]
fn test_query_get_interactions_between() {
    let (mut deps, env) = setup_contract();
    
    // Log interaction between AGENT1 and AGENT2
    let participants = vec![AGENT1.to_string(), AGENT2.to_string()];
    let interaction_type = "conversation".to_string();
    let metadata = create_test_metadata();
    
    let msg = ExecuteMsg::LogInteraction {
        participants,
        interaction_type,
        metadata,
    };
    
    let info = mock_info(AGENT1, &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Log interaction between AGENT1 and AGENT3 (should not appear in results)
    let participants = vec![AGENT1.to_string(), AGENT3.to_string()];
    let interaction_type = "task".to_string();
    let metadata = create_test_metadata();
    
    let msg = ExecuteMsg::LogInteraction {
        participants,
        interaction_type,
        metadata,
    };
    
    let info = mock_info(AGENT1, &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Query interactions between AGENT1 and AGENT2
    let query_msg = QueryMsg::GetInteractionsBetween {
        agent1: AGENT1.to_string(),
        agent2: AGENT2.to_string(),
        start_after: None,
        limit: None,
    };
    
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let response: InteractionsResponse = from_json_binary(&res).unwrap();
    
    assert_eq!(response.interactions.len(), 1);
    let interaction = &response.interactions[0];
    assert!(interaction.participants.contains(&Addr::unchecked(AGENT1)));
    assert!(interaction.participants.contains(&Addr::unchecked(AGENT2)));
    assert_eq!(interaction.interaction_type, "conversation");
}

#[test]
fn test_query_get_recent_interactions() {
    let (mut deps, env) = setup_contract();
    
    // Log multiple interactions
    for i in 0..5 {
        let participants = vec![AGENT1.to_string(), format!("agent{}", i + 2)];
        let interaction_type = "conversation".to_string();
        let metadata = create_test_metadata();
        
        let msg = ExecuteMsg::LogInteraction {
            participants,
            interaction_type,
            metadata,
        };
        
        let info = mock_info(AGENT1, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }
    
    // Query recent interactions with limit
    let query_msg = QueryMsg::GetRecentInteractions {
        start_after: None,
        limit: Some(3),
    };
    
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let response: InteractionsResponse = from_json_binary(&res).unwrap();
    
    assert_eq!(response.interactions.len(), 3);
}

#[test]
fn test_query_verify_interaction_exists() {
    let (mut deps, env) = setup_contract();
    
    // Log an interaction
    let participants = vec![AGENT1.to_string(), AGENT2.to_string()];
    let interaction_type = "conversation".to_string();
    let metadata = create_test_metadata();
    
    let msg = ExecuteMsg::LogInteraction {
        participants,
        interaction_type,
        metadata,
    };
    
    let info = mock_info(AGENT1, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Extract interaction hash from response
    let interaction_hash = res.attributes.iter()
        .find(|attr| attr.key == "interaction_hash")
        .unwrap()
        .value
        .clone();
    
    // Query verification status
    let query_msg = QueryMsg::VerifyInteractionExists {
        interaction_hash: interaction_hash.clone(),
    };
    
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let response: VerificationResponse = from_json_binary(&res).unwrap();
    
    // Should exist but not be verified yet
    assert!(!response.verified); // Not verified until explicitly verified
    assert!(response.details.is_some());
    assert!(response.details.unwrap().contains("not verified"));
}

#[test]
fn test_interaction_hash_generation() {
    let participants = vec![Addr::unchecked(AGENT1), Addr::unchecked(AGENT2)];
    let interaction_type = "conversation";
    let timestamp = Timestamp::from_seconds(1234567890);
    let metadata = create_test_metadata();
    
    let hash1 = generate_interaction_hash(&participants, interaction_type, &timestamp, &metadata);
    let hash2 = generate_interaction_hash(&participants, interaction_type, &timestamp, &metadata);
    
    // Same inputs should produce same hash
    assert_eq!(hash1, hash2);
    
    // Hash should be 64 characters (SHA256 hex)
    assert_eq!(hash1.len(), 64);
    
    // Different inputs should produce different hash
    let different_metadata = InteractionMetadata {
        duration: Some(600),
        outcome: Some("different".to_string()),
        context: Some("different context".to_string()),
    };
    
    let hash3 = generate_interaction_hash(&participants, interaction_type, &timestamp, &different_metadata);
    assert_ne!(hash1, hash3);
}

#[test]
fn test_validate_interaction_hash() {
    // Valid hash
    let valid_hash = "a".repeat(64);
    assert!(validate_interaction_hash(&valid_hash).is_ok());
    
    // Invalid length
    let short_hash = "a".repeat(32);
    assert!(validate_interaction_hash(&short_hash).is_err());
    
    let long_hash = "a".repeat(128);
    assert!(validate_interaction_hash(&long_hash).is_err());
    
    // Invalid characters
    let invalid_hash = "g".repeat(64); // 'g' is not a valid hex character
    assert!(validate_interaction_hash(&invalid_hash).is_err());
}

#[test]
fn test_validate_participants() {
    let config = crate::state::Config::default();
    
    // Valid participants
    let valid_participants = vec![AGENT1.to_string(), AGENT2.to_string()];
    let result = validate_participants(&valid_participants, &config);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
    
    // Empty participants
    let empty_participants = vec![];
    let result = validate_participants(&empty_participants, &config);
    assert!(result.is_err());
    
    // Duplicate participants
    let duplicate_participants = vec![AGENT1.to_string(), AGENT1.to_string()];
    let result = validate_participants(&duplicate_participants, &config);
    assert!(result.is_err());
    
    // Too many participants
    let too_many_participants: Vec<String> = (0..15).map(|i| format!("agent{}", i)).collect();
    let result = validate_participants(&too_many_participants, &config);
    assert!(result.is_err());
}

#[test]
fn test_validate_interaction_type() {
    let config = crate::state::Config::default();
    
    // Valid types
    for valid_type in &config.valid_interaction_types {
        assert!(validate_interaction_type(valid_type, &config).is_ok());
    }
    
    // Invalid type
    assert!(validate_interaction_type("invalid_type", &config).is_err());
    
    // Empty type
    assert!(validate_interaction_type("", &config).is_err());
}

#[test]
fn test_validate_metadata() {
    // Valid metadata
    let valid_metadata = create_test_metadata();
    assert!(validate_metadata(&valid_metadata).is_ok());
    
    // Invalid duration (zero)
    let invalid_duration = InteractionMetadata {
        duration: Some(0),
        outcome: None,
        context: None,
    };
    assert!(validate_metadata(&invalid_duration).is_err());
    
    // Invalid duration (too long)
    let too_long_duration = InteractionMetadata {
        duration: Some(25 * 60 * 60), // 25 hours
        outcome: None,
        context: None,
    };
    assert!(validate_metadata(&too_long_duration).is_err());
    
    // Outcome too long
    let long_outcome = InteractionMetadata {
        duration: None,
        outcome: Some("a".repeat(501)),
        context: None,
    };
    assert!(validate_metadata(&long_outcome).is_err());
    
    // Context too long
    let long_context = InteractionMetadata {
        duration: None,
        outcome: None,
        context: Some("a".repeat(1001)),
    };
    assert!(validate_metadata(&long_context).is_err());
}

#[test]
fn test_validate_pagination() {
    let config = crate::state::Config::default();
    
    // Valid limits
    assert_eq!(validate_pagination(Some(10), &config).unwrap(), 10);
    assert_eq!(validate_pagination(Some(50), &config).unwrap(), 50);
    assert_eq!(validate_pagination(None, &config).unwrap(), 50); // Default
    
    // Invalid limits
    assert!(validate_pagination(Some(0), &config).is_err());
    assert!(validate_pagination(Some(101), &config).is_err()); // Exceeds max
}

#[test]
fn test_verify_interaction_integrity() {
    let participants = vec![Addr::unchecked(AGENT1), Addr::unchecked(AGENT2)];
    let interaction_type = "conversation".to_string();
    let timestamp = Timestamp::from_seconds(1234567890);
    let metadata = create_test_metadata();
    
    let interaction = agent_karma_contracts::types::Interaction {
        id: "test_id".to_string(),
        participants: participants.clone(),
        interaction_type: interaction_type.clone(),
        timestamp,
        block_height: 12345,
        metadata: metadata.clone(),
    };
    
    let correct_hash = generate_interaction_hash(&participants, &interaction_type, &timestamp, &metadata);
    let incorrect_hash = "b".repeat(64);
    
    // Correct hash should verify
    assert!(verify_interaction_integrity(&interaction, &correct_hash).unwrap());
    
    // Incorrect hash should not verify
    assert!(!verify_interaction_integrity(&interaction, &incorrect_hash).unwrap());
}

#[test]
fn test_sanitize_string() {
    // Normal string should pass through
    let normal = "Hello World 123";
    assert_eq!(sanitize_string(normal), normal);
    
    // String with allowed punctuation
    let with_punctuation = "Hello, World! How are you? (Fine) [Good] {Great} - Nice.";
    assert_eq!(sanitize_string(with_punctuation), with_punctuation);
    
    // String with disallowed characters should be filtered
    let with_bad_chars = "Hello<script>alert('xss')</script>World";
    let sanitized = sanitize_string(with_bad_chars);
    assert!(!sanitized.contains("<script>"));
    assert!(!sanitized.contains("</script>"));
    
    // String with extra whitespace should be trimmed
    let with_whitespace = "  Hello World  ";
    assert_eq!(sanitize_string(with_whitespace), "Hello World");
}

#[test]
fn test_interaction_duplicate_prevention() {
    let (mut deps, env) = setup_contract();
    
    let participants = vec![AGENT1.to_string(), AGENT2.to_string()];
    let interaction_type = "conversation".to_string();
    let metadata = create_test_metadata();
    
    // Log first interaction
    let msg = ExecuteMsg::LogInteraction {
        participants: participants.clone(),
        interaction_type: interaction_type.clone(),
        metadata: metadata.clone(),
    };
    
    let info = mock_info(AGENT1, &[]);
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    
    // Try to log the same interaction again (should fail due to same hash)
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::InteractionAlreadyExists { interaction_hash: _ } => {},
        _ => panic!("Expected InteractionAlreadyExists error"),
    }
}

#[test]
fn test_pagination_limits() {
    let (mut deps, env) = setup_contract();
    
    // Log many interactions
    for i in 0..10 {
        let participants = vec![AGENT1.to_string(), format!("agent{}", i + 2)];
        let interaction_type = "conversation".to_string();
        let metadata = create_test_metadata();
        
        let msg = ExecuteMsg::LogInteraction {
            participants,
            interaction_type,
            metadata,
        };
        
        let info = mock_info(AGENT1, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }
    
    // Test pagination with limit
    let query_msg = QueryMsg::GetInteractionHistory {
        agent_address: AGENT1.to_string(),
        start_after: None,
        limit: Some(5),
    };
    
    let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let response: InteractionsResponse = from_json_binary(&res).unwrap();
    
    assert_eq!(response.interactions.len(), 5);
    
    // Test pagination limit exceeded
    let query_msg = QueryMsg::GetInteractionHistory {
        agent_address: AGENT1.to_string(),
        start_after: None,
        limit: Some(101), // Exceeds max limit
    };
    
    let err = query(deps.as_ref(), env, query_msg).unwrap_err();
    // Should return an error due to pagination limit exceeded
    assert!(err.to_string().contains("Pagination limit exceeded") || 
            err.to_string().contains("limit"));
}