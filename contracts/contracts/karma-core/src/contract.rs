use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use agent_karma_contracts::{
    messages::{karma_core::*, InstantiateMsg, MigrateMsg},
    types::{KarmaConfig, OracleData, Rating},
};

use crate::compliance::{
    apply_abuse_penalty, check_rate_limit, create_dispute, resolve_dispute, run_abuse_detection,
    DisputeResolution, ViolationType,
};
use crate::error::ContractError;
use crate::helpers::{
    apply_karma_penalty, check_karma_requirement, check_minimum_requirements,
    earn_karma_from_rating, generate_rating_id, get_agent_karma_score, spend_karma,
    validate_interaction_hash,
};
use crate::karma::{
    calculate_karma_score, update_karma_score, validate_rating_score, validate_rating_window,
    validate_rating_window_with_hash,
};
use crate::state::{
    ratings, Config, RatingTracker, StoredRating, CONFIG, KARMA_HISTORY, KARMA_SCORES, LEADERBOARD,
    ORACLE_DATA, RATING_COUNTER, RATING_TRACKERS,
};

// Contract name and version for migration
const CONTRACT_NAME: &str = "karma-core";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Default karma configuration
const DEFAULT_MIN_KARMA_FOR_RATING: u128 = 10;
const DEFAULT_MIN_KARMA_FOR_VOTING: u128 = 50;
const DEFAULT_MIN_KARMA_FOR_PROPOSAL: u128 = 100;
const DEFAULT_RATING_WINDOW: u64 = 24 * 60 * 60; // 24 hours in seconds
const DEFAULT_MAX_RATINGS_PER_INTERACTION: u8 = 1;
const DEFAULT_RATING_FEE: u128 = 2;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = match msg.admin {
        Some(admin_str) => deps.api.addr_validate(&admin_str)?,
        None => info.sender.clone(),
    };

    // Use provided config or defaults
    let karma_config = msg.config.unwrap_or_else(|| KarmaConfig {
        min_karma_for_rating: Uint128::from(DEFAULT_MIN_KARMA_FOR_RATING),
        min_karma_for_voting: Uint128::from(DEFAULT_MIN_KARMA_FOR_VOTING),
        min_karma_for_proposal: Uint128::from(DEFAULT_MIN_KARMA_FOR_PROPOSAL),
        rating_window: DEFAULT_RATING_WINDOW,
        max_ratings_per_interaction: DEFAULT_MAX_RATINGS_PER_INTERACTION,
        rating_fee: Uint128::from(DEFAULT_RATING_FEE),
    });

    let config = Config {
        admin: admin.clone(),
        agent_registry: admin.clone(),     // Will be updated later
        interaction_logger: admin.clone(), // Will be updated later
        karma_config,
    };

    CONFIG.save(deps.storage, &config)?;
    RATING_COUNTER.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin)
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SubmitRating {
            rated_agent,
            score,
            feedback,
            interaction_hash,
        } => execute_submit_rating(
            deps,
            env,
            info,
            rated_agent,
            score,
            feedback,
            interaction_hash,
        ),
        ExecuteMsg::RecalculateKarma { agent_address } => {
            execute_recalculate_karma(deps, env, info, agent_address)
        }
        ExecuteMsg::UpdateConfig { config } => execute_update_config(deps, env, info, config),
        ExecuteMsg::ProcessOracleData {
            agent_address,
            oracle_data,
        } => execute_process_oracle_data(deps, env, info, agent_address, oracle_data),
        ExecuteMsg::RunAbuseDetection { agent_address } => {
            execute_run_abuse_detection(deps, env, info, agent_address)
        }
        ExecuteMsg::ApplyCompliancePenalty {
            agent_address,
            violation_type,
            severity,
            evidence,
        } => execute_apply_compliance_penalty(
            deps,
            env,
            info,
            agent_address,
            violation_type,
            severity,
            evidence,
        ),
        ExecuteMsg::CreateDispute {
            violation_id,
            stake_amount,
            evidence,
        } => execute_create_dispute(deps, env, info, violation_id, stake_amount, evidence),
        ExecuteMsg::ResolveDispute {
            case_id,
            resolution,
        } => execute_resolve_dispute(deps, env, info, case_id, resolution),
    }
}

pub fn execute_submit_rating(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    rated_agent: String,
    score: u8,
    feedback: Option<String>,
    interaction_hash: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let rater = info.sender.clone();
    let rated_agent_addr = deps.api.addr_validate(&rated_agent)?;

    // Validate inputs
    validate_rating_score(score)?;
    validate_interaction_hash(&interaction_hash)?;

    // Check if rater is trying to rate themselves
    if rater == rated_agent_addr {
        return Err(ContractError::CannotRateSelf {});
    }

    // Check minimum karma requirements for rating
    check_minimum_requirements(deps.as_ref(), &rater, "rating")?;

    // Check rate limiting for rating submissions
    if !check_rate_limit(deps.branch(), &env, &rater, "rating")? {
        return Err(ContractError::RateLimitExceeded {
            action: "rating".to_string(),
        });
    }

    // Check for duplicate ratings
    let tracker_key = (interaction_hash.as_str(), rater.as_str());
    if RATING_TRACKERS.has(deps.storage, tracker_key) {
        return Err(ContractError::RatingAlreadySubmitted {
            interaction_hash: interaction_hash.clone(),
        });
    }

    // Validate rating window (24 hours) with enhanced validation
    validate_rating_window_with_hash(
        deps.as_ref(),
        &interaction_hash,
        &env.block.time,
        config.karma_config.rating_window,
    )?;

    // Charge rating fee
    spend_karma(deps.branch(), &rater, config.karma_config.rating_fee)?;

    // Generate unique rating ID
    let rating_counter = RATING_COUNTER.load(deps.storage)?;
    let new_counter = rating_counter + 1;
    RATING_COUNTER.save(deps.storage, &new_counter)?;

    let rating_id = generate_rating_id(
        &rater,
        &rated_agent_addr,
        &interaction_hash,
        env.block.time.seconds(),
    );

    // Create rating
    let rating = Rating {
        id: rating_id.clone(),
        rater_address: rater.clone(),
        rated_address: rated_agent_addr.clone(),
        score,
        feedback,
        interaction_hash: interaction_hash.clone(),
        timestamp: env.block.time,
        block_height: env.block.height,
    };

    let stored_rating = StoredRating {
        rating,
        processed: false,
        fee_paid: config.karma_config.rating_fee,
    };

    // Save rating
    ratings().save(deps.storage, &rating_id, &stored_rating)?;

    // Create duplicate prevention tracker
    let tracker = RatingTracker {
        interaction_hash: interaction_hash.clone(),
        rater: rater.clone(),
        submitted_at: env.block.time,
    };
    RATING_TRACKERS.save(deps.storage, tracker_key, &tracker)?;

    // Apply karma earning/penalty based on rating score
    let rater_karma = get_agent_karma_score(deps.as_ref(), &rater)?;
    let karma_earned =
        earn_karma_from_rating(deps.branch(), &rated_agent_addr, score, rater_karma)?;
    let karma_penalty = apply_karma_penalty(deps.branch(), &rated_agent_addr, score)?;

    // Recalculate karma for the rated agent
    let karma_calculation = calculate_karma_score(deps.as_ref(), &env, &rated_agent_addr)?;
    update_karma_score(deps.branch(), &env, &rated_agent_addr, &karma_calculation)?;

    // Update leaderboard
    update_leaderboard(
        deps.branch(),
        &rated_agent_addr,
        karma_calculation.current_score,
    )?;

    Ok(Response::new()
        .add_attribute("method", "submit_rating")
        .add_attribute("rater", rater)
        .add_attribute("rated_agent", rated_agent)
        .add_attribute("score", score.to_string())
        .add_attribute("interaction_hash", interaction_hash)
        .add_attribute("rating_id", rating_id)
        .add_attribute("new_karma", karma_calculation.current_score)
        .add_attribute("fee_paid", config.karma_config.rating_fee))
}

pub fn execute_recalculate_karma(
    mut deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    agent_address: String,
) -> Result<Response, ContractError> {
    let agent_addr = deps.api.addr_validate(&agent_address)?;

    // Calculate new karma score
    let karma_calculation = calculate_karma_score(deps.as_ref(), &env, &agent_addr)?;

    // Update karma score
    update_karma_score(deps.branch(), &env, &agent_addr, &karma_calculation)?;

    // Update leaderboard
    update_leaderboard(deps.branch(), &agent_addr, karma_calculation.current_score)?;

    Ok(Response::new()
        .add_attribute("method", "recalculate_karma")
        .add_attribute("agent_address", agent_address)
        .add_attribute("new_karma", karma_calculation.current_score)
        .add_attribute("previous_karma", karma_calculation.previous_score))
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_config: KarmaConfig,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // Only admin can update config
    if info.sender != config.admin {
        return Err(ContractError::AdminRequired {});
    }

    // Validate new configuration
    if new_config.rating_window == 0 {
        return Err(ContractError::InvalidKarmaConfig {
            reason: "Rating window cannot be zero".to_string(),
        });
    }

    if new_config.max_ratings_per_interaction == 0 {
        return Err(ContractError::InvalidKarmaConfig {
            reason: "Max ratings per interaction cannot be zero".to_string(),
        });
    }

    config.karma_config = new_config.clone();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "update_config")
        .add_attribute("admin", info.sender)
        .add_attribute("min_karma_for_rating", new_config.min_karma_for_rating)
        .add_attribute("rating_window", new_config.rating_window.to_string())
        .add_attribute("rating_fee", new_config.rating_fee))
}

pub fn execute_process_oracle_data(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    agent_address: String,
    oracle_data: Vec<OracleData>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only admin or verified oracle providers can submit data
    if info.sender != config.admin {
        return Err(ContractError::AdminRequired {});
    }

    let agent_addr = deps.api.addr_validate(&agent_address)?;

    // Process each oracle data entry
    for data in oracle_data.iter() {
        // Verify oracle data (simplified - in real implementation would check signatures)
        if !data.verified {
            return Err(ContractError::OracleDataVerificationFailed {});
        }

        // Store oracle data hash for karma calculation
        let data_key = (agent_address.as_str(), data.data_type.as_str());
        let data_hash = format!("{}:{}", data.timestamp.seconds(), data.data.len());
        ORACLE_DATA.save(deps.storage, data_key, &data_hash)?;
    }

    // Recalculate karma with new oracle data
    let karma_calculation = calculate_karma_score(deps.as_ref(), &env, &agent_addr)?;
    update_karma_score(deps.branch(), &env, &agent_addr, &karma_calculation)?;

    // Update leaderboard
    update_leaderboard(deps.branch(), &agent_addr, karma_calculation.current_score)?;

    Ok(Response::new()
        .add_attribute("method", "process_oracle_data")
        .add_attribute("agent_address", agent_address)
        .add_attribute("oracle_entries", oracle_data.len().to_string())
        .add_attribute("new_karma", karma_calculation.current_score))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetKarmaScore { agent_address } => {
            to_json_binary(&query_get_karma_score(deps, agent_address)?)
        }
        QueryMsg::GetKarmaCalculation { agent_address } => {
            to_json_binary(&query_get_karma_calculation(deps, agent_address)?)
        }
        QueryMsg::GetKarmaHistory {
            agent_address,
            start_after,
            limit,
        } => to_json_binary(&query_get_karma_history(
            deps,
            agent_address,
            start_after,
            limit,
        )?),
        QueryMsg::GetAgentRatings {
            agent_address,
            start_after,
            limit,
        } => to_json_binary(&query_get_agent_ratings(
            deps,
            agent_address,
            start_after,
            limit,
        )?),
        QueryMsg::GetLeaderboard { limit } => to_json_binary(&query_get_leaderboard(deps, limit)?),
        QueryMsg::GetConfig {} => to_json_binary(&query_get_config(deps)?),
        QueryMsg::GetComplianceViolations {
            agent_address,
            start_after,
            limit,
        } => to_json_binary(&query_get_compliance_violations(
            deps,
            agent_address,
            start_after,
            limit,
        )?),
        QueryMsg::GetDisputeCases {
            status,
            start_after,
            limit,
        } => to_json_binary(&query_get_dispute_cases(deps, status, start_after, limit)?),
        QueryMsg::GetAbuseDetectionResults { agent_address } => {
            to_json_binary(&query_get_abuse_detection_results(deps, agent_address)?)
        }
        QueryMsg::GetRateLimitStatus {
            agent_address,
            action_type,
        } => to_json_binary(&query_get_rate_limit_status(
            deps,
            agent_address,
            action_type,
        )?),
    }
}

pub fn query_get_karma_score(deps: Deps, agent_address: String) -> StdResult<KarmaScoreResponse> {
    let karma = KARMA_SCORES
        .may_load(deps.storage, &agent_address)?
        .unwrap_or_default();

    Ok(KarmaScoreResponse {
        score: karma.current_score,
        last_updated: karma.last_updated,
    })
}

pub fn query_get_karma_calculation(
    deps: Deps,
    agent_address: String,
) -> StdResult<KarmaCalculationResponse> {
    let agent_addr = deps.api.addr_validate(&agent_address)?;

    // Get the most recent calculation from history
    let history: StdResult<Vec<_>> = KARMA_HISTORY
        .prefix(&agent_address)
        .range(deps.storage, None, None, Order::Descending)
        .take(1)
        .collect();

    let calculation = match history?.first() {
        Some((_, calc)) => calc.clone(),
        None => {
            // Return default calculation if no history exists
            agent_karma_contracts::types::KarmaCalculation {
                agent_address: agent_addr,
                current_score: Uint128::zero(),
                previous_score: Uint128::zero(),
                factors: agent_karma_contracts::types::KarmaFactors {
                    average_rating: "0.0".to_string(),
                    rating_count: 0,
                    interaction_frequency: Uint128::zero(),
                    time_decay: "1.0".to_string(),
                    external_factors: Some(Uint128::zero()),
                },
                last_updated: cosmwasm_std::Timestamp::from_seconds(0),
                calculation_hash: "".to_string(),
            }
        }
    };

    Ok(KarmaCalculationResponse { calculation })
}

pub fn query_get_karma_history(
    deps: Deps,
    agent_address: String,
    start_after: Option<cosmwasm_std::Timestamp>,
    limit: Option<u32>,
) -> StdResult<KarmaHistoryResponse> {
    let limit = limit.unwrap_or(50).min(100) as usize;

    let start_bound = start_after.map(|ts| Bound::exclusive(ts.seconds()));

    let history: StdResult<Vec<_>> = KARMA_HISTORY
        .prefix(&agent_address)
        .range(deps.storage, start_bound, None, Order::Descending)
        .take(limit)
        .map(|item| item.map(|(_, calc)| calc))
        .collect();

    Ok(KarmaHistoryResponse { history: history? })
}

pub fn query_get_agent_ratings(
    deps: Deps,
    agent_address: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<RatingsResponse> {
    let limit = limit.unwrap_or(50).min(100) as usize;

    let start_bound = start_after.as_ref().map(|id| Bound::exclusive(id.as_str()));

    let ratings_result: StdResult<Vec<_>> = ratings()
        .idx
        .rated_agent
        .prefix(agent_address)
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, stored_rating)| stored_rating.rating))
        .collect();

    Ok(RatingsResponse {
        ratings: ratings_result?,
    })
}

pub fn query_get_leaderboard(deps: Deps, limit: Option<u32>) -> StdResult<LeaderboardResponse> {
    let limit = limit.unwrap_or(20).min(100) as usize;

    let leaderboard_entries: StdResult<Vec<_>> = LEADERBOARD
        .range(deps.storage, None, None, Order::Descending)
        .take(limit)
        .map(|item| {
            let (karma_score, agent_address) = item?;
            let karma = KARMA_SCORES.load(deps.storage, &agent_address)?;

            Ok(LeaderboardEntry {
                agent_address: Addr::unchecked(&agent_address),
                karma_score: Uint128::from(karma_score),
                agent_name: "Unknown".to_string(), // Would query agent registry in real implementation
                framework: "Unknown".to_string(), // Would query agent registry in real implementation
            })
        })
        .collect();

    Ok(LeaderboardResponse {
        leaderboard: leaderboard_entries?,
    })
}

pub fn query_get_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        config: config.karma_config,
    })
}

// Compliance query functions

pub fn query_get_compliance_violations(
    deps: Deps,
    agent_address: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ComplianceViolationsResponse> {
    let limit = limit.unwrap_or(50).min(100) as usize;

    let start_bound = start_after.as_ref().map(|id| Bound::exclusive(id.as_str()));

    let violations: StdResult<Vec<_>> = crate::state::COMPLIANCE_VIOLATIONS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .filter_map(|item| match item {
            Ok((violation_id, violation)) => {
                if violation.agent_address.to_string() == agent_address {
                    Some(Ok(
                        agent_karma_contracts::messages::karma_core::ComplianceViolation {
                            agent_address: violation.agent_address,
                            violation_type: format!("{:?}", violation.violation_type),
                            severity: violation.severity,
                            timestamp: violation.timestamp,
                            evidence: violation.evidence,
                            penalty_applied: violation.penalty_applied,
                            disputed: violation.disputed,
                        },
                    ))
                } else {
                    None
                }
            }
            Err(e) => Some(Err(e)),
        })
        .take(limit)
        .collect();

    Ok(ComplianceViolationsResponse {
        violations: violations?,
    })
}

pub fn query_get_dispute_cases(
    deps: Deps,
    status: Option<String>,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<DisputeCasesResponse> {
    let limit = limit.unwrap_or(50).min(100) as usize;

    let start_bound = start_after.as_ref().map(|id| Bound::exclusive(id.as_str()));

    let cases: StdResult<Vec<_>> = crate::state::DISPUTE_CASES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .filter_map(|item| match item {
            Ok((case_id, case)) => {
                let case_status = format!("{:?}", case.status);
                if status.is_none() || status.as_ref() == Some(&case_status) {
                    Some(Ok(
                        agent_karma_contracts::messages::karma_core::DisputeCase {
                            case_id: case.case_id,
                            violation_id: case.violation_id,
                            challenger: case.challenger,
                            stake_amount: case.stake_amount,
                            evidence: case.evidence,
                            status: case_status,
                            created_at: case.created_at,
                            resolved_at: case.resolved_at,
                            resolution: case.resolution.map(|r| format!("{:?}", r)),
                        },
                    ))
                } else {
                    None
                }
            }
            Err(e) => Some(Err(e)),
        })
        .take(limit)
        .collect();

    Ok(DisputeCasesResponse { cases: cases? })
}

pub fn query_get_abuse_detection_results(
    deps: Deps,
    agent_address: String,
) -> StdResult<AbuseDetectionResponse> {
    let agent_addr = deps.api.addr_validate(&agent_address)?;

    // Create a mock environment for detection (in real implementation, would use current env)
    let mock_env = cosmwasm_std::testing::mock_env();

    // Run abuse detection
    let detection_results =
        crate::compliance::run_abuse_detection(deps, &mock_env, &agent_addr).unwrap_or_default();

    let results: Vec<_> = detection_results
        .into_iter()
        .map(|result| AbuseDetectionResult {
            is_suspicious: result.is_suspicious,
            violation_type: result.violation_type.map(|v| format!("{:?}", v)),
            confidence_score: format!("{:.2}", result.confidence_score),
            evidence: result.evidence,
            recommended_penalty: result.recommended_penalty,
        })
        .collect();

    Ok(AbuseDetectionResponse { results })
}

pub fn query_get_rate_limit_status(
    deps: Deps,
    agent_address: String,
    action_type: String,
) -> StdResult<RateLimitStatusResponse> {
    let agent_addr = deps.api.addr_validate(&agent_address)?;

    // Get rate limit tracker
    let tracker =
        crate::state::RATE_LIMIT_TRACKERS.may_load(deps.storage, agent_address.as_str())?;

    let current_time = cosmwasm_std::testing::mock_env().block.time;
    let window_duration = crate::compliance::RATING_PATTERN_WINDOW;

    let (current_count, limit, window_start, remaining_actions) = if let Some(tracker) = tracker {
        // Check if window has expired
        let window_expired =
            current_time.seconds() - tracker.window_start.seconds() > window_duration;
        let count = if window_expired { 0 } else { tracker.count };

        // Get karma-based limit
        let karma_score = crate::state::KARMA_SCORES
            .may_load(deps.storage, &agent_address)?
            .unwrap_or_default();

        let base_limit = match action_type.as_str() {
            "rating" => crate::compliance::SPAM_RATING_THRESHOLD,
            "interaction" => crate::compliance::BOT_BEHAVIOR_THRESHOLD,
            _ => 20,
        };

        let karma_multiplier = if karma_score.current_score.u128() > 1000 {
            2.0
        } else if karma_score.current_score.u128() > 500 {
            1.5
        } else if karma_score.current_score.u128() > 100 {
            1.2
        } else {
            1.0
        };

        let effective_limit = (base_limit as f64 * karma_multiplier) as u32;
        let remaining = effective_limit.saturating_sub(count);

        (count, effective_limit, tracker.window_start, remaining)
    } else {
        // No tracker exists, use defaults
        let base_limit = match action_type.as_str() {
            "rating" => crate::compliance::SPAM_RATING_THRESHOLD,
            "interaction" => crate::compliance::BOT_BEHAVIOR_THRESHOLD,
            _ => 20,
        };
        (0, base_limit, current_time, base_limit)
    };

    let status = RateLimitStatus {
        agent_address: agent_addr,
        action_type,
        current_count,
        limit,
        window_start,
        window_end: window_start.plus_seconds(window_duration),
        remaining_actions,
    };

    Ok(RateLimitStatusResponse { status })
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

// Compliance execute functions

pub fn execute_run_abuse_detection(
    mut deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    agent_address: String,
) -> Result<Response, ContractError> {
    let agent_addr = deps.api.addr_validate(&agent_address)?;

    // Run comprehensive abuse detection
    let detection_results = run_abuse_detection(deps.as_ref(), &env, &agent_addr).map_err(|e| {
        ContractError::ComplianceViolation {
            reason: format!("Abuse detection failed: {}", e),
        }
    })?;

    let mut violations_detected = 0;
    let mut total_penalty = Uint128::zero();

    // Process detection results and apply penalties if needed
    for result in &detection_results {
        if result.is_suspicious {
            violations_detected += 1;
            total_penalty += result.recommended_penalty;

            // Create compliance violation record
            let violation_id = format!(
                "{}:{}:{}",
                agent_address,
                result
                    .violation_type
                    .as_ref()
                    .map(|v| format!("{:?}", v))
                    .unwrap_or("unknown".to_string()),
                env.block.time.seconds()
            );

            let violation = crate::compliance::ComplianceViolation {
                agent_address: agent_addr.clone(),
                violation_type: result
                    .violation_type
                    .clone()
                    .unwrap_or(ViolationType::SuspiciousPattern),
                severity: (result.confidence_score * 10.0) as u8,
                timestamp: env.block.time,
                evidence: result.evidence.join("; "),
                penalty_applied: result.recommended_penalty,
                disputed: false,
            };

            // Save violation record
            crate::state::COMPLIANCE_VIOLATIONS.save(deps.storage, &violation_id, &violation)?;

            // Apply penalty
            apply_abuse_penalty(deps.branch(), &env, &agent_addr, &violation)?;
        }
    }

    Ok(Response::new()
        .add_attribute("method", "run_abuse_detection")
        .add_attribute("agent_address", agent_address)
        .add_attribute("violations_detected", violations_detected.to_string())
        .add_attribute("total_penalty", total_penalty))
}

pub fn execute_apply_compliance_penalty(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    agent_address: String,
    violation_type: String,
    severity: u8,
    evidence: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only admin can manually apply penalties
    if info.sender != config.admin {
        return Err(ContractError::AdminRequired {});
    }

    let agent_addr = deps.api.addr_validate(&agent_address)?;

    // Validate severity (1-10 scale)
    if severity == 0 || severity > 10 {
        return Err(ContractError::ComplianceViolation {
            reason: "Severity must be between 1 and 10".to_string(),
        });
    }

    // Calculate penalty based on severity
    let base_penalty = Uint128::from(crate::compliance::KARMA_PENALTY_MULTIPLIER);
    let penalty_amount = base_penalty * Uint128::from(severity as u128);

    // Parse violation type
    let violation_enum = match violation_type.as_str() {
        "spam_rating" => ViolationType::SpamRating,
        "rating_manipulation" => ViolationType::RatingManipulation,
        "bot_behavior" => ViolationType::BotBehavior,
        "rate_limit_exceeded" => ViolationType::RateLimitExceeded,
        _ => ViolationType::SuspiciousPattern,
    };

    // Create violation record
    let violation_id = format!(
        "manual:{}:{}:{}",
        agent_address,
        violation_type,
        env.block.time.seconds()
    );
    let violation = crate::compliance::ComplianceViolation {
        agent_address: agent_addr.clone(),
        violation_type: violation_enum,
        severity,
        timestamp: env.block.time,
        evidence,
        penalty_applied: penalty_amount,
        disputed: false,
    };

    // Save violation record
    crate::state::COMPLIANCE_VIOLATIONS.save(deps.storage, &violation_id, &violation)?;

    // Apply penalty
    apply_abuse_penalty(deps.branch(), &env, &agent_addr, &violation)?;

    Ok(Response::new()
        .add_attribute("method", "apply_compliance_penalty")
        .add_attribute("agent_address", agent_address)
        .add_attribute("violation_type", violation_type)
        .add_attribute("severity", severity.to_string())
        .add_attribute("penalty_applied", penalty_amount))
}

pub fn execute_create_dispute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    violation_id: String,
    stake_amount: Uint128,
    evidence: String,
) -> Result<Response, ContractError> {
    let case_id = create_dispute(
        deps,
        &env,
        &info.sender,
        violation_id.clone(),
        stake_amount,
        evidence,
    )?;

    Ok(Response::new()
        .add_attribute("method", "create_dispute")
        .add_attribute("challenger", info.sender)
        .add_attribute("violation_id", violation_id)
        .add_attribute("case_id", case_id)
        .add_attribute("stake_amount", stake_amount))
}

pub fn execute_resolve_dispute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    case_id: String,
    resolution: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only admin can resolve disputes
    if info.sender != config.admin {
        return Err(ContractError::AdminRequired {});
    }

    let dispute_resolution = match resolution.as_str() {
        "confirmed" => DisputeResolution::ViolationConfirmed,
        "overturned" => DisputeResolution::ViolationOverturned,
        "partial" => DisputeResolution::PartialOverturned,
        _ => {
            return Err(ContractError::ComplianceViolation {
                reason: "Invalid resolution type".to_string(),
            })
        }
    };

    resolve_dispute(deps, &env, &case_id, dispute_resolution)?;

    Ok(Response::new()
        .add_attribute("method", "resolve_dispute")
        .add_attribute("case_id", case_id)
        .add_attribute("resolution", resolution)
        .add_attribute("resolver", info.sender))
}

/// Update leaderboard with new karma score
pub fn update_leaderboard(
    deps: DepsMut,
    agent_address: &Addr,
    new_karma_score: Uint128,
) -> Result<(), ContractError> {
    let agent_str = agent_address.to_string();

    // Remove old entry if it exists
    let old_karma = KARMA_SCORES
        .may_load(deps.storage, &agent_str)?
        .map(|k| k.current_score.u128())
        .unwrap_or(0);

    if old_karma > 0 {
        LEADERBOARD.remove(deps.storage, old_karma);
    }

    // Add new entry
    if new_karma_score.u128() > 0 {
        LEADERBOARD.save(deps.storage, new_karma_score.u128(), &agent_str)?;
    }

    Ok(())
}
