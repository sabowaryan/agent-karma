use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    Addr, Uint128,
};
use agent_karma_contracts::{ProposalStatus};

use crate::{
    contract::{execute, instantiate, query},
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ProposalResponse, VotingPowerResponse, ConfigResponse, VoteResponse, ProposalsResponse},
};

const CREATOR: &str = "creator";
const AGENT1: &str = "agent1";
const AGENT2: &str = "agent2";
const KARMA_CORE: &str = "karma_core";
const AGENT_REGISTRY: &str = "agent_registry";

fn default_instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {
        min_karma_for_proposal: Uint128::from(100u128),
        min_karma_for_voting: Uint128::from(50u128),
        default_voting_period: 86400, // 24 hours
        quorum_threshold: 20, // 20%
        execution_delay: 3600, // 1 hour
        karma_core_address: KARMA_CORE.to_string(),
        agent_registry_address: AGENT_REGISTRY.to_string(),
    }
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();

    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].value, "instantiate");

    // Query config to verify it was saved correctly
    let query_msg = QueryMsg::GetConfig {};
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let config_response: ConfigResponse = cosmwasm_std::from_binary(&res).unwrap();
    
    assert_eq!(config_response.config.min_karma_for_proposal, Uint128::from(100u128));
    assert_eq!(config_response.config.min_karma_for_voting, Uint128::from(50u128));
    assert_eq!(config_response.config.quorum_threshold, 20);
}

#[test]
fn test_instantiate_invalid_quorum() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(CREATOR, &[]);
    let mut msg = default_instantiate_msg();
    msg.quorum_threshold = 101; // Invalid: > 100

    let err = instantiate(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::InvalidProposalData {}));
}

#[test]
fn test_create_proposal_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate contract
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Create proposal
    let info = mock_info(AGENT1, &[]);
    let msg = ExecuteMsg::CreateProposal {
        title: "Test Proposal".to_string(),
        description: "This is a test proposal".to_string(),
        calldata: "test_calldata".to_string(),
        voting_period: Some(86400),
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes[0].value, "create_proposal");
    assert_eq!(res.attributes[1].value, "1"); // proposal_id
    assert_eq!(res.attributes[2].value, AGENT1); // proposer

    // Query the created proposal
    let query_msg = QueryMsg::GetProposal { proposal_id: 1 };
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let proposal_response: ProposalResponse = cosmwasm_std::from_binary(&res).unwrap();
    
    let proposal = proposal_response.proposal.unwrap();
    assert_eq!(proposal.id, 1);
    assert_eq!(proposal.title, "Test Proposal");
    assert_eq!(proposal.proposer, Addr::unchecked(AGENT1));
    assert_eq!(proposal.status, ProposalStatus::Active);
}

#[test]
fn test_create_proposal_empty_title() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate contract
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Try to create proposal with empty title
    let info = mock_info(AGENT1, &[]);
    let msg = ExecuteMsg::CreateProposal {
        title: "".to_string(),
        description: "This is a test proposal".to_string(),
        calldata: "test_calldata".to_string(),
        voting_period: Some(86400),
    };

    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::InvalidProposalData {}));
}

#[test]
fn test_create_proposal_invalid_voting_period() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate contract
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Try to create proposal with invalid voting period (too short)
    let info = mock_info(AGENT1, &[]);
    let msg = ExecuteMsg::CreateProposal {
        title: "Test Proposal".to_string(),
        description: "This is a test proposal".to_string(),
        calldata: "test_calldata".to_string(),
        voting_period: Some(1800), // 30 minutes - too short
    };

    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::InvalidVotingPeriod { min: 3600, max: 604800 }));
}

#[test]
fn test_vote_proposal_success() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    
    // Instantiate contract
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Create proposal
    let info = mock_info(AGENT1, &[]);
    let msg = ExecuteMsg::CreateProposal {
        title: "Test Proposal".to_string(),
        description: "This is a test proposal".to_string(),
        calldata: "test_calldata".to_string(),
        voting_period: Some(86400),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Vote on proposal
    let info = mock_info(AGENT2, &[]);
    let msg = ExecuteMsg::VoteProposal {
        proposal_id: 1,
        support: true,
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes[0].value, "vote_proposal");
    assert_eq!(res.attributes[1].value, "1"); // proposal_id
    assert_eq!(res.attributes[2].value, AGENT2); // voter
    assert_eq!(res.attributes[3].value, "true"); // support

    // Query the vote
    let query_msg = QueryMsg::GetVote {
        proposal_id: 1,
        voter: AGENT2.to_string(),
    };
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let vote_response: VoteResponse = cosmwasm_std::from_binary(&res).unwrap();
    let vote = vote_response.vote.unwrap();
    assert_eq!(vote.proposal_id, 1);
    assert_eq!(vote.voter, Addr::unchecked(AGENT2));
    assert_eq!(vote.support, true);
}

#[test]
fn test_vote_proposal_already_voted() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate contract
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Create proposal
    let info = mock_info(AGENT1, &[]);
    let msg = ExecuteMsg::CreateProposal {
        title: "Test Proposal".to_string(),
        description: "This is a test proposal".to_string(),
        calldata: "test_calldata".to_string(),
        voting_period: Some(86400),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Vote on proposal first time
    let info = mock_info(AGENT2, &[]);
    let msg = ExecuteMsg::VoteProposal {
        proposal_id: 1,
        support: true,
    };
    execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

    // Try to vote again
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::AlreadyVoted {}));
}

#[test]
fn test_vote_proposal_voting_ended() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    
    // Instantiate contract
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Create proposal
    let info = mock_info(AGENT1, &[]);
    let msg = ExecuteMsg::CreateProposal {
        title: "Test Proposal".to_string(),
        description: "This is a test proposal".to_string(),
        calldata: "test_calldata".to_string(),
        voting_period: Some(86400),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Fast forward time past voting deadline
    env.block.time = env.block.time.plus_seconds(86401);

    // Try to vote after deadline
    let info = mock_info(AGENT2, &[]);
    let msg = ExecuteMsg::VoteProposal {
        proposal_id: 1,
        support: true,
    };

    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::VotingPeriodEnded {}));
}

#[test]
fn test_vote_nonexistent_proposal() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate contract
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Try to vote on non-existent proposal
    let info = mock_info(AGENT1, &[]);
    let msg = ExecuteMsg::VoteProposal {
        proposal_id: 999,
        support: true,
    };

    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::ProposalNotFound { id: 999 }));
}

#[test]
fn test_finalize_proposal_success() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    
    // Instantiate contract
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Create proposal
    let info = mock_info(AGENT1, &[]);
    let msg = ExecuteMsg::CreateProposal {
        title: "Test Proposal".to_string(),
        description: "This is a test proposal".to_string(),
        calldata: "test_calldata".to_string(),
        voting_period: Some(86400),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Vote on proposal
    let info = mock_info(AGENT2, &[]);
    let msg = ExecuteMsg::VoteProposal {
        proposal_id: 1,
        support: true,
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Fast forward time past voting deadline
    env.block.time = env.block.time.plus_seconds(86401);

    // Finalize proposal
    let info = mock_info(AGENT1, &[]);
    let msg = ExecuteMsg::FinalizeProposal { proposal_id: 1 };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes[0].value, "finalize_proposal");
    assert_eq!(res.attributes[1].value, "1"); // proposal_id

    // Query the finalized proposal
    let query_msg = QueryMsg::GetProposal { proposal_id: 1 };
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let proposal_response: ProposalResponse = cosmwasm_std::from_binary(&res).unwrap();
    
    let proposal = proposal_response.proposal.unwrap();
    // Status should be either Passed or Failed depending on votes and quorum
    assert!(proposal.status == ProposalStatus::Passed || proposal.status == ProposalStatus::Failed);
}

#[test]
fn test_finalize_proposal_voting_not_ended() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate contract
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Create proposal
    let info = mock_info(AGENT1, &[]);
    let msg = ExecuteMsg::CreateProposal {
        title: "Test Proposal".to_string(),
        description: "This is a test proposal".to_string(),
        calldata: "test_calldata".to_string(),
        voting_period: Some(86400),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Try to finalize before voting period ends
    let info = mock_info(AGENT1, &[]);
    let msg = ExecuteMsg::FinalizeProposal { proposal_id: 1 };

    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::VotingPeriodNotEnded {}));
}

#[test]
fn test_cancel_proposal_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate contract
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Create proposal
    let info = mock_info(AGENT1, &[]);
    let msg = ExecuteMsg::CreateProposal {
        title: "Test Proposal".to_string(),
        description: "This is a test proposal".to_string(),
        calldata: "test_calldata".to_string(),
        voting_period: Some(86400),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Cancel proposal (by proposer)
    let info = mock_info(AGENT1, &[]);
    let msg = ExecuteMsg::CancelProposal { proposal_id: 1 };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes[0].value, "cancel_proposal");
    assert_eq!(res.attributes[1].value, "1"); // proposal_id
    assert_eq!(res.attributes[2].value, AGENT1); // cancelled_by

    // Query the cancelled proposal
    let query_msg = QueryMsg::GetProposal { proposal_id: 1 };
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let proposal_response: ProposalResponse = cosmwasm_std::from_binary(&res).unwrap();
    
    let proposal = proposal_response.proposal.unwrap();
    assert_eq!(proposal.status, ProposalStatus::Failed);
}

#[test]
fn test_cancel_proposal_unauthorized() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate contract
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Create proposal
    let info = mock_info(AGENT1, &[]);
    let msg = ExecuteMsg::CreateProposal {
        title: "Test Proposal".to_string(),
        description: "This is a test proposal".to_string(),
        calldata: "test_calldata".to_string(),
        voting_period: Some(86400),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Try to cancel proposal by different agent
    let info = mock_info(AGENT2, &[]);
    let msg = ExecuteMsg::CancelProposal { proposal_id: 1 };

    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::Unauthorized {}));
}

#[test]
fn test_query_voting_power() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate contract
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Query voting power
    let query_msg = QueryMsg::GetVotingPower {
        agent_address: AGENT1.to_string(),
    };
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let voting_power_response: VotingPowerResponse = cosmwasm_std::from_binary(&res).unwrap();
    
    // With placeholder karma of 500, voting power should be sqrt(500) ≈ 22
    assert_eq!(voting_power_response.karma_score, Uint128::from(500u128));
    assert_eq!(voting_power_response.voting_power, Uint128::from(22u128));
}

#[test]
fn test_query_nonexistent_proposal() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate contract
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Query non-existent proposal
    let query_msg = QueryMsg::GetProposal { proposal_id: 999 };
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let proposal_response: ProposalResponse = cosmwasm_std::from_binary(&res).unwrap();
    
    assert!(proposal_response.proposal.is_none());
}

#[test]
fn test_voting_power_calculation() {
    // Test square root voting power calculation
    use crate::contract::calculate_voting_power;
    
    assert_eq!(calculate_voting_power(Uint128::from(100u128)), Uint128::from(10u128));
    assert_eq!(calculate_voting_power(Uint128::from(400u128)), Uint128::from(20u128));
    assert_eq!(calculate_voting_power(Uint128::from(900u128)), Uint128::from(30u128));
    assert_eq!(calculate_voting_power(Uint128::from(0u128)), Uint128::from(0u128));
}

#[test]
fn test_multiple_proposals() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate contract
    let info = mock_info(CREATOR, &[]);
    let msg = default_instantiate_msg();
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Create multiple proposals
    for i in 1..=3 {
        let info = mock_info(AGENT1, &[]);
        let msg = ExecuteMsg::CreateProposal {
            title: format!("Test Proposal {}", i),
            description: format!("This is test proposal number {}", i),
            calldata: format!("test_calldata_{}", i),
            voting_period: Some(86400),
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }

    // Query all proposals
    let query_msg = QueryMsg::GetAllProposals {
        start_after: None,
        limit: None,
    };
    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let proposals_response: ProposalsResponse = cosmwasm_std::from_binary(&res).unwrap();
    assert_eq!(proposals_response.proposals.len(), 3);
    assert_eq!(proposals_response.proposals[0].title, "Test Proposal 1");
    assert_eq!(proposals_response.proposals[1].title, "Test Proposal 2");
    assert_eq!(proposals_response.proposals[2].title, "Test Proposal 3");
}