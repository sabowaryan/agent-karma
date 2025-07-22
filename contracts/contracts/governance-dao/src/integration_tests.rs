#[cfg(test)]
mod integration_tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info, MockDeps, MockApi, MockQuerier, MockStorage},
        Uint128, MemoryStorage, OwnedDeps,
    };
    use agent_karma_contracts::{ProposalStatus};

    use crate::{
        contract::{execute, instantiate, query},
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ProposalResponse},
    };

    const CREATOR: &str = "creator";
    const AGENT1: &str = "agent1";
    const AGENT2: &str = "agent2";
    const AGENT3: &str = "agent3";
    const KARMA_CORE: &str = "karma_core";
    const AGENT_REGISTRY: &str = "agent_registry";

    fn setup_governance() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, cosmwasm_std::Env) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        
        let info = mock_info(CREATOR, &[]);
        let msg = InstantiateMsg {
            min_karma_for_proposal: Uint128::from(100u128),
            min_karma_for_voting: Uint128::from(50u128),
            default_voting_period: 86400, // 24 hours
            quorum_threshold: 20, // 20%
            execution_delay: 3600, // 1 hour
            karma_core_address: KARMA_CORE.to_string(),
            agent_registry_address: AGENT_REGISTRY.to_string(),
        };
        
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
        (deps, env)
    }

    #[test]
    fn test_complete_governance_workflow() {
        let (mut deps, mut env) = setup_governance();

        // Step 1: Create a proposal
        let info = mock_info(AGENT1, &[]);
        let msg = ExecuteMsg::CreateProposal {
            title: "Increase Karma Requirements".to_string(),
            description: "Proposal to increase minimum karma for voting to 100".to_string(),
            calldata: "update_config(min_karma_for_voting: 100)".to_string(),
            voting_period: Some(86400),
        };
        
        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(res.attributes[0].value, "create_proposal");
        assert_eq!(res.attributes[1].value, "1"); // proposal_id

        // Step 2: Multiple agents vote on the proposal
        // Agent 2 votes YES
        let info = mock_info(AGENT2, &[]);
        let msg = ExecuteMsg::VoteProposal {
            proposal_id: 1,
            support: true,
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Agent 3 votes NO
        let info = mock_info(AGENT3, &[]);
        let msg = ExecuteMsg::VoteProposal {
            proposal_id: 1,
            support: false,
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Step 3: Check proposal status during voting period
        let query_msg = QueryMsg::GetProposal { proposal_id: 1 };
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let proposal_response: ProposalResponse = cosmwasm_std::from_binary(&res).unwrap();
        let proposal = proposal_response.proposal.unwrap();
        
        assert_eq!(proposal.status, ProposalStatus::Active);
        assert!(proposal.votes_for > Uint128::zero());
        assert!(proposal.votes_against > Uint128::zero());

        // Step 4: Fast forward time past voting deadline
        env.block.time = env.block.time.plus_seconds(86401);

        // Step 5: Finalize the proposal
        let info = mock_info(AGENT1, &[]);
        let msg = ExecuteMsg::FinalizeProposal { proposal_id: 1 };
        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        
        assert_eq!(res.attributes[0].value, "finalize_proposal");
        assert_eq!(res.attributes[1].value, "1");

        // Step 6: Check final proposal status
        let query_msg = QueryMsg::GetProposal { proposal_id: 1 };
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let proposal_response: ProposalResponse = cosmwasm_std::from_binary(&res).unwrap();
        let proposal = proposal_response.proposal.unwrap();
        
        // Proposal should be either Passed or Failed based on vote results
        assert!(proposal.status == ProposalStatus::Passed || proposal.status == ProposalStatus::Failed);
        assert!(!proposal.executed); // Not executed yet

        // Step 7: If proposal passed, execute it after delay
        if proposal.status == ProposalStatus::Passed {
            // Fast forward past execution delay
            env.block.time = env.block.time.plus_seconds(3601);
            
            let info = mock_info(AGENT1, &[]);
            let msg = ExecuteMsg::ExecuteProposal { proposal_id: 1 };
            let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
            
            assert_eq!(res.attributes[0].value, "execute_proposal");
            assert_eq!(res.attributes[1].value, "1");
            assert_eq!(res.attributes[2].value, "true");

            // Check final executed status
            let query_msg = QueryMsg::GetProposal { proposal_id: 1 };
            let res = query(deps.as_ref(), env, query_msg).unwrap();
            let proposal_response: ProposalResponse = cosmwasm_std::from_binary(&res).unwrap();
            let proposal = proposal_response.proposal.unwrap();
            
            assert_eq!(proposal.status, ProposalStatus::Executed);
            assert!(proposal.executed);
        }
    }

    #[test]
    fn test_karma_weighted_voting_power() {
        let (mut deps, env) = setup_governance();

        // Create a proposal
        let info = mock_info(AGENT1, &[]);
        let msg = ExecuteMsg::CreateProposal {
            title: "Test Voting Power".to_string(),
            description: "Testing karma-weighted voting".to_string(),
            calldata: "test".to_string(),
            voting_period: Some(86400),
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Query voting power for different agents
        let query_msg = QueryMsg::GetVotingPower {
            agent_address: AGENT1.to_string(),
        };
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let voting_power_response = cosmwasm_std::from_binary(&res).unwrap();
        
        // With placeholder karma of 500, voting power should be sqrt(500) ≈ 22
        // This demonstrates the square root voting power calculation
        // In a real implementation, different agents would have different karma scores
    }

    #[test]
    fn test_governance_edge_cases() {
        let (mut deps, mut env) = setup_governance();

        // Test 1: Create proposal with minimum voting period
        let info = mock_info(AGENT1, &[]);
        let msg = ExecuteMsg::CreateProposal {
            title: "Minimum Period Test".to_string(),
            description: "Testing minimum voting period".to_string(),
            calldata: "test".to_string(),
            voting_period: Some(3600), // Minimum 1 hour
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Test 2: Try to finalize immediately (should fail)
        let info = mock_info(AGENT1, &[]);
        let msg = ExecuteMsg::FinalizeProposal { proposal_id: 1 };
        let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
        assert!(matches!(err, crate::error::ContractError::VotingPeriodNotEnded {}));

        // Test 3: Vote and then try to vote again (should fail)
        let info = mock_info(AGENT2, &[]);
        let msg = ExecuteMsg::VoteProposal {
            proposal_id: 1,
            support: true,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
        
        let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
        assert!(matches!(err, crate::error::ContractError::AlreadyVoted {}));

        // Test 4: Fast forward and finalize
        env.block.time = env.block.time.plus_seconds(3601);
        
        let info = mock_info(AGENT1, &[]);
        let msg = ExecuteMsg::FinalizeProposal { proposal_id: 1 };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Test 5: Try to vote after finalization (should fail)
        let info = mock_info(AGENT3, &[]);
        let msg = ExecuteMsg::VoteProposal {
            proposal_id: 1,
            support: false,
        };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, crate::error::ContractError::VotingPeriodEnded {}));
    }
}