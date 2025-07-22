use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_json, Addr, Timestamp, Uint128};

use agent_karma_contracts::{
    messages::{karma_core::*, InstantiateMsg},
    types::{KarmaConfig, OracleData, Rating},
};

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::state::{CONFIG, KARMA_SCORES, ratings, RATING_TRACKERS};

// Test constants
const ADMIN: &str = "admin";
const AGENT1: &str = "agent1";
const AGENT2: &str = "agent2";
const AGENT3: &str = "agent3";

fn default_karma_config() -> KarmaConfig {
    KarmaConfig {
        min_karma_for_rating: Uint128::from(10u128),
        min_karma_for_voting: Uint128::from(50u128),
        min_karma_for_proposal: Uint128::from(100u128),
        rating_window: 24 * 60 * 60, // 24 hours
        max_ratings_per_interaction: 1,
        rating_fee: Uint128::from(2u128),
    }
}

fn setup_contract() -> (cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, cosmwasm_std::Env) {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    let msg = InstantiateMsg {
        admin: Some(ADMIN.to_string()),
        config: Some(default_karma_config()),
    };
    
    let info = mock_info(ADMIN, &coins(1000, "token"));
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    (deps, env)
}

fn give_initial_karma(deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, agent: &str, karma: u128) {
    use crate::state::KarmaScore;
    
    let karma_score = KarmaScore {
        current_score: Uint128::from(karma),
        previous_score: Uint128::zero(),
        last_updated: mock_env().block.time,
        total_ratings: 0,
        average_rating: "0.0".to_string(),
        interaction_count: 0,
    };
    
    KARMA_SCORES.save(deps.as_mut().storage, agent, &karma_score).unwrap();
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    let msg = InstantiateMsg {
        admin: Some(ADMIN.to_string()),
        config: Some(default_karma_config()),
    };
    
    let info = mock_info(ADMIN, &coins(1000, "token"));
    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
    
    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.attributes[0].value, "instantiate");
    assert_eq!(res.attributes[1].value, ADMIN);
    
    // Check config was saved
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.admin, Addr::unchecked(ADMIN));
    assert_eq!(config.karma_config.min_karma_for_rating, Uint128::from(10u128));
}

#[test]
fn test_submit_rating_success() {
    let (mut deps, env) = setup_contract();
    
    // Give both agents initial karma
    give_initial_karma(&mut deps, AGENT1, 50);
    give_initial_karma(&mut deps, AGENT2, 20);
    
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 8,
        feedback: Some("Great interaction!".to_string()),
        interaction_hash: "a".repeat(64),
    };
    
    let info = mock_info(AGENT1, &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    assert_eq!(res.attributes[0].value, "submit_rating");
    assert_eq!(res.attributes[1].value, AGENT1);
    assert_eq!(res.attributes[2].value, AGENT2);
    assert_eq!(res.attributes[3].value, "8");
    
    // Check that rating was saved
    let ratings_result: Vec<_> = ratings()
        .idx
        .rated_agent
        .prefix(AGENT2.to_string())
        .range(&deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    
    assert_eq!(ratings_result.len(), 1);
    assert_eq!(ratings_result[0].1.rating.score, 8);
    assert_eq!(ratings_result[0].1.rating.rater_address, Addr::unchecked(AGENT1));
    
    // Check that karma was updated for rated agent
    let karma = KARMA_SCORES.load(&deps.storage, AGENT2).unwrap();
    assert!(karma.current_score > Uint128::from(20u128)); // Should have increased
}

#[test]
fn test_submit_rating_invalid_score() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 11, // Invalid score
        feedback: None,
        interaction_hash: "a".repeat(64),
    };
    
    let info = mock_info(AGENT1, &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::InvalidRatingScore { score } => assert_eq!(score, 11),
        _ => panic!("Expected InvalidRatingScore error"),
    }
}

#[test]
fn test_submit_rating_cannot_rate_self() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT1.to_string(), // Same as rater
        score: 8,
        feedback: None,
        interaction_hash: "a".repeat(64),
    };
    
    let info = mock_info(AGENT1, &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::CannotRateSelf {} => {},
        _ => panic!("Expected CannotRateSelf error"),
    }
}

#[test]
fn test_submit_rating_insufficient_karma() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 5); // Below minimum of 10
    
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 8,
        feedback: None,
        interaction_hash: "a".repeat(64),
    };
    
    let info = mock_info(AGENT1, &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::MinimumRequirementsNotMet { reason: _ } => {},
        _ => panic!("Expected MinimumRequirementsNotMet error"),
    }
}

#[test]
fn test_submit_rating_duplicate_prevention() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    give_initial_karma(&mut deps, AGENT2, 20);
    
    let interaction_hash = "a".repeat(64);
    
    // Submit first rating
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 8,
        feedback: None,
        interaction_hash: interaction_hash.clone(),
    };
    
    let info = mock_info(AGENT1, &[]);
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    
    // Try to submit duplicate rating
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 9,
        feedback: None,
        interaction_hash: interaction_hash.clone(),
    };
    
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::RatingAlreadySubmitted { interaction_hash: hash } => {
            assert_eq!(hash, interaction_hash);
        },
        _ => panic!("Expected RatingAlreadySubmitted error"),
    }
}

#[test]
fn test_recalculate_karma() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    give_initial_karma(&mut deps, AGENT2, 20);
    
    // Submit a rating first
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 9,
        feedback: None,
        interaction_hash: "a".repeat(64),
    };
    
    let info = mock_info(AGENT1, &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // Get karma before recalculation
    let karma_before = KARMA_SCORES.load(&deps.storage, AGENT2).unwrap();
    
    // Recalculate karma
    let msg = ExecuteMsg::RecalculateKarma {
        agent_address: AGENT2.to_string(),
    };
    
    let info = mock_info("anyone", &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    assert_eq!(res.attributes[0].value, "recalculate_karma");
    assert_eq!(res.attributes[1].value, AGENT2);
    
    // Check that karma was recalculated
    let karma_after = KARMA_SCORES.load(&deps.storage, AGENT2).unwrap();
    assert_eq!(karma_after.current_score, karma_before.current_score); // Should be same since already calculated
}

#[test]
fn test_update_config_admin_only() {
    let (mut deps, env) = setup_contract();
    
    let new_config = KarmaConfig {
        min_karma_for_rating: Uint128::from(20u128),
        min_karma_for_voting: Uint128::from(100u128),
        min_karma_for_proposal: Uint128::from(200u128),
        rating_window: 48 * 60 * 60, // 48 hours
        max_ratings_per_interaction: 2,
        rating_fee: Uint128::from(5u128),
    };
    
    let msg = ExecuteMsg::UpdateConfig {
        config: new_config.clone(),
    };
    
    // Try with non-admin
    let info = mock_info(AGENT1, &[]);
    let err = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap_err();
    
    match err {
        ContractError::AdminRequired {} => {},
        _ => panic!("Expected AdminRequired error"),
    }
    
    // Try with admin
    let info = mock_info(ADMIN, &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    assert_eq!(res.attributes[0].value, "update_config");
    
    // Check config was updated
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.karma_config.min_karma_for_rating, Uint128::from(20u128));
    assert_eq!(config.karma_config.rating_window, 48 * 60 * 60);
}

#[test]
fn test_update_config_invalid() {
    let (mut deps, env) = setup_contract();
    
    let invalid_config = KarmaConfig {
        min_karma_for_rating: Uint128::from(10u128),
        min_karma_for_voting: Uint128::from(50u128),
        min_karma_for_proposal: Uint128::from(100u128),
        rating_window: 0, // Invalid - cannot be zero
        max_ratings_per_interaction: 1,
        rating_fee: Uint128::from(2u128),
    };
    
    let msg = ExecuteMsg::UpdateConfig {
        config: invalid_config,
    };
    
    let info = mock_info(ADMIN, &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::InvalidKarmaConfig { reason } => {
            assert!(reason.contains("Rating window cannot be zero"));
        },
        _ => panic!("Expected InvalidKarmaConfig error"),
    }
}

#[test]
fn test_process_oracle_data() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 100);
    
    let oracle_data = vec![
        OracleData {
            provider: Addr::unchecked("oracle1"),
            data_type: "performance".to_string(),
            data: "high_performance_data".to_string(),
            timestamp: env.block.time,
            signatures: vec!["sig1".to_string(), "sig2".to_string(), "sig3".to_string()],
            verified: true,
        },
        OracleData {
            provider: Addr::unchecked("oracle2"),
            data_type: "cross_chain".to_string(),
            data: "cross_chain_reputation".to_string(),
            timestamp: env.block.time,
            signatures: vec!["sig1".to_string(), "sig2".to_string(), "sig3".to_string()],
            verified: true,
        },
    ];
    
    let msg = ExecuteMsg::ProcessOracleData {
        agent_address: AGENT1.to_string(),
        oracle_data,
    };
    
    let info = mock_info(ADMIN, &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    assert_eq!(res.attributes[0].value, "process_oracle_data");
    assert_eq!(res.attributes[1].value, AGENT1);
    assert_eq!(res.attributes[2].value, "2");
    
    // Check that karma was recalculated with oracle data
    let karma = KARMA_SCORES.load(&deps.storage, AGENT1).unwrap();
    assert!(karma.current_score >= Uint128::from(100u128)); // Should have bonus from oracle data
}

#[test]
fn test_query_karma_score() {
    let (mut deps, _env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 150);
    
    let msg = QueryMsg::GetKarmaScore {
        agent_address: AGENT1.to_string(),
    };
    
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let response: KarmaScoreResponse = from_json(&res).unwrap();
    
    assert_eq!(response.score, Uint128::from(150u128));
}

#[test]
fn test_query_karma_score_nonexistent() {
    let (deps, _env) = setup_contract();
    
    let msg = QueryMsg::GetKarmaScore {
        agent_address: "nonexistent".to_string(),
    };
    
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let response: KarmaScoreResponse = from_json(&res).unwrap();
    
    assert_eq!(response.score, Uint128::zero());
}

#[test]
fn test_query_agent_ratings() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    give_initial_karma(&mut deps, AGENT2, 20);
    give_initial_karma(&mut deps, AGENT3, 30);
    
    // Submit multiple ratings for AGENT2
    let ratings_data = vec![
        (AGENT1, 8, "rating1"),
        (AGENT3, 9, "rating2"),
    ];
    
    for (rater, score, interaction) in ratings_data {
        let msg = ExecuteMsg::SubmitRating {
            rated_agent: AGENT2.to_string(),
            score,
            feedback: Some(format!("Feedback from {}", rater)),
            interaction_hash: interaction.repeat(64),
        };
        
        let info = mock_info(rater, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }
    
    // Query ratings
    let msg = QueryMsg::GetAgentRatings {
        agent_address: AGENT2.to_string(),
        start_after: None,
        limit: None,
    };
    
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let response: RatingsResponse = from_json(&res).unwrap();
    
    assert_eq!(response.ratings.len(), 2);
    assert_eq!(response.ratings[0].score, 8);
    assert_eq!(response.ratings[1].score, 9);
}

#[test]
fn test_query_leaderboard() {
    let (mut deps, _env) = setup_contract();
    
    // Give different karma scores to agents
    give_initial_karma(&mut deps, AGENT1, 100);
    give_initial_karma(&mut deps, AGENT2, 200);
    give_initial_karma(&mut deps, AGENT3, 150);
    
    // Update leaderboard manually for test
    use crate::state::LEADERBOARD;
    LEADERBOARD.save(&mut deps.storage, 100, &AGENT1.to_string()).unwrap();
    LEADERBOARD.save(&mut deps.storage, 200, &AGENT2.to_string()).unwrap();
    LEADERBOARD.save(&mut deps.storage, 150, &AGENT3.to_string()).unwrap();
    
    let msg = QueryMsg::GetLeaderboard { limit: None };
    
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let response: LeaderboardResponse = from_json(&res).unwrap();
    
    assert_eq!(response.leaderboard.len(), 3);
    // Should be sorted by karma score descending
    assert_eq!(response.leaderboard[0].karma_score, Uint128::from(200u128));
    assert_eq!(response.leaderboard[1].karma_score, Uint128::from(150u128));
    assert_eq!(response.leaderboard[2].karma_score, Uint128::from(100u128));
}

#[test]
fn test_query_config() {
    let (deps, _env) = setup_contract();
    
    let msg = QueryMsg::GetConfig {};
    
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let response: ConfigResponse = from_json(&res).unwrap();
    
    assert_eq!(response.config.min_karma_for_rating, Uint128::from(10u128));
    assert_eq!(response.config.rating_window, 24 * 60 * 60);
    assert_eq!(response.config.rating_fee, Uint128::from(2u128));
}

#[test]
fn test_karma_calculation_time_decay() {
    let (mut deps, mut env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    give_initial_karma(&mut deps, AGENT2, 100);
    
    // Submit a rating
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 8,
        feedback: None,
        interaction_hash: "a".repeat(64),
    };
    
    let info = mock_info(AGENT1, &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let karma_before = KARMA_SCORES.load(&deps.storage, AGENT2).unwrap();
    
    // Advance time by 2 months to trigger time decay
    env.block.time = env.block.time.plus_seconds(60 * 24 * 60 * 60); // 60 days
    
    // Recalculate karma
    let msg = ExecuteMsg::RecalculateKarma {
        agent_address: AGENT2.to_string(),
    };
    
    let info = mock_info("anyone", &[]);
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    let karma_after = KARMA_SCORES.load(&deps.storage, AGENT2).unwrap();
    
    // Karma should be lower due to time decay
    assert!(karma_after.current_score < karma_before.current_score);
}

#[test]
fn test_karma_earning_spending_mechanisms() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 100);
    give_initial_karma(&mut deps, AGENT2, 50);
    
    let initial_karma = KARMA_SCORES.load(&deps.storage, AGENT1).unwrap().current_score;
    
    // Submit rating (should spend karma fee)
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 8,
        feedback: None,
        interaction_hash: "a".repeat(64),
    };
    
    let info = mock_info(AGENT1, &[]);
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    let final_karma = KARMA_SCORES.load(&deps.storage, AGENT1).unwrap().current_score;
    
    // Should have spent rating fee (2 karma)
    assert_eq!(final_karma, initial_karma.checked_sub(Uint128::from(2u128)).unwrap());
}

#[test]
fn test_contextual_modifiers() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    give_initial_karma(&mut deps, AGENT2, 50);
    give_initial_karma(&mut deps, AGENT3, 100);
    
    // Submit multiple high ratings to trigger consistency bonus
    for i in 0..12 {
        let interaction_hash = format!("{}", i).repeat(64);
        let msg = ExecuteMsg::SubmitRating {
            rated_agent: AGENT3.to_string(),
            score: 9, // High rating
            feedback: None,
            interaction_hash,
        };
        
        let rater = if i % 2 == 0 { AGENT1 } else { AGENT2 };
        let info = mock_info(rater, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        
        // Give more karma to raters for subsequent ratings
        give_initial_karma(&mut deps, rater, 50 + (i as u128 * 10));
    }
    
    let final_karma = KARMA_SCORES.load(&deps.storage, AGENT3).unwrap();
    
    // Should have significant karma due to consistency bonus
    assert!(final_karma.current_score > Uint128::from(500u128));
}

// Additional comprehensive edge case tests

#[test]
fn test_karma_earning_from_high_quality_ratings() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 1000); // High karma rater
    give_initial_karma(&mut deps, AGENT2, 50);   // Low karma rater
    give_initial_karma(&mut deps, AGENT3, 100);  // Agent being rated
    
    let initial_karma = KARMA_SCORES.load(&deps.storage, AGENT3).unwrap().current_score;
    
    // High karma agent gives excellent rating
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT3.to_string(),
        score: 10,
        feedback: Some("Excellent work!".to_string()),
        interaction_hash: "high_karma_rating".repeat(4),
    };
    
    let info = mock_info(AGENT1, &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let karma_after_high_rater = KARMA_SCORES.load(&deps.storage, AGENT3).unwrap().current_score;
    
    // Low karma agent gives same rating
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT3.to_string(),
        score: 10,
        feedback: Some("Also excellent!".to_string()),
        interaction_hash: "low_karma_rating".repeat(4),
    };
    
    let info = mock_info(AGENT2, &[]);
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    let final_karma = KARMA_SCORES.load(&deps.storage, AGENT3).unwrap().current_score;
    
    // Rating from high-karma agent should provide more karma boost
    let high_karma_boost = karma_after_high_rater.u128() - initial_karma.u128();
    let low_karma_boost = final_karma.u128() - karma_after_high_rater.u128();
    
    assert!(high_karma_boost > low_karma_boost, 
        "High karma rater should provide more karma boost. High: {}, Low: {}", 
        high_karma_boost, low_karma_boost);
}

#[test]
fn test_karma_penalty_for_poor_ratings() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    give_initial_karma(&mut deps, AGENT2, 200); // Agent with good karma to be penalized
    
    let initial_karma = KARMA_SCORES.load(&deps.storage, AGENT2).unwrap().current_score;
    
    // Submit very poor rating
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 1, // Very poor rating
        feedback: Some("Terrible performance".to_string()),
        interaction_hash: "poor_rating_test".repeat(4),
    };
    
    let info = mock_info(AGENT1, &[]);
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    let final_karma = KARMA_SCORES.load(&deps.storage, AGENT2).unwrap().current_score;
    
    // Karma should have decreased due to poor rating
    assert!(final_karma < initial_karma, 
        "Karma should decrease after poor rating. Initial: {}, Final: {}", 
        initial_karma, final_karma);
}

#[test]
fn test_minimum_karma_balance_enforcement() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 7); // Just above minimum balance
    give_initial_karma(&mut deps, AGENT2, 20);
    
    // Try to submit rating that would bring karma below minimum balance
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 8,
        feedback: None,
        interaction_hash: "minimum_balance_test".repeat(4),
    };
    
    let info = mock_info(AGENT1, &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::MinimumRequirementsNotMet { reason } => {
            assert!(reason.contains("minimum balance"));
        },
        _ => panic!("Expected MinimumRequirementsNotMet error for minimum balance"),
    }
}

#[test]
fn test_karma_cap_enforcement() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    
    // Set agent karma very high (near cap)
    use crate::state::KarmaScore;
    let high_karma = KarmaScore {
        current_score: Uint128::from(9950u128), // Near the 10000 cap
        previous_score: Uint128::zero(),
        last_updated: mock_env().block.time,
        total_ratings: 0,
        average_rating: "0.0".to_string(),
        interaction_count: 0,
    };
    KARMA_SCORES.save(deps.as_mut().storage, AGENT2, &high_karma).unwrap();
    
    // Submit excellent rating to trigger karma earning
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 10,
        feedback: None,
        interaction_hash: "karma_cap_test".repeat(4),
    };
    
    let info = mock_info(AGENT1, &[]);
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    let final_karma = KARMA_SCORES.load(&deps.storage, AGENT2).unwrap().current_score;
    
    // Karma should not exceed the cap
    assert!(final_karma <= Uint128::from(10000u128), 
        "Karma should not exceed cap of 10000. Final: {}", final_karma);
}

#[test]
fn test_time_decay_calculation() {
    let (mut deps, mut env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    give_initial_karma(&mut deps, AGENT2, 100);
    
    // Submit initial rating
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 8,
        feedback: None,
        interaction_hash: "time_decay_test".repeat(4),
    };
    
    let info = mock_info(AGENT1, &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let karma_before_decay = KARMA_SCORES.load(&deps.storage, AGENT2).unwrap().current_score;
    
    // Advance time by 45 days to trigger significant time decay
    env.block.time = env.block.time.plus_seconds(45 * 24 * 60 * 60);
    
    // Recalculate karma to apply time decay
    let msg = ExecuteMsg::RecalculateKarma {
        agent_address: AGENT2.to_string(),
    };
    
    let info = mock_info("anyone", &[]);
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    let karma_after_decay = KARMA_SCORES.load(&deps.storage, AGENT2).unwrap().current_score;
    
    // Karma should be lower due to time decay
    assert!(karma_after_decay < karma_before_decay, 
        "Karma should decrease due to time decay. Before: {}, After: {}", 
        karma_before_decay, karma_after_decay);
}

#[test]
fn test_interaction_frequency_bonus() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 100);
    give_initial_karma(&mut deps, AGENT2, 100);
    give_initial_karma(&mut deps, AGENT3, 100);
    
    // Agent with many interactions
    for i in 0..15 {
        let interaction_hash = format!("frequent_{}", i).repeat(4);
        let msg = ExecuteMsg::SubmitRating {
            rated_agent: AGENT3.to_string(),
            score: 7,
            feedback: None,
            interaction_hash,
        };
        
        let rater = if i % 2 == 0 { AGENT1 } else { AGENT2 };
        let info = mock_info(rater, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        
        // Refresh karma for raters
        give_initial_karma(&mut deps, rater, 100 + (i as u128 * 5));
    }
    
    let frequent_agent_karma = KARMA_SCORES.load(&deps.storage, AGENT3).unwrap().current_score;
    
    // Compare with agent having fewer interactions
    give_initial_karma(&mut deps, "agent4", 100);
    
    // Submit only 3 ratings for comparison agent
    for i in 0..3 {
        let interaction_hash = format!("infrequent_{}", i).repeat(4);
        let msg = ExecuteMsg::SubmitRating {
            rated_agent: "agent4".to_string(),
            score: 7,
            feedback: None,
            interaction_hash,
        };
        
        let info = mock_info(AGENT1, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        
        give_initial_karma(&mut deps, AGENT1, 100 + (i as u128 * 5));
    }
    
    let infrequent_agent_karma = KARMA_SCORES.load(&deps.storage, "agent4").unwrap().current_score;
    
    // Agent with more interactions should have higher karma
    assert!(frequent_agent_karma > infrequent_agent_karma, 
        "Agent with more interactions should have higher karma. Frequent: {}, Infrequent: {}", 
        frequent_agent_karma, infrequent_agent_karma);
}

#[test]
fn test_rating_score_validation_edge_cases() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    
    // Test boundary values
    let test_cases = vec![
        (0, true),   // Below minimum
        (1, false),  // Minimum valid
        (5, false),  // Middle value
        (10, false), // Maximum valid
        (11, true),  // Above maximum
        (255, true), // Edge case for u8
    ];
    
    for (score, should_fail) in test_cases {
        let interaction_hash = format!("score_test_{}", score).repeat(4);
        let msg = ExecuteMsg::SubmitRating {
            rated_agent: AGENT2.to_string(),
            score,
            feedback: None,
            interaction_hash,
        };
        
        let info = mock_info(AGENT1, &[]);
        let result = execute(deps.as_mut(), env.clone(), info, msg);
        
        if should_fail {
            assert!(result.is_err(), "Score {} should fail validation", score);
            if let Err(ContractError::InvalidRatingScore { score: invalid_score }) = result {
                assert_eq!(invalid_score, score);
            }
        } else {
            // For valid scores, we might get other errors (like duplicate), but not InvalidRatingScore
            if let Err(err) = result {
                match err {
                    ContractError::InvalidRatingScore { .. } => {
                        panic!("Score {} should be valid but got InvalidRatingScore", score);
                    },
                    _ => {}, // Other errors are acceptable for this test
                }
            }
        }
    }
}

#[test]
fn test_oracle_data_integration() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 100);
    
    let karma_before_oracle = KARMA_SCORES.load(&deps.storage, AGENT1).unwrap().current_score;
    
    // Submit oracle data
    let oracle_data = vec![
        OracleData {
            provider: Addr::unchecked("oracle_provider"),
            data_type: "performance".to_string(),
            data: "high_performance_metrics".to_string(),
            timestamp: env.block.time,
            signatures: vec!["sig1".to_string(), "sig2".to_string(), "sig3".to_string()],
            verified: true,
        },
        OracleData {
            provider: Addr::unchecked("oracle_provider"),
            data_type: "cross_chain".to_string(),
            data: "excellent_cross_chain_reputation".to_string(),
            timestamp: env.block.time,
            signatures: vec!["sig1".to_string(), "sig2".to_string(), "sig3".to_string()],
            verified: true,
        },
        OracleData {
            provider: Addr::unchecked("oracle_provider"),
            data_type: "sentiment".to_string(),
            data: "positive_community_sentiment".to_string(),
            timestamp: env.block.time,
            signatures: vec!["sig1".to_string(), "sig2".to_string(), "sig3".to_string()],
            verified: true,
        },
    ];
    
    let msg = ExecuteMsg::ProcessOracleData {
        agent_address: AGENT1.to_string(),
        oracle_data,
    };
    
    let info = mock_info(ADMIN, &[]);
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    let karma_after_oracle = KARMA_SCORES.load(&deps.storage, AGENT1).unwrap().current_score;
    
    // Karma should increase due to positive oracle data
    assert!(karma_after_oracle > karma_before_oracle, 
        "Karma should increase with positive oracle data. Before: {}, After: {}", 
        karma_before_oracle, karma_after_oracle);
}

#[test]
fn test_oracle_data_verification_failure() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 100);
    
    // Submit unverified oracle data
    let oracle_data = vec![
        OracleData {
            provider: Addr::unchecked("oracle_provider"),
            data_type: "performance".to_string(),
            data: "suspicious_data".to_string(),
            timestamp: env.block.time,
            signatures: vec!["invalid_sig".to_string()],
            verified: false, // Not verified
        },
    ];
    
    let msg = ExecuteMsg::ProcessOracleData {
        agent_address: AGENT1.to_string(),
        oracle_data,
    };
    
    let info = mock_info(ADMIN, &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::OracleDataVerificationFailed {} => {},
        _ => panic!("Expected OracleDataVerificationFailed error"),
    }
}

#[test]
fn test_karma_calculation_with_mixed_ratings() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 100);
    give_initial_karma(&mut deps, AGENT2, 100);
    give_initial_karma(&mut deps, AGENT3, 100);
    
    // Submit mixed ratings (good, average, poor)
    let ratings = vec![
        (AGENT1, 9, "excellent_interaction"),
        (AGENT2, 5, "average_interaction"),
        (AGENT3, 2, "poor_interaction"),
    ];
    
    for (rater, score, interaction_id) in ratings {
        let msg = ExecuteMsg::SubmitRating {
            rated_agent: "target_agent".to_string(),
            score,
            feedback: Some(format!("Rating from {}", rater)),
            interaction_hash: interaction_id.repeat(4),
        };
        
        let info = mock_info(rater, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }
    
    // Check that karma calculation handles mixed ratings appropriately
    let final_karma = KARMA_SCORES.load(&deps.storage, "target_agent").unwrap();
    
    // Should have some karma but not too high due to mixed ratings
    assert!(final_karma.current_score > Uint128::zero());
    assert!(final_karma.current_score < Uint128::from(500u128));
    
    // Check average rating calculation
    assert!(final_karma.average_rating.parse::<f64>().unwrap() > 0.0);
    assert!(final_karma.total_ratings == 3);
}

#[test]
fn test_leaderboard_updates() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    give_initial_karma(&mut deps, AGENT2, 100);
    give_initial_karma(&mut deps, AGENT3, 150);
    
    // Manually update leaderboard for test
    use crate::state::LEADERBOARD;
    LEADERBOARD.save(&mut deps.storage, 50, &AGENT1.to_string()).unwrap();
    LEADERBOARD.save(&mut deps.storage, 100, &AGENT2.to_string()).unwrap();
    LEADERBOARD.save(&mut deps.storage, 150, &AGENT3.to_string()).unwrap();
    
    // Submit rating to change karma and trigger leaderboard update
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT1.to_string(),
        score: 10,
        feedback: None,
        interaction_hash: "leaderboard_test".repeat(4),
    };
    
    let info = mock_info(AGENT2, &[]);
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    // Query leaderboard
    let msg = QueryMsg::GetLeaderboard { limit: None };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let response: LeaderboardResponse = from_json(&res).unwrap();
    
    // Should have updated leaderboard entries
    assert!(!response.leaderboard.is_empty());
    
    // Leaderboard should be sorted by karma score descending
    for i in 1..response.leaderboard.len() {
        assert!(response.leaderboard[i-1].karma_score >= response.leaderboard[i].karma_score);
    }
}

// ============================================================================
// COMPREHENSIVE EDGE CASE TESTS FOR KARMA CALCULATION ENGINE
// ============================================================================

#[test]
fn test_enhanced_rating_window_validation() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    give_initial_karma(&mut deps, AGENT2, 20);
    
    // Test with valid interaction hash (should pass)
    let valid_hash = "a1b2c3d4e5f6789012345678901234567890123456789012345678901234abcd";
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 8,
        feedback: None,
        interaction_hash: valid_hash.to_string(),
    };
    
    let info = mock_info(AGENT1, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(res.is_ok(), "Valid interaction hash should pass");
    
    // Test with invalid interaction hash format (should fail)
    let invalid_hash = "invalid_hash_format";
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 8,
        feedback: None,
        interaction_hash: invalid_hash.to_string(),
    };
    
    let info = mock_info(AGENT1, &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::InteractionNotFound { interaction_hash } => {
            assert_eq!(interaction_hash, invalid_hash);
        },
        _ => panic!("Expected InteractionNotFound error for invalid hash format"),
    }
}

#[test]
fn test_karma_calculation_with_complex_rating_patterns() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 100);
    give_initial_karma(&mut deps, AGENT2, 200);
    give_initial_karma(&mut deps, AGENT3, 300);
    
    // Create a complex rating pattern for AGENT3
    let rating_patterns = vec![
        (AGENT1, 9, "excellent_work_1"),
        (AGENT2, 8, "good_collaboration_2"),
        (AGENT1, 10, "perfect_execution_3"),
        (AGENT2, 7, "satisfactory_4"),
        (AGENT1, 9, "great_results_5"),
        (AGENT2, 8, "solid_performance_6"),
        (AGENT1, 10, "outstanding_7"),
        (AGENT2, 9, "very_good_8"),
        (AGENT1, 8, "good_work_9"),
        (AGENT2, 9, "excellent_10"),
        (AGENT1, 10, "perfect_again_11"),
        (AGENT2, 8, "consistent_12"),
    ];
    
    let initial_karma = KARMA_SCORES.load(&deps.storage, AGENT3).unwrap().current_score;
    
    for (rater, score, interaction_id) in rating_patterns {
        let interaction_hash = format!("{}", interaction_id).repeat(16); // 64 chars
        let msg = ExecuteMsg::SubmitRating {
            rated_agent: AGENT3.to_string(),
            score,
            feedback: Some(format!("Rating from {}", rater)),
            interaction_hash,
        };
        
        let info = mock_info(rater, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        
        // Refresh rater karma to maintain rating ability
        give_initial_karma(&mut deps, rater, 100 + (score as u128 * 10));
    }
    
    let final_karma = KARMA_SCORES.load(&deps.storage, AGENT3).unwrap();
    
    // Should have significant karma increase due to:
    // 1. High average rating (8.75)
    // 2. Consistency bonus (10+ high ratings)
    // 3. Interaction frequency bonus
    // 4. High-karma rater bonuses
    assert!(final_karma.current_score > initial_karma.checked_add(Uint128::from(500u128)).unwrap(),
        "Complex high-quality rating pattern should significantly increase karma. Initial: {}, Final: {}",
        initial_karma, final_karma.current_score);
    
    // Check that average rating is calculated correctly
    let avg_rating: f64 = final_karma.average_rating.parse().unwrap();
    assert!(avg_rating >= 8.5 && avg_rating <= 9.0, 
        "Average rating should be between 8.5 and 9.0, got: {}", avg_rating);
}

#[test]
fn test_karma_calculation_with_mixed_quality_ratings() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 100);
    give_initial_karma(&mut deps, AGENT2, 100);
    give_initial_karma(&mut deps, AGENT3, 100);
    
    // Mixed quality ratings: some excellent, some poor
    let mixed_ratings = vec![
        (AGENT1, 10, "perfect_1"),
        (AGENT2, 2, "very_poor_2"),
        (AGENT1, 9, "excellent_3"),
        (AGENT2, 1, "terrible_4"),
        (AGENT1, 8, "good_5"),
        (AGENT2, 3, "poor_6"),
        (AGENT1, 10, "perfect_7"),
        (AGENT2, 2, "bad_8"),
    ];
    
    let initial_karma = KARMA_SCORES.load(&deps.storage, AGENT3).unwrap().current_score;
    
    for (rater, score, interaction_id) in mixed_ratings {
        let interaction_hash = format!("{}", interaction_id).repeat(16);
        let msg = ExecuteMsg::SubmitRating {
            rated_agent: AGENT3.to_string(),
            score,
            feedback: Some(format!("Mixed rating: {}", score)),
            interaction_hash,
        };
        
        let info = mock_info(rater, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        
        give_initial_karma(&mut deps, rater, 100);
    }
    
    let final_karma = KARMA_SCORES.load(&deps.storage, AGENT3).unwrap();
    
    // With mixed ratings (average ~5.6), karma should be moderate
    // Poor ratings should apply penalties
    assert!(final_karma.current_score < initial_karma.checked_add(Uint128::from(200u128)).unwrap(),
        "Mixed quality ratings should result in moderate karma increase");
    
    let avg_rating: f64 = final_karma.average_rating.parse().unwrap();
    assert!(avg_rating >= 5.0 && avg_rating <= 6.5, 
        "Average rating should reflect mixed quality, got: {}", avg_rating);
}

#[test]
fn test_karma_earning_spending_balance_tracking() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 100);
    give_initial_karma(&mut deps, AGENT2, 50);
    
    // Check initial balance
    let initial_balance = crate::state::KARMA_BALANCE
        .may_load(&deps.storage, AGENT1)
        .unwrap()
        .unwrap_or((Uint128::zero(), Uint128::zero()));
    
    // Submit rating (should spend karma fee)
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 9,
        feedback: None,
        interaction_hash: "balance_tracking_test".repeat(4),
    };
    
    let info = mock_info(AGENT1, &[]);
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    // Check that spending was tracked
    let final_balance = crate::state::KARMA_BALANCE
        .load(&deps.storage, AGENT1)
        .unwrap();
    
    assert_eq!(final_balance.1, initial_balance.1.checked_add(Uint128::from(2u128)).unwrap(),
        "Karma spending should be tracked correctly");
    
    // Check that AGENT2 earned karma and it was tracked
    let agent2_balance = crate::state::KARMA_BALANCE
        .load(&deps.storage, AGENT2)
        .unwrap();
    
    assert!(agent2_balance.0 > Uint128::zero(),
        "Agent2 should have earned karma from positive rating");
}

#[test]
fn test_karma_calculation_with_extreme_time_decay() {
    let (mut deps, mut env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    give_initial_karma(&mut deps, AGENT2, 1000); // High initial karma
    
    // Submit initial rating
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 8,
        feedback: None,
        interaction_hash: "time_decay_extreme_test".repeat(3),
    };
    
    let info = mock_info(AGENT1, &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let karma_after_rating = KARMA_SCORES.load(&deps.storage, AGENT2).unwrap().current_score;
    
    // Advance time by 6 months (extreme decay)
    env.block.time = env.block.time.plus_seconds(180 * 24 * 60 * 60); // 180 days
    
    // Recalculate karma
    let msg = ExecuteMsg::RecalculateKarma {
        agent_address: AGENT2.to_string(),
    };
    
    let info = mock_info("anyone", &[]);
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    let karma_after_extreme_decay = KARMA_SCORES.load(&deps.storage, AGENT2).unwrap().current_score;
    
    // Should have significant decay (30% reduction)
    let expected_max_karma = (karma_after_rating.u128() as f64 * 0.7) as u128;
    assert!(karma_after_extreme_decay.u128() <= expected_max_karma + 10, // Small tolerance
        "Extreme time decay should reduce karma significantly. Before: {}, After: {}, Expected max: {}",
        karma_after_rating, karma_after_extreme_decay, expected_max_karma);
}

#[test]
fn test_contextual_modifiers_with_rater_karma_influence() {
    let (mut deps, env) = setup_contract();
    
    // Set up agents with different karma levels
    give_initial_karma(&mut deps, "high_karma_rater", 1000);
    give_initial_karma(&mut deps, "medium_karma_rater", 300);
    give_initial_karma(&mut deps, "low_karma_rater", 50);
    give_initial_karma(&mut deps, AGENT1, 100); // Agent being rated
    
    let initial_karma = KARMA_SCORES.load(&deps.storage, AGENT1).unwrap().current_score;
    
    // High karma rater gives excellent rating
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT1.to_string(),
        score: 10,
        feedback: Some("Excellent from high karma rater".to_string()),
        interaction_hash: "high_karma_rater_test".repeat(3),
    };
    
    let info = mock_info("high_karma_rater", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let karma_after_high_rater = KARMA_SCORES.load(&deps.storage, AGENT1).unwrap().current_score;
    
    // Medium karma rater gives same rating
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT1.to_string(),
        score: 10,
        feedback: Some("Excellent from medium karma rater".to_string()),
        interaction_hash: "medium_karma_rater_test".repeat(3),
    };
    
    let info = mock_info("medium_karma_rater", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let karma_after_medium_rater = KARMA_SCORES.load(&deps.storage, AGENT1).unwrap().current_score;
    
    // Low karma rater gives same rating
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT1.to_string(),
        score: 10,
        feedback: Some("Excellent from low karma rater".to_string()),
        interaction_hash: "low_karma_rater_test".repeat(3),
    };
    
    let info = mock_info("low_karma_rater", &[]);
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    let final_karma = KARMA_SCORES.load(&deps.storage, AGENT1).unwrap().current_score;
    
    // Calculate karma increases from each rater
    let high_rater_boost = karma_after_high_rater.u128() - initial_karma.u128();
    let medium_rater_boost = karma_after_medium_rater.u128() - karma_after_high_rater.u128();
    let low_rater_boost = final_karma.u128() - karma_after_medium_rater.u128();
    
    // High karma rater should provide the most boost
    assert!(high_rater_boost > medium_rater_boost,
        "High karma rater should provide more boost than medium. High: {}, Medium: {}",
        high_rater_boost, medium_rater_boost);
    
    assert!(medium_rater_boost > low_rater_boost,
        "Medium karma rater should provide more boost than low. Medium: {}, Low: {}",
        medium_rater_boost, low_rater_boost);
}

#[test]
fn test_oracle_data_integration_with_karma_calculation() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 200);
    
    let karma_before_oracle = KARMA_SCORES.load(&deps.storage, AGENT1).unwrap().current_score;
    
    // Submit comprehensive oracle data
    let oracle_data = vec![
        OracleData {
            provider: Addr::unchecked("performance_oracle"),
            data_type: "performance".to_string(),
            data: "high_performance_metrics_abcdef123456".to_string(),
            timestamp: env.block.time,
            signatures: vec!["sig1".to_string(), "sig2".to_string(), "sig3".to_string()],
            verified: true,
        },
        OracleData {
            provider: Addr::unchecked("cross_chain_oracle"),
            data_type: "cross_chain".to_string(),
            data: "excellent_cross_chain_reputation_789abc".to_string(),
            timestamp: env.block.time,
            signatures: vec!["sig1".to_string(), "sig2".to_string(), "sig3".to_string()],
            verified: true,
        },
        OracleData {
            provider: Addr::unchecked("sentiment_oracle"),
            data_type: "sentiment".to_string(),
            data: "positive_community_sentiment_def456".to_string(),
            timestamp: env.block.time,
            signatures: vec!["sig1".to_string(), "sig2".to_string(), "sig3".to_string()],
            verified: true,
        },
    ];
    
    let msg = ExecuteMsg::ProcessOracleData {
        agent_address: AGENT1.to_string(),
        oracle_data,
    };
    
    let info = mock_info(ADMIN, &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    assert_eq!(res.attributes[0].value, "process_oracle_data");
    assert_eq!(res.attributes[2].value, "3"); // 3 oracle entries processed
    
    let karma_after_oracle = KARMA_SCORES.load(&deps.storage, AGENT1).unwrap().current_score;
    
    // Oracle data should provide additional karma boost
    assert!(karma_after_oracle > karma_before_oracle,
        "Oracle data should increase karma. Before: {}, After: {}",
        karma_before_oracle, karma_after_oracle);
    
    // The boost should be significant but not excessive
    let oracle_boost = karma_after_oracle.u128() - karma_before_oracle.u128();
    assert!(oracle_boost >= 50 && oracle_boost <= 500,
        "Oracle boost should be reasonable (50-500), got: {}", oracle_boost);
}

#[test]
fn test_karma_calculation_edge_case_zero_ratings() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 100);
    
    // Try to recalculate karma for agent with no ratings
    let msg = ExecuteMsg::RecalculateKarma {
        agent_address: AGENT1.to_string(),
    };
    
    let info = mock_info("anyone", &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    assert_eq!(res.attributes[0].value, "recalculate_karma");
    
    // Should handle zero ratings gracefully
    let karma = KARMA_SCORES.load(&deps.storage, AGENT1).unwrap();
    assert_eq!(karma.total_ratings, 0);
    assert_eq!(karma.average_rating, "0.0");
}

#[test]
fn test_karma_calculation_with_single_rating() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 50);
    give_initial_karma(&mut deps, AGENT2, 100);
    
    // Submit single rating
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 7,
        feedback: Some("Single rating test".to_string()),
        interaction_hash: "single_rating_test".repeat(4),
    };
    
    let info = mock_info(AGENT1, &[]);
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    let karma = KARMA_SCORES.load(&deps.storage, AGENT2).unwrap();
    
    // Should handle single rating correctly
    assert_eq!(karma.total_ratings, 1);
    assert_eq!(karma.average_rating, "7.00");
    assert!(karma.current_score > Uint128::from(100u128)); // Should have increased
}

#[test]
fn test_karma_spending_with_insufficient_balance() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 8); // Just above minimum but below rating requirement + fee
    give_initial_karma(&mut deps, AGENT2, 20);
    
    // Try to submit rating with insufficient karma (needs 10 + 2 fee = 12, but has only 8)
    let msg = ExecuteMsg::SubmitRating {
        rated_agent: AGENT2.to_string(),
        score: 8,
        feedback: None,
        interaction_hash: "insufficient_balance_test".repeat(3),
    };
    
    let info = mock_info(AGENT1, &[]);
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    
    match err {
        ContractError::MinimumRequirementsNotMet { reason } => {
            assert!(reason.contains("Insufficient karma"));
        },
        _ => panic!("Expected MinimumRequirementsNotMet error for insufficient karma"),
    }
}

#[test]
fn test_karma_calculation_with_improvement_trend_bonus() {
    let (mut deps, env) = setup_contract();
    give_initial_karma(&mut deps, AGENT1, 100);
    give_initial_karma(&mut deps, AGENT2, 100);
    give_initial_karma(&mut deps, AGENT3, 100);
    
    // Submit ratings showing improvement trend (poor start, excellent finish)
    let improvement_ratings = vec![
        (AGENT1, 4, "poor_start_1"),
        (AGENT2, 4, "poor_start_2"),
        (AGENT1, 5, "slight_improvement_3"),
        (AGENT2, 5, "slight_improvement_4"),
        (AGENT1, 6, "getting_better_5"),
        (AGENT2, 6, "getting_better_6"),
        (AGENT1, 7, "good_progress_7"),
        (AGENT2, 7, "good_progress_8"),
        (AGENT1, 8, "very_good_9"),
        (AGENT2, 8, "very_good_10"),
        (AGENT1, 9, "excellent_11"),
        (AGENT2, 9, "excellent_12"),
    ];
    
    for (rater, score, interaction_id) in improvement_ratings {
        let interaction_hash = format!("{}", interaction_id).repeat(16);
        let msg = ExecuteMsg::SubmitRating {
            rated_agent: AGENT3.to_string(),
            score,
            feedback: Some(format!("Improvement trend: {}", score)),
            interaction_hash,
        };
        
        let info = mock_info(rater, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        
        give_initial_karma(&mut deps, rater, 100);
    }
    
    let final_karma = KARMA_SCORES.load(&deps.storage, AGENT3).unwrap();
    
    // Should receive improvement bonus for positive trend
    // Average is 6.5, but improvement trend should add bonus
    assert!(final_karma.current_score > Uint128::from(200u128),
        "Agent showing improvement trend should receive bonus karma");
    
    let avg_rating: f64 = final_karma.average_rating.parse().unwrap();
    assert!(avg_rating >= 6.0 && avg_rating <= 7.0,
        "Average rating should reflect improvement pattern, got: {}", avg_rating);
}

#[test]
fn test_karma_calculation_with_diversity_bonus() {
    let (mut deps, env) = setup_contract();
    
    // Create multiple raters for diversity test
    let raters = vec!["rater1", "rater2", "rater3", "rater4", "rater5", "rater6"];
    for rater in &raters {
        give_initial_karma(&mut deps, rater, 100);
    }
    give_initial_karma(&mut deps, AGENT1, 100);
    
    let initial_karma = KARMA_SCORES.load(&deps.storage, AGENT1).unwrap().current_score;
    
    // Submit ratings from diverse raters
    for (i, rater) in raters.iter().enumerate() {
        let interaction_hash = format!("diversity_test_{}", i).repeat(8);
        let msg = ExecuteMsg::SubmitRating {
            rated_agent: AGENT1.to_string(),
            score: 8, // Consistent good rating
            feedback: Some(format!("Rating from diverse rater {}", rater)),
            interaction_hash,
        };
        
        let info = mock_info(rater, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }
    
    let final_karma = KARMA_SCORES.load(&deps.storage, AGENT1).unwrap();
    
    // Should receive diversity bonus for having 5+ unique raters
    let karma_increase = final_karma.current_score.u128() - initial_karma.u128();
    assert!(karma_increase > 100, // Base increase plus diversity bonus
        "Agent with diverse raters should receive diversity bonus. Increase: {}", karma_increase);
}

#[test]
fn test_comprehensive_karma_calculation_integration() {
    let (mut deps, env) = setup_contract();
    
    // Set up complex scenario with multiple agents and interactions
    give_initial_karma(&mut deps, "high_karma_agent", 1000);
    give_initial_karma(&mut deps, "medium_karma_agent", 500);
    give_initial_karma(&mut deps, "low_karma_agent", 100);
    give_initial_karma(&mut deps, "target_agent", 200);
    
    let initial_karma = KARMA_SCORES.load(&deps.storage, "target_agent").unwrap().current_score;
    
    // Complex interaction pattern
    let complex_interactions = vec![
        ("high_karma_agent", 10, "perfect_collaboration"),
        ("medium_karma_agent", 9, "excellent_teamwork"),
        ("low_karma_agent", 8, "good_interaction"),
        ("high_karma_agent", 9, "great_results"),
        ("medium_karma_agent", 8, "solid_performance"),
        ("high_karma_agent", 10, "outstanding_work"),
        ("medium_karma_agent", 9, "very_impressive"),
        ("low_karma_agent", 7, "satisfactory"),
        ("high_karma_agent", 9, "consistent_excellence"),
        ("medium_karma_agent", 8, "reliable_quality"),
        ("high_karma_agent", 10, "exceptional_delivery"),
        ("low_karma_agent", 8, "improved_performance"),
    ];
    
    for (rater, score, interaction_id) in complex_interactions {
        let interaction_hash = format!("{}", interaction_id).repeat(5);
        let msg = ExecuteMsg::SubmitRating {
            rated_agent: "target_agent".to_string(),
            score,
            feedback: Some(format!("Complex scenario: {} from {}", score, rater)),
            interaction_hash,
        };
        
        let info = mock_info(rater, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }
    
    // Add oracle data for comprehensive test
    let oracle_data = vec![
        OracleData {
            provider: Addr::unchecked("comprehensive_oracle"),
            data_type: "performance".to_string(),
            data: "exceptional_performance_data_123abc".to_string(),
            timestamp: env.block.time,
            signatures: vec!["sig1".to_string(), "sig2".to_string(), "sig3".to_string()],
            verified: true,
        },
    ];
    
    let msg = ExecuteMsg::ProcessOracleData {
        agent_address: "target_agent".to_string(),
        oracle_data,
    };
    
    let info = mock_info(ADMIN, &[]);
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    let final_karma = KARMA_SCORES.load(&deps.storage, "target_agent").unwrap();
    
    // Comprehensive validation
    assert!(final_karma.current_score > initial_karma.checked_add(Uint128::from(800u128)).unwrap(),
        "Complex high-quality interaction pattern should significantly increase karma");
    
    assert_eq!(final_karma.total_ratings, 12);
    
    let avg_rating: f64 = final_karma.average_rating.parse().unwrap();
    assert!(avg_rating >= 8.5 && avg_rating <= 9.0,
        "Average rating should reflect high-quality interactions, got: {}", avg_rating);
    
    // Verify all bonuses were applied:
    // 1. High base score from excellent ratings
    // 2. Consistency bonus (10+ high ratings)
    // 3. High-karma rater bonuses
    // 4. Diversity bonus (3+ unique raters)
    // 5. Oracle data bonus
    assert!(final_karma.current_score > Uint128::from(1000u128),
        "Comprehensive scenario should result in very high karma");
}

// Test the enhanced rating window validation function directly
#[test]
fn test_rating_window_validation_edge_cases() {
    use crate::karma::{validate_rating_window_with_hash, validate_rating_window};
    use cosmwasm_std::testing::mock_dependencies;
    
    let deps = mock_dependencies();
    let current_time = cosmwasm_std::Timestamp::from_seconds(1000000);
    let window_seconds = 24 * 60 * 60; // 24 hours
    
    // Test valid hash format
    let valid_hash = "a1b2c3d4e5f6789012345678901234567890123456789012345678901234abcd";
    let result = validate_rating_window_with_hash(
        deps.as_ref(),
        valid_hash,
        &current_time,
        window_seconds,
    );
    assert!(result.is_ok(), "Valid hash should pass window validation");
    
    // Test invalid hash format
    let invalid_hash = "invalid";
    let result = validate_rating_window_with_hash(
        deps.as_ref(),
        invalid_hash,
        &current_time,
        window_seconds,
    );
    assert!(result.is_err(), "Invalid hash should fail validation");
    
    // Test direct window validation
    let old_time = current_time.minus_seconds(25 * 60 * 60); // 25 hours ago
    let result = validate_rating_window(&old_time, &current_time, window_seconds);
    assert!(result.is_err(), "Old interaction should fail window validation");
    
    let recent_time = current_time.minus_seconds(23 * 60 * 60); // 23 hours ago
    let result = validate_rating_window(&recent_time, &current_time, window_seconds);
    assert!(result.is_ok(), "Recent interaction should pass window validation");
}