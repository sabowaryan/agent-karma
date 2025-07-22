//! Unit tests for Agent-Karma smart contract interfaces and data structures
//!
//! This module contains comprehensive tests for all the core components
//! of the Agent-Karma system to ensure correctness and reliability.

#[cfg(test)]
mod tests {
    
    use crate::errors::ContractError;
    use crate::errors::validation::*;
    use crate::events::{AgentRegistryEvents, InteractionLoggerEvents, KarmaCoreEvents};
    use crate::types::{
        Agent, AgentMetadata, Interaction, InteractionMetadata, KarmaCalculation, KarmaConfig,
        KarmaFactors, OracleData, Proposal, ProposalStatus, Rating, Vote,
    };
    use cosmwasm_std::{Addr, Timestamp, Uint128};

    /// Test agent metadata creation and validation
    #[test]
    fn test_agent_metadata_creation() {
        let metadata = AgentMetadata {
            name: "TestAgent".to_string(),
            description: "A test AI agent".to_string(),
            framework: "ElizaOS".to_string(),
            version: "1.0.0".to_string(),
            ipfs_hash: Some("QmTest123".to_string()),
        };

        assert_eq!(metadata.name, "TestAgent");
        assert_eq!(metadata.framework, "ElizaOS");
        assert!(metadata.ipfs_hash.is_some());
    }

    /// Test agent creation with all required fields
    #[test]
    fn test_agent_creation() {
        let metadata = AgentMetadata {
            name: "TestAgent".to_string(),
            description: "A test AI agent".to_string(),
            framework: "ElizaOS".to_string(),
            version: "1.0.0".to_string(),
            ipfs_hash: None,
        };

        let agent = Agent {
            address: Addr::unchecked("sei1test123"),
            registration_date: Timestamp::from_seconds(1640995200),
            metadata,
            karma_score: Uint128::new(100),
            interaction_count: 5,
            ratings_received: 3,
        };

        assert_eq!(agent.address, "sei1test123");
        assert_eq!(agent.karma_score, Uint128::new(100));
        assert_eq!(agent.interaction_count, 5);
        assert_eq!(agent.ratings_received, 3);
    }

    /// Test rating creation and validation
    #[test]
    fn test_rating_creation() {
        let rating = Rating {
            id: "rating_001".to_string(),
            rater_address: Addr::unchecked("sei1rater123"),
            rated_address: Addr::unchecked("sei1rated456"),
            score: 8,
            feedback: Some("Great collaboration!".to_string()),
            interaction_hash: "interaction_hash_123".to_string(),
            timestamp: Timestamp::from_seconds(1640995200),
            block_height: 12345,
        };

        assert_eq!(rating.score, 8);
        assert!(rating.feedback.is_some());
        assert_eq!(rating.interaction_hash, "interaction_hash_123");
    }

    /// Test interaction creation with metadata
    #[test]
    fn test_interaction_creation() {
        let metadata = InteractionMetadata {
            duration: Some(300), // 5 minutes
            outcome: Some("successful".to_string()),
            context: Some("Task collaboration".to_string()),
        };

        let interaction = Interaction {
            id: "interaction_001".to_string(),
            participants: vec![Addr::unchecked("sei1agent1"), Addr::unchecked("sei1agent2")],
            interaction_type: "collaboration".to_string(),
            timestamp: Timestamp::from_seconds(1640995200),
            block_height: 12345,
            metadata,
        };

        assert_eq!(interaction.participants.len(), 2);
        assert_eq!(interaction.interaction_type, "collaboration");
        assert!(interaction.metadata.duration.is_some());
    }

    /// Test karma calculation structure
    #[test]
    fn test_karma_calculation() {
        let factors = KarmaFactors {
            average_rating: "7.5".to_string(),
            rating_count: 10,
            interaction_frequency: Uint128::new(50),
            time_decay: "0.95".to_string(),
            external_factors: Some(Uint128::new(20)),
        };

        let calculation = KarmaCalculation {
            agent_address: Addr::unchecked("sei1agent123"),
            current_score: Uint128::new(150),
            previous_score: Uint128::new(120),
            factors,
            last_updated: Timestamp::from_seconds(1640995200),
            calculation_hash: "calc_hash_123".to_string(),
        };

        assert_eq!(calculation.current_score, Uint128::new(150));
        assert_eq!(calculation.previous_score, Uint128::new(120));
        assert_eq!(calculation.factors.rating_count, 10);
    }

    /// Test proposal creation and status
    #[test]
    fn test_proposal_creation() {
        let proposal = Proposal {
            id: 1,
            title: "Update Karma Algorithm".to_string(),
            description: "Proposal to update the karma calculation algorithm".to_string(),
            proposer: Addr::unchecked("sei1proposer123"),
            calldata: "encoded_function_call".to_string(),
            created_at: Timestamp::from_seconds(1640995200),
            voting_deadline: Timestamp::from_seconds(1641081600), // 24 hours later
            executed: false,
            votes_for: Uint128::new(500),
            votes_against: Uint128::new(200),
            quorum_required: Uint128::new(1000),
            status: ProposalStatus::Active,
        };

        assert_eq!(proposal.id, 1);
        assert_eq!(proposal.status, ProposalStatus::Active);
        assert!(!proposal.executed);
        assert_eq!(proposal.votes_for, Uint128::new(500));
    }

    /// Test vote creation
    #[test]
    fn test_vote_creation() {
        let vote = Vote {
            proposal_id: 1,
            voter: Addr::unchecked("sei1voter123"),
            support: true,
            voting_power: Uint128::new(100),
            timestamp: Timestamp::from_seconds(1640995200),
            block_height: 12345,
        };

        assert_eq!(vote.proposal_id, 1);
        assert!(vote.support);
        assert_eq!(vote.voting_power, Uint128::new(100));
    }

    /// Test oracle data structure
    #[test]
    fn test_oracle_data() {
        let oracle_data = OracleData {
            provider: Addr::unchecked("sei1oracle123"),
            data_type: "performance_metrics".to_string(),
            data: "{\"response_time\": 250, \"success_rate\": 0.95}".to_string(),
            timestamp: Timestamp::from_seconds(1640995200),
            signatures: vec![
                "signature1".to_string(),
                "signature2".to_string(),
                "signature3".to_string(),
            ],
            verified: true,
        };

        assert_eq!(oracle_data.data_type, "performance_metrics");
        assert_eq!(oracle_data.signatures.len(), 3);
        assert!(oracle_data.verified);
    }

    /// Test karma configuration
    #[test]
    fn test_karma_config() {
        let config = KarmaConfig {
            min_karma_for_rating: Uint128::new(10),
            min_karma_for_voting: Uint128::new(50),
            min_karma_for_proposal: Uint128::new(100),
            rating_window: 86400, // 24 hours in seconds
            max_ratings_per_interaction: 1,
            rating_fee: Uint128::new(1),
        };

        assert_eq!(config.min_karma_for_rating, Uint128::new(10));
        assert_eq!(config.rating_window, 86400);
        assert_eq!(config.max_ratings_per_interaction, 1);
    }

    /// Test validation functions
    mod validation_tests {
        use super::*;

        #[test]
        fn test_string_length_validation() {
            // Valid string
            assert!(validate_string_length("valid", 3, 10).is_ok());

            // Too short
            assert!(validate_string_length("ab", 3, 10).is_err());

            // Too long
            assert!(validate_string_length("this_is_too_long", 3, 10).is_err());
        }

        #[test]
        fn test_rating_score_validation() {
            // Valid scores
            assert!(validate_rating_score(1).is_ok());
            assert!(validate_rating_score(5).is_ok());
            assert!(validate_rating_score(10).is_ok());

            // Invalid scores
            assert!(validate_rating_score(0).is_err());
            assert!(validate_rating_score(11).is_err());
        }

        #[test]
        fn test_address_validation() {
            // Valid address
            let addr = Addr::unchecked("sei1test123");
            assert!(validate_address(&addr).is_ok());

            // Empty address should fail
            let empty_addr = Addr::unchecked("");
            assert!(validate_address(&empty_addr).is_err());
        }

        #[test]
        fn test_participants_validation() {
            // Valid participants
            let participants = vec![Addr::unchecked("sei1agent1"), Addr::unchecked("sei1agent2")];
            assert!(validate_participants(&participants).is_ok());

            // Empty participants should fail
            let empty_participants: Vec<Addr> = vec![];
            assert!(validate_participants(&empty_participants).is_err());
        }

        #[test]
        fn test_karma_amount_validation() {
            // Valid amounts
            assert!(validate_karma_amount(1).is_ok());
            assert!(validate_karma_amount(100).is_ok());

            // Zero amount should fail
            assert!(validate_karma_amount(0).is_err());
        }
    }

    /// Test error creation and handling
    mod error_tests {
        use super::*;

        #[test]
        fn test_error_creation() {
            let error = ContractError::insufficient_karma(100, 50);
            match error {
                ContractError::InsufficientKarma {
                    required,
                    available,
                } => {
                    assert_eq!(required, 100);
                    assert_eq!(available, 50);
                }
                _ => panic!("Wrong error type"),
            }
        }

        #[test]
        fn test_agent_not_found_error() {
            let error = ContractError::agent_not_found("sei1test123");
            match error {
                ContractError::AgentNotFound { address } => {
                    assert_eq!(address, "sei1test123");
                }
                _ => panic!("Wrong error type"),
            }
        }

        #[test]
        fn test_invalid_rating_score_error() {
            let error = ContractError::invalid_rating_score(15);
            match error {
                ContractError::InvalidRatingScore { score } => {
                    assert_eq!(score, 15);
                }
                _ => panic!("Wrong error type"),
            }
        }
    }

    /// Test message serialization and deserialization
    mod message_tests {
        use super::*;
        use serde_json;

        #[test]
        fn test_agent_metadata_serialization() {
            let metadata = AgentMetadata {
                name: "TestAgent".to_string(),
                description: "A test agent".to_string(),
                framework: "ElizaOS".to_string(),
                version: "1.0.0".to_string(),
                ipfs_hash: None,
            };

            // Test serialization
            let json = serde_json::to_string(&metadata).unwrap();
            assert!(json.contains("TestAgent"));
            assert!(json.contains("ElizaOS"));

            // Test deserialization
            let deserialized: AgentMetadata = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.name, metadata.name);
            assert_eq!(deserialized.framework, metadata.framework);
        }

        #[test]
        fn test_execute_message_serialization() {
            use crate::messages::agent_registry::ExecuteMsg;

            let metadata = AgentMetadata {
                name: "TestAgent".to_string(),
                description: "A test agent".to_string(),
                framework: "ElizaOS".to_string(),
                version: "1.0.0".to_string(),
                ipfs_hash: None,
            };

            let msg = ExecuteMsg::RegisterAgent { metadata };

            // Test serialization
            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains("register_agent"));
            assert!(json.contains("TestAgent"));

            // Test deserialization
            let deserialized: ExecuteMsg = serde_json::from_str(&json).unwrap();
            match deserialized {
                ExecuteMsg::RegisterAgent { metadata } => {
                    assert_eq!(metadata.name, "TestAgent");
                }
                _ => panic!("Wrong message type"),
            }
        }

        #[test]
        fn test_query_message_serialization() {
            use crate::messages::karma_core::QueryMsg;

            let msg = QueryMsg::GetKarmaScore {
                agent_address: "sei1test123".to_string(),
            };

            // Test serialization
            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains("get_karma_score"));
            assert!(json.contains("sei1test123"));

            // Test deserialization
            let deserialized: QueryMsg = serde_json::from_str(&json).unwrap();
            match deserialized {
                QueryMsg::GetKarmaScore { agent_address } => {
                    assert_eq!(agent_address, "sei1test123");
                }
                _ => panic!("Wrong message type"),
            }
        }
    }

    /// Test event creation and attributes
    mod event_tests {
        use super::*;

        #[test]
        fn test_agent_registered_event() {
            let event = AgentRegistryEvents::agent_registered(
                &Addr::unchecked("sei1test123"),
                "TestAgent",
                "ElizaOS",
                Timestamp::from_seconds(1640995200),
            );

            assert_eq!(event.ty, "agent-registry-agent-registered");

            // Check attributes
            let attributes: Vec<_> = event.attributes.iter().collect();
            assert!(attributes
                .iter()
                .any(|attr| attr.key == "agent_address" && attr.value == "sei1test123"));
            assert!(attributes
                .iter()
                .any(|attr| attr.key == "name" && attr.value == "TestAgent"));
            assert!(attributes
                .iter()
                .any(|attr| attr.key == "framework" && attr.value == "ElizaOS"));
        }

        #[test]
        fn test_karma_updated_event() {
            let event = KarmaCoreEvents::karma_updated(
                &Addr::unchecked("sei1test123"),
                Uint128::new(100),
                Uint128::new(150),
                "calc_hash_123",
                Timestamp::from_seconds(1640995200),
            );

            assert_eq!(event.ty, "karma-core-karma-updated");

            // Check attributes
            let attributes: Vec<_> = event.attributes.iter().collect();
            assert!(attributes
                .iter()
                .any(|attr| attr.key == "agent_address" && attr.value == "sei1test123"));
            assert!(attributes
                .iter()
                .any(|attr| attr.key == "previous_score" && attr.value == "100"));
            assert!(attributes
                .iter()
                .any(|attr| attr.key == "new_score" && attr.value == "150"));
        }

        #[test]
        fn test_interaction_logged_event() {
            let participants = vec![Addr::unchecked("sei1agent1"), Addr::unchecked("sei1agent2")];

            let event = InteractionLoggerEvents::interaction_logged(
                "interaction_001",
                &participants,
                "collaboration",
                Timestamp::from_seconds(1640995200),
            );

            assert_eq!(event.ty, "interaction-logger-interaction-logged");

            // Check attributes
            let attributes: Vec<_> = event.attributes.iter().collect();
            assert!(attributes
                .iter()
                .any(|attr| attr.key == "interaction_id" && attr.value == "interaction_001"));
            assert!(attributes
                .iter()
                .any(|attr| attr.key == "interaction_type" && attr.value == "collaboration"));
            assert!(attributes
                .iter()
                .any(|attr| attr.key == "participants" && attr.value.contains("sei1agent1")));
        }
    }

    /// Performance and gas optimization tests
    mod performance_tests {
        use super::*;

        #[test]
        fn test_large_agent_list_handling() {
            // Test handling of large numbers of agents
            let mut agents = Vec::new();
            for i in 0..1000 {
                let metadata = AgentMetadata {
                    name: format!("Agent{}", i),
                    description: format!("Test agent number {}", i),
                    framework: "ElizaOS".to_string(),
                    version: "1.0.0".to_string(),
                    ipfs_hash: None,
                };

                let agent = Agent {
                    address: Addr::unchecked(format!("sei1agent{}", i)),
                    registration_date: Timestamp::from_seconds(1640995200 + i),
                    metadata,
                    karma_score: Uint128::new(i as u128),
                    interaction_count: i as u64,
                    ratings_received: (i / 2) as u64,
                };

                agents.push(agent);
            }

            assert_eq!(agents.len(), 1000);
            assert_eq!(agents[999].karma_score, Uint128::new(999));
        }

        #[test]
        fn test_karma_calculation_performance() {
            // Test karma calculation with many factors
            let factors = KarmaFactors {
                average_rating: "8.5".to_string(),
                rating_count: 1000,
                interaction_frequency: Uint128::new(500),
                time_decay: "0.98".to_string(),
                external_factors: Some(Uint128::new(100)),
            };

            let calculation = KarmaCalculation {
                agent_address: Addr::unchecked("sei1highkarma"),
                current_score: Uint128::new(5000),
                previous_score: Uint128::new(4800),
                factors,
                last_updated: Timestamp::from_seconds(1640995200),
                calculation_hash: "complex_calc_hash".to_string(),
            };

            // Verify the calculation structure handles large numbers
            assert_eq!(calculation.factors.rating_count, 1000);
            assert_eq!(calculation.current_score, Uint128::new(5000));
        }
    }
}
