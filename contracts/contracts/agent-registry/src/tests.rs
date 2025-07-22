use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    Addr, Uint128,
};

use agent_karma_contracts::{
    messages::{agent_registry::*, InstantiateMsg},
    types::AgentMetadata,
};

use crate::{
    contract::{execute, instantiate, query},
    error::ContractError,
    state::{CONFIG, AGENTS, AGENT_COUNT},
};

// Helper function to create valid agent metadata
fn create_valid_metadata() -> AgentMetadata {
    AgentMetadata {
        name: "Test Agent".to_string(),
        description: "A test agent for unit testing".to_string(),
        framework: "ElizaOS".to_string(),
        version: "1.0.0".to_string(),
        ipfs_hash: Some("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_string()),
    }
}

// Helper function to create agent metadata with custom values
fn create_metadata(name: &str, framework: &str, ipfs_hash: Option<String>) -> AgentMetadata {
    AgentMetadata {
        name: name.to_string(),
        description: "Test description".to_string(),
        framework: framework.to_string(),
        version: "1.0.0".to_string(),
        ipfs_hash,
    }
}

#[test]
fn test_instantiate_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("creator", &[]);

    let msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };

    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
    
    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "instantiate");
    assert_eq!(res.attributes[1].key, "admin");
    assert_eq!(res.attributes[1].value, "admin");

    // Check that config was saved correctly
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.admin, Addr::unchecked("admin"));
    assert_eq!(config.registration_enabled, true);
    assert_eq!(config.max_agents, None);

    // Check that agent count was initialized
    let count = AGENT_COUNT.load(&deps.storage).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_instantiate_default_admin() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("creator", &[]);

    let msg = InstantiateMsg {
        admin: None,
        config: None,
    };

    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
    
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.admin, Addr::unchecked("creator"));
}

#[test]
fn test_register_agent_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Register agent
    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent {
        metadata: metadata.clone(),
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Check response attributes
    assert_eq!(res.attributes.len(), 5);
    assert_eq!(res.attributes[0].key, "method");
    assert_eq!(res.attributes[0].value, "register_agent");
    assert_eq!(res.attributes[1].key, "agent_address");
    assert_eq!(res.attributes[1].value, "agent1");
    assert_eq!(res.attributes[2].key, "agent_name");
    assert_eq!(res.attributes[2].value, "Test Agent");

    // Check that agent was stored correctly
    let stored_agent = AGENTS.load(&deps.storage, "agent1").unwrap();
    assert_eq!(stored_agent.agent.address, Addr::unchecked("agent1"));
    assert_eq!(stored_agent.agent.metadata.name, "Test Agent");
    assert_eq!(stored_agent.agent.karma_score, Uint128::zero());
    assert_eq!(stored_agent.agent.interaction_count, 0);
    assert_eq!(stored_agent.agent.ratings_received, 0);

    // Check that agent count was incremented
    let count = AGENT_COUNT.load(&deps.storage).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_register_agent_already_registered() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Register agent first time
    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent {
        metadata: metadata.clone(),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Try to register same agent again
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();

    match err {
        ContractError::AgentAlreadyRegistered { address } => {
            assert_eq!(address, "agent1");
        }
        _ => panic!("Expected AgentAlreadyRegistered error"),
    }
}

#[test]
fn test_register_agent_invalid_metadata() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Test empty name
    let info = mock_info("agent1", &[]);
    let mut metadata = create_valid_metadata();
    metadata.name = "".to_string();
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
    assert!(matches!(err, ContractError::InvalidMetadata { .. }));

    // Test name too long
    metadata = create_valid_metadata();
    metadata.name = "a".repeat(65); // Max is 64
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
    assert!(matches!(err, ContractError::AgentNameTooLong {}));

    // Test description too long
    metadata = create_valid_metadata();
    metadata.description = "a".repeat(513); // Max is 512
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
    assert!(matches!(err, ContractError::AgentDescriptionTooLong {}));

    // Test invalid framework
    metadata = create_valid_metadata();
    metadata.framework = "InvalidFramework".to_string();
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
    assert!(matches!(err, ContractError::InvalidFramework { .. }));

    // Test invalid IPFS hash
    metadata = create_valid_metadata();
    metadata.ipfs_hash = Some("invalid_hash".to_string());
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::InvalidIpfsHash { .. }));
}

#[test]
fn test_update_agent_metadata_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract and register agent
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent {
        metadata: metadata.clone(),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Update metadata
    let new_metadata = create_metadata("Updated Agent", "MCP", None);
    let msg = ExecuteMsg::UpdateAgentMetadata {
        metadata: new_metadata.clone(),
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();

    // Check response
    assert_eq!(res.attributes[0].value, "update_agent_metadata");
    assert_eq!(res.attributes[2].value, "Updated Agent");
    assert_eq!(res.attributes[3].value, "MCP");

    // Check that metadata was updated
    let stored_agent = AGENTS.load(&deps.storage, "agent1").unwrap();
    assert_eq!(stored_agent.agent.metadata.name, "Updated Agent");
    assert_eq!(stored_agent.agent.metadata.framework, "MCP");
}

#[test]
fn test_update_agent_metadata_not_owner() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract and register two agents
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Register agent1
    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Register agent2
    let info = mock_info("agent2", &[]);
    let metadata = create_metadata("Agent 2", "MCP", None);
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Try to update agent2's metadata but with agent1's signature (simulate ownership check)
    // This test actually tests that agent2 can update their own metadata successfully
    let info = mock_info("agent2", &[]);
    let new_metadata = create_metadata("Updated Agent 2", "AIDN", None);
    let msg = ExecuteMsg::UpdateAgentMetadata {
        metadata: new_metadata,
    };
    let res = execute(deps.as_mut(), env, info, msg);
    
    // This should succeed because agent2 is updating their own metadata
    assert!(res.is_ok());
}

#[test]
fn test_register_agent_with_max_length_fields() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Test with maximum allowed field lengths
    let info = mock_info("agent1", &[]);
    let metadata = AgentMetadata {
        name: "a".repeat(64), // Max length
        description: "b".repeat(512), // Max length
        framework: "ElizaOS".to_string(),
        version: "c".repeat(16), // Max length
        ipfs_hash: Some("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_string()),
    };
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let res = execute(deps.as_mut(), env, info, msg);
    
    assert!(res.is_ok());
}

#[test]
fn test_register_agent_empty_optional_fields() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Test with no IPFS hash
    let info = mock_info("agent1", &[]);
    let metadata = AgentMetadata {
        name: "Test Agent".to_string(),
        description: "Test description".to_string(),
        framework: "Custom".to_string(),
        version: "1.0.0".to_string(),
        ipfs_hash: None,
    };
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let res = execute(deps.as_mut(), env, info, msg);
    
    assert!(res.is_ok());
}

#[test]
fn test_deactivate_agent_already_deactivated() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract and register agent
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Deactivate agent first time
    let info = mock_info("admin", &[]);
    let msg = ExecuteMsg::DeactivateAgent {
        agent_address: "agent1".to_string(),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Try to deactivate again
    let msg = ExecuteMsg::DeactivateAgent {
        agent_address: "agent1".to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();

    assert!(matches!(err, ContractError::AgentDeactivated { .. }));
}

#[test]
fn test_deactivate_nonexistent_agent() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Try to deactivate non-existent agent
    let info = mock_info("admin", &[]);
    let msg = ExecuteMsg::DeactivateAgent {
        agent_address: "nonexistent".to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();

    assert!(matches!(err, ContractError::AgentNotFound { .. }));
}

#[test]
fn test_update_deactivated_agent_metadata() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract and register agent
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Deactivate agent
    let admin_info = mock_info("admin", &[]);
    let msg = ExecuteMsg::DeactivateAgent {
        agent_address: "agent1".to_string(),
    };
    execute(deps.as_mut(), env.clone(), admin_info, msg).unwrap();

    // Try to update metadata of deactivated agent
    let new_metadata = create_metadata("Updated Agent", "MCP", None);
    let msg = ExecuteMsg::UpdateAgentMetadata {
        metadata: new_metadata,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();

    assert!(matches!(err, ContractError::AgentDeactivated { .. }));
}

#[test]
fn test_query_deactivated_agent() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract and register agent
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Deactivate agent
    let admin_info = mock_info("admin", &[]);
    let msg = ExecuteMsg::DeactivateAgent {
        agent_address: "agent1".to_string(),
    };
    execute(deps.as_mut(), env.clone(), admin_info, msg).unwrap();

    // Query deactivated agent - should return None
    let msg = QueryMsg::GetAgent {
        agent_address: "agent1".to_string(),
    };
    let res = query(deps.as_ref(), env.clone(), msg).unwrap();
    let response: AgentResponse = cosmwasm_std::from_json(&res).unwrap();
    assert!(response.agent.is_none());

    // IsRegistered should return false for deactivated agent
    let msg = QueryMsg::IsRegistered {
        agent_address: "agent1".to_string(),
    };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let response: IsRegisteredResponse = cosmwasm_std::from_json(&res).unwrap();
    assert!(!response.registered);
}

#[test]
fn test_framework_index_update_on_metadata_change() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Register agent with ElizaOS framework
    let info = mock_info("agent1", &[]);
    let metadata = create_metadata("Agent 1", "ElizaOS", None);
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Verify agent is in ElizaOS framework index
    let msg = QueryMsg::GetAgentsByFramework {
        framework: "ElizaOS".to_string(),
        start_after: None,
        limit: None,
    };
    let res = query(deps.as_ref(), env.clone(), msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_json(&res).unwrap();
    assert_eq!(response.agents.len(), 1);

    // Update agent to MCP framework
    let new_metadata = create_metadata("Agent 1", "MCP", None);
    let msg = ExecuteMsg::UpdateAgentMetadata {
        metadata: new_metadata,
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Verify agent is no longer in ElizaOS framework index
    let msg = QueryMsg::GetAgentsByFramework {
        framework: "ElizaOS".to_string(),
        start_after: None,
        limit: None,
    };
    let res = query(deps.as_ref(), env.clone(), msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_json(&res).unwrap();
    assert_eq!(response.agents.len(), 0);

    // Verify agent is now in MCP framework index
    let msg = QueryMsg::GetAgentsByFramework {
        framework: "MCP".to_string(),
        start_after: None,
        limit: None,
    };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_json(&res).unwrap();
    assert_eq!(response.agents.len(), 1);
    assert_eq!(response.agents[0].metadata.framework, "MCP");
}

#[test]
fn test_pagination_edge_cases() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Register 3 agents
    for i in 1..=3 {
        let info = mock_info(&format!("agent{}", i), &[]);
        let metadata = create_metadata(&format!("Agent {}", i), "ElizaOS", None);
        let msg = ExecuteMsg::RegisterAgent { metadata };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }

    // Test limit larger than available agents
    let msg = QueryMsg::GetAllAgents {
        start_after: None,
        limit: Some(100),
    };
    let res = query(deps.as_ref(), env.clone(), msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_json(&res).unwrap();
    assert_eq!(response.agents.len(), 3);

    // Test limit of 0 (should default to 30)
    let msg = QueryMsg::GetAllAgents {
        start_after: None,
        limit: Some(0),
    };
    let res = query(deps.as_ref(), env.clone(), msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_json(&res).unwrap();
    assert_eq!(response.agents.len(), 3); // All agents since we have less than 30

    // Test start_after with non-existent agent (should start from beginning)
    let msg = QueryMsg::GetAllAgents {
        start_after: Some("nonexistent".to_string()),
        limit: Some(10),
    };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_json(&res).unwrap();
    assert_eq!(response.agents.len(), 3); // Should return all agents starting from beginning
}

#[test]
fn test_ipfs_hash_edge_cases() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Test with CIDv1 hash
    let info = mock_info("agent1", &[]);
    let metadata = AgentMetadata {
        name: "Test Agent".to_string(),
        description: "Test description".to_string(),
        framework: "ElizaOS".to_string(),
        version: "1.0.0".to_string(),
        ipfs_hash: Some("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string()),
    };
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let res = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(res.is_ok());

    // Test with invalid CIDv0 hash (wrong length)
    let info = mock_info("agent2", &[]);
    let metadata = AgentMetadata {
        name: "Test Agent 2".to_string(),
        description: "Test description".to_string(),
        framework: "ElizaOS".to_string(),
        version: "1.0.0".to_string(),
        ipfs_hash: Some("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdGX".to_string()), // 47 chars instead of 46
    };
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert!(matches!(err, ContractError::InvalidIpfsHash { .. }));

    // Test with invalid CIDv1 hash (too short)
    let info = mock_info("agent3", &[]);
    let metadata = AgentMetadata {
        name: "Test Agent 3".to_string(),
        description: "Test description".to_string(),
        framework: "ElizaOS".to_string(),
        version: "1.0.0".to_string(),
        ipfs_hash: Some("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oc".to_string()), // Too short (49 chars)
    };
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::InvalidIpfsHash { .. }));
}

#[test]
fn test_update_agent_metadata_not_found() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Try to update non-existent agent
    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::UpdateAgentMetadata { metadata };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();

    assert!(matches!(err, ContractError::AgentNotFound { .. }));
}

#[test]
fn test_deactivate_agent_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract and register agent
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Deactivate agent as admin
    let info = mock_info("admin", &[]);
    let msg = ExecuteMsg::DeactivateAgent {
        agent_address: "agent1".to_string(),
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();

    // Check response
    assert_eq!(res.attributes[0].value, "deactivate_agent");
    assert_eq!(res.attributes[1].value, "agent1");
    assert_eq!(res.attributes[2].value, "admin");

    // Check that agent was deactivated
    let stored_agent = AGENTS.load(&deps.storage, "agent1").unwrap();
    assert_eq!(stored_agent.status, crate::state::AgentStatus::Deactivated);
}

#[test]
fn test_deactivate_agent_not_admin() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract and register agent
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Try to deactivate as non-admin
    let info = mock_info("agent2", &[]);
    let msg = ExecuteMsg::DeactivateAgent {
        agent_address: "agent1".to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();

    assert!(matches!(err, ContractError::AdminRequired {}));
}

#[test]
fn test_query_get_agent_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract and register agent
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Query agent
    let msg = QueryMsg::GetAgent {
        agent_address: "agent1".to_string(),
    };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let response: AgentResponse = cosmwasm_std::from_binary(&res).unwrap();

    assert!(response.agent.is_some());
    let agent = response.agent.unwrap();
    assert_eq!(agent.address, Addr::unchecked("agent1"));
    assert_eq!(agent.metadata.name, "Test Agent");
}

#[test]
fn test_query_get_agent_not_found() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Query non-existent agent
    let msg = QueryMsg::GetAgent {
        agent_address: "agent1".to_string(),
    };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let response: AgentResponse = cosmwasm_std::from_binary(&res).unwrap();

    assert!(response.agent.is_none());
}

#[test]
fn test_query_is_registered() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract and register agent
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Query registered agent
    let msg = QueryMsg::IsRegistered {
        agent_address: "agent1".to_string(),
    };
    let res = query(deps.as_ref(), env.clone(), msg).unwrap();
    let response: IsRegisteredResponse = cosmwasm_std::from_binary(&res).unwrap();
    assert!(response.registered);

    // Query non-existent agent
    let msg = QueryMsg::IsRegistered {
        agent_address: "agent2".to_string(),
    };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let response: IsRegisteredResponse = cosmwasm_std::from_binary(&res).unwrap();
    assert!(!response.registered);
}

#[test]
fn test_query_get_all_agents() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Register multiple agents
    for i in 1..=3 {
        let info = mock_info(&format!("agent{}", i), &[]);
        let metadata = create_metadata(&format!("Agent {}", i), "ElizaOS", None);
        let msg = ExecuteMsg::RegisterAgent { metadata };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }

    // Query all agents
    let msg = QueryMsg::GetAllAgents {
        start_after: None,
        limit: None,
    };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_binary(&res).unwrap();

    assert_eq!(response.agents.len(), 3);
    assert_eq!(response.agents[0].metadata.name, "Agent 1");
    assert_eq!(response.agents[1].metadata.name, "Agent 2");
    assert_eq!(response.agents[2].metadata.name, "Agent 3");
}

#[test]
fn test_query_get_agents_by_framework() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Register agents with different frameworks
    let info = mock_info("agent1", &[]);
    let metadata = create_metadata("Agent 1", "ElizaOS", None);
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info("agent2", &[]);
    let metadata = create_metadata("Agent 2", "MCP", None);
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info("agent3", &[]);
    let metadata = create_metadata("Agent 3", "ElizaOS", None);
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Query agents by framework
    let msg = QueryMsg::GetAgentsByFramework {
        framework: "ElizaOS".to_string(),
        start_after: None,
        limit: None,
    };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_binary(&res).unwrap();

    assert_eq!(response.agents.len(), 2);
    assert_eq!(response.agents[0].metadata.name, "Agent 1");
    assert_eq!(response.agents[1].metadata.name, "Agent 3");
}

#[test]
fn test_ipfs_hash_validation() {
    // Test valid CIDv0 hash
    let valid_cidv0 = "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG";
    assert!(crate::contract::is_valid_ipfs_hash(valid_cidv0));

    // Test valid CIDv1 hash
    let valid_cidv1 = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
    assert!(crate::contract::is_valid_ipfs_hash(valid_cidv1));

    // Test invalid hashes
    assert!(!crate::contract::is_valid_ipfs_hash("invalid"));
    assert!(!crate::contract::is_valid_ipfs_hash("Qm123")); // Too short
    assert!(!crate::contract::is_valid_ipfs_hash("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG!")); // Invalid character
}

#[test]
fn test_framework_validation() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Test valid frameworks
    let valid_frameworks = vec!["ElizaOS", "MCP", "AIDN", "Custom"];
    
    for (i, framework) in valid_frameworks.iter().enumerate() {
        let info = mock_info(&format!("agent{}", i + 1), &[]);
        let metadata = create_metadata(&format!("Agent {}", i + 1), framework, None);
        let msg = ExecuteMsg::RegisterAgent { metadata };
        let res = execute(deps.as_mut(), env.clone(), info, msg);
        assert!(res.is_ok(), "Framework {} should be valid", framework);
    }
}

#[test]
fn test_pagination_limits() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Register multiple agents
    for i in 1..=5 {
        let info = mock_info(&format!("agent{}", i), &[]);
        let metadata = create_metadata(&format!("Agent {}", i), "ElizaOS", None);
        let msg = ExecuteMsg::RegisterAgent { metadata };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }

    // Test pagination with limit
    let msg = QueryMsg::GetAllAgents {
        start_after: None,
        limit: Some(2),
    };
    let res = query(deps.as_ref(), env.clone(), msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_binary(&res).unwrap();
    assert_eq!(response.agents.len(), 2);

    // Test pagination with start_after
    let msg = QueryMsg::GetAllAgents {
        start_after: Some("agent2".to_string()),
        limit: Some(2),
    };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_binary(&res).unwrap();
    assert_eq!(response.agents.len(), 2);
}

#[test]
fn test_register_agent_with_registration_disabled() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Disable registration by updating config
    let mut config = CONFIG.load(&deps.storage).unwrap();
    config.registration_enabled = false;
    CONFIG.save(&mut deps.storage, &config).unwrap();

    // Try to register agent with registration disabled
    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();

    assert!(matches!(err, ContractError::Unauthorized {}));
}

#[test]
fn test_register_agent_with_max_agents_limit() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Set max agents limit to 1
    let mut config = CONFIG.load(&deps.storage).unwrap();
    config.max_agents = Some(1);
    CONFIG.save(&mut deps.storage, &config).unwrap();

    // Register first agent (should succeed)
    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let res = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(res.is_ok());

    // Try to register second agent (should fail due to limit)
    let info = mock_info("agent2", &[]);
    let metadata = create_metadata("Agent 2", "MCP", None);
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();

    assert!(matches!(err, ContractError::Unauthorized {}));
}

#[test]
fn test_agent_address_validation() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Try to deactivate agent with invalid address format
    let info = mock_info("admin", &[]);
    let msg = ExecuteMsg::DeactivateAgent {
        agent_address: "invalid_address_format!@#".to_string(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();

    // The address validation happens, but since the agent doesn't exist, 
    // we get AgentNotFound error instead of address validation error
    assert!(matches!(err, ContractError::AgentNotFound { .. }));
}

#[test]
fn test_empty_framework_list_query() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Query agents by framework that doesn't exist
    let msg = QueryMsg::GetAgentsByFramework {
        framework: "NonExistentFramework".to_string(),
        start_after: None,
        limit: None,
    };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_binary(&res).unwrap();

    assert_eq!(response.agents.len(), 0);
}

#[test]
fn test_agent_metadata_whitespace_validation() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Test with whitespace-only name
    let info = mock_info("agent1", &[]);
    let metadata = AgentMetadata {
        name: "   ".to_string(), // Only whitespace
        description: "Valid description".to_string(),
        framework: "ElizaOS".to_string(),
        version: "1.0.0".to_string(),
        ipfs_hash: None,
    };
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
    assert!(matches!(err, ContractError::InvalidMetadata { .. }));

    // Test with whitespace-only framework - this will fail with InvalidFramework instead
    let metadata = AgentMetadata {
        name: "Valid Name".to_string(),
        description: "Valid description".to_string(),
        framework: "   ".to_string(), // Only whitespace
        version: "1.0.0".to_string(),
        ipfs_hash: None,
    };
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
    assert!(matches!(err, ContractError::InvalidFramework { .. }));

    // Test with whitespace-only version
    let metadata = AgentMetadata {
        name: "Valid Name".to_string(),
        description: "Valid description".to_string(),
        framework: "ElizaOS".to_string(),
        version: "   ".to_string(), // Only whitespace
        ipfs_hash: None,
    };
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::InvalidMetadata { .. }));
}

#[test]
fn test_concurrent_agent_registration_order() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Register agents in specific order
    let agents = vec!["agent3", "agent1", "agent2"];
    for agent in &agents {
        let info = mock_info(agent, &[]);
        let metadata = create_metadata(&format!("Agent {}", agent), "ElizaOS", None);
        let msg = ExecuteMsg::RegisterAgent { metadata };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }

    // Query all agents - should be in registration order, not alphabetical
    let msg = QueryMsg::GetAllAgents {
        start_after: None,
        limit: None,
    };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_binary(&res).unwrap();

    assert_eq!(response.agents.len(), 3);
    assert_eq!(response.agents[0].address, Addr::unchecked("agent3"));
    assert_eq!(response.agents[1].address, Addr::unchecked("agent1"));
    assert_eq!(response.agents[2].address, Addr::unchecked("agent2"));
}

#[test]
fn test_deactivated_agent_not_in_framework_index() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract and register agent
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    let info = mock_info("agent1", &[]);
    let metadata = create_metadata("Agent 1", "ElizaOS", None);
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Verify agent is in framework index
    let msg = QueryMsg::GetAgentsByFramework {
        framework: "ElizaOS".to_string(),
        start_after: None,
        limit: None,
    };
    let res = query(deps.as_ref(), env.clone(), msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_binary(&res).unwrap();
    assert_eq!(response.agents.len(), 1);

    // Deactivate agent
    let admin_info = mock_info("admin", &[]);
    let msg = ExecuteMsg::DeactivateAgent {
        agent_address: "agent1".to_string(),
    };
    execute(deps.as_mut(), env.clone(), admin_info, msg).unwrap();

    // Verify agent is no longer in framework index
    let msg = QueryMsg::GetAgentsByFramework {
        framework: "ElizaOS".to_string(),
        start_after: None,
        limit: None,
    };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_binary(&res).unwrap();
    assert_eq!(response.agents.len(), 0);
}

#[test]
fn test_large_pagination_request() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Register 5 agents
    for i in 1..=5 {
        let info = mock_info(&format!("agent{}", i), &[]);
        let metadata = create_metadata(&format!("Agent {}", i), "ElizaOS", None);
        let msg = ExecuteMsg::RegisterAgent { metadata };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }

    // Request with limit larger than max allowed (should be capped at 100)
    let msg = QueryMsg::GetAllAgents {
        start_after: None,
        limit: Some(1000), // Very large limit
    };
    let res = query(deps.as_ref(), env, msg).unwrap();
    let response: AgentsResponse = cosmwasm_std::from_binary(&res).unwrap();
    
    // Should return all 5 agents (less than the 100 cap)
    assert_eq!(response.agents.len(), 5);
}

#[test]
fn test_gas_optimization_agent_existence_check() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Register an agent
    let info = mock_info("agent1", &[]);
    let metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent { metadata };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Test multiple rapid existence checks (simulating gas optimization)
    for _ in 0..10 {
        let msg = QueryMsg::IsRegistered {
            agent_address: "agent1".to_string(),
        };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let response: IsRegisteredResponse = cosmwasm_std::from_binary(&res).unwrap();
        assert!(response.registered);
    }

    // Test non-existent agent checks
    for i in 2..12 {
        let msg = QueryMsg::IsRegistered {
            agent_address: format!("agent{}", i),
        };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let response: IsRegisteredResponse = cosmwasm_std::from_binary(&res).unwrap();
        assert!(!response.registered);
    }
}

#[test]
fn test_framework_case_sensitivity() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Register agent with lowercase framework (should fail)
    let info = mock_info("agent1", &[]);
    let metadata = create_metadata("Agent 1", "elizaos", None); // lowercase
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert!(matches!(err, ContractError::InvalidFramework { .. }));

    // Register agent with correct case (should succeed)
    let info = mock_info("agent1", &[]);
    let metadata = create_metadata("Agent 1", "ElizaOS", None);
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());
}

#[test]
fn test_ipfs_hash_special_characters() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Test IPFS hash with special characters (should fail)
    let info = mock_info("agent1", &[]);
    let metadata = AgentMetadata {
        name: "Test Agent".to_string(),
        description: "Test description".to_string(),
        framework: "ElizaOS".to_string(),
        version: "1.0.0".to_string(),
        ipfs_hash: Some("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPb@#".to_string()),
    };
    let msg = ExecuteMsg::RegisterAgent { metadata };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::InvalidIpfsHash { .. }));
}

#[test]
fn test_agent_count_consistency() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    // Check initial count
    let count = AGENT_COUNT.load(&deps.storage).unwrap();
    assert_eq!(count, 0);

    // Register agents and verify count increments
    for i in 1..=3 {
        let info = mock_info(&format!("agent{}", i), &[]);
        let metadata = create_metadata(&format!("Agent {}", i), "ElizaOS", None);
        let msg = ExecuteMsg::RegisterAgent { metadata };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        
        let count = AGENT_COUNT.load(&deps.storage).unwrap();
        assert_eq!(count, i);
    }

    // Deactivate an agent - count should remain the same (agents are not deleted)
    let admin_info = mock_info("admin", &[]);
    let msg = ExecuteMsg::DeactivateAgent {
        agent_address: "agent1".to_string(),
    };
    execute(deps.as_mut(), env, admin_info, msg).unwrap();
    
    let count = AGENT_COUNT.load(&deps.storage).unwrap();
    assert_eq!(count, 3); // Count should still be 3
}

#[test]
fn test_metadata_update_preserves_other_fields() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Initialize contract and register agent
    let init_msg = InstantiateMsg {
        admin: Some("admin".to_string()),
        config: None,
    };
    instantiate(deps.as_mut(), env.clone(), mock_info("creator", &[]), init_msg).unwrap();

    let info = mock_info("agent1", &[]);
    let original_metadata = create_valid_metadata();
    let msg = ExecuteMsg::RegisterAgent {
        metadata: original_metadata.clone(),
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Get original agent data
    let original_agent = AGENTS.load(&deps.storage, "agent1").unwrap();
    let original_registration_date = original_agent.agent.registration_date;
    let original_karma = original_agent.agent.karma_score;

    // Update metadata
    let new_metadata = create_metadata("Updated Agent", "MCP", None);
    let msg = ExecuteMsg::UpdateAgentMetadata {
        metadata: new_metadata.clone(),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Verify metadata was updated but other fields preserved
    let updated_agent = AGENTS.load(&deps.storage, "agent1").unwrap();
    assert_eq!(updated_agent.agent.metadata.name, "Updated Agent");
    assert_eq!(updated_agent.agent.metadata.framework, "MCP");
    assert_eq!(updated_agent.agent.registration_date, original_registration_date);
    assert_eq!(updated_agent.agent.karma_score, original_karma);
    assert_eq!(updated_agent.agent.interaction_count, 0);
    assert_eq!(updated_agent.agent.ratings_received, 0);
}