//! Compliance and abuse detection module for Agent-Karma system
//! 
//! This module implements automated detection of spam ratings, malicious behavior patterns,
//! and provides mechanisms for karma penalties and dispute resolution.

use cosmwasm_std::{Addr, Deps, DepsMut, Env, Timestamp, Uint128, StdResult, Order};
use cw_storage_plus::Bound;
use std::collections::HashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use agent_karma_contracts::types::Rating;
use crate::error::ContractError;
use crate::state::{
    ratings, KARMA_SCORES, DISPUTE_CASES,
    RATE_LIMIT_TRACKERS, KARMA_PENALTIES,
};

/// Abuse detection patterns and thresholds
pub const SPAM_RATING_THRESHOLD: u32 = 10; // Max ratings per hour
pub const RATING_PATTERN_WINDOW: u64 = 3600; // 1 hour in seconds
pub const MIN_RATING_VARIANCE: f64 = 0.5; // Minimum variance in rating scores
pub const SUSPICIOUS_INTERACTION_RATIO: f64 = 0.8; // Max ratio of interactions with same agents
pub const BOT_BEHAVIOR_THRESHOLD: u32 = 50; // Max actions per hour for bot detection
pub const KARMA_PENALTY_MULTIPLIER: u128 = 10; // Penalty multiplier for violations

/// Types of compliance violations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ViolationType {
    SpamRating,
    RatingManipulation,
    BotBehavior,
    SuspiciousPattern,
    RateLimitExceeded,
}

/// Compliance violation record
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ComplianceViolation {
    pub agent_address: Addr,
    pub violation_type: ViolationType,
    pub severity: u8, // 1-10 scale
    pub timestamp: Timestamp,
    pub evidence: String,
    pub penalty_applied: Uint128,
    pub disputed: bool,
}

/// Dispute case for false positive detections
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DisputeCase {
    pub case_id: String,
    pub violation_id: String,
    pub challenger: Addr,
    pub stake_amount: Uint128,
    pub evidence: String,
    pub status: DisputeStatus,
    pub created_at: Timestamp,
    pub resolved_at: Option<Timestamp>,
    pub resolution: Option<DisputeResolution>,
}

/// Status of a dispute case
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum DisputeStatus {
    Pending,
    UnderReview,
    Resolved,
    Rejected,
}

/// Resolution of a dispute case
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum DisputeResolution {
    ViolationConfirmed,
    ViolationOverturned,
    PartialOverturned,
}

/// Rate limiting tracker
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RateLimitTracker {
    pub agent_address: Addr,
    pub action_type: String,
    pub count: u32,
    pub window_start: Timestamp,
    pub last_action: Timestamp,
}

/// Abuse pattern detection result
#[derive(Clone, Debug, PartialEq)]
pub struct AbuseDetectionResult {
    pub is_suspicious: bool,
    pub violation_type: Option<ViolationType>,
    pub confidence_score: f64, // 0.0 to 1.0
    pub evidence: Vec<String>,
    pub recommended_penalty: Uint128,
}

/// Detect spam ratings based on frequency and patterns
pub fn detect_spam_ratings(
    deps: Deps,
    env: &Env,
    agent_address: &Addr,
) -> StdResult<AbuseDetectionResult> {
    let current_time = env.block.time;
    let window_start = current_time.minus_seconds(RATING_PATTERN_WINDOW);
    
    // Get all ratings and filter by time and rater
    let recent_ratings: StdResult<Vec<_>> = ratings()
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|item| {
            match item {
                Ok((_, stored_rating)) => {
                    stored_rating.rating.rater_address == *agent_address &&
                    stored_rating.rating.timestamp >= window_start
                },
                Err(_) => true,
            }
        })
        .collect();
    
    let ratings_list = recent_ratings?;
    let rating_count = ratings_list.len() as u32;
    
    let mut evidence = Vec::new();
    let mut is_suspicious = false;
    let mut confidence_score = 0.0;
    
    // Check rating frequency
    if rating_count > SPAM_RATING_THRESHOLD {
        is_suspicious = true;
        confidence_score += 0.4;
        evidence.push(format!("High rating frequency: {} ratings in 1 hour", rating_count));
    }
    
    // Check rating score variance (detect bot-like behavior)
    if rating_count > 5 {
        let scores: Vec<u8> = ratings_list
            .iter()
            .map(|(_, stored_rating)| stored_rating.rating.score)
            .collect();
        
        let variance = calculate_variance(&scores);
        if variance < MIN_RATING_VARIANCE {
            is_suspicious = true;
            confidence_score += 0.3;
            evidence.push(format!("Low rating variance: {:.2}", variance));
        }
    }
    
    // Check for rating the same agents repeatedly
    let mut target_counts: HashMap<String, u32> = HashMap::new();
    for (_, stored_rating) in &ratings_list {
        let target = stored_rating.rating.rated_address.to_string();
        *target_counts.entry(target).or_insert(0) += 1;
    }
    
    let max_target_count = target_counts.values().max().unwrap_or(&0);
    let suspicious_ratio = (*max_target_count as f64) / (rating_count as f64);
    
    if suspicious_ratio > SUSPICIOUS_INTERACTION_RATIO && rating_count > 3 {
        is_suspicious = true;
        confidence_score += 0.3;
        evidence.push(format!("Suspicious interaction pattern: {:.2} ratio", suspicious_ratio));
    }
    
    let recommended_penalty = if is_suspicious {
        Uint128::from(KARMA_PENALTY_MULTIPLIER * (confidence_score * 100.0) as u128)
    } else {
        Uint128::zero()
    };
    
    Ok(AbuseDetectionResult {
        is_suspicious,
        violation_type: if is_suspicious { Some(ViolationType::SpamRating) } else { None },
        confidence_score,
        evidence,
        recommended_penalty,
    })
}

/// Detect bot behavior patterns
pub fn detect_bot_behavior(
    deps: Deps,
    env: &Env,
    agent_address: &Addr,
) -> StdResult<AbuseDetectionResult> {
    let current_time = env.block.time;
    let _window_start = current_time.minus_seconds(RATING_PATTERN_WINDOW);
    
    // Check rate limiting tracker for this agent
    let rate_tracker = RATE_LIMIT_TRACKERS.may_load(deps.storage, agent_address.as_str())?;
    
    let mut evidence = Vec::new();
    let mut is_suspicious = false;
    let mut confidence_score = 0.0;
    
    if let Some(tracker) = rate_tracker {
        // Check if actions exceed bot threshold
        if tracker.count > BOT_BEHAVIOR_THRESHOLD {
            is_suspicious = true;
            confidence_score += 0.5;
            evidence.push(format!("High action frequency: {} actions in 1 hour", tracker.count));
        }
        
        // Check for perfectly regular timing (bot-like)
        let time_since_start = current_time.seconds() - tracker.window_start.seconds();
        if time_since_start > 0 {
            let average_interval = time_since_start / (tracker.count as u64);
            if average_interval < 10 && tracker.count > 20 {
                is_suspicious = true;
                confidence_score += 0.3;
                evidence.push(format!("Regular timing pattern: {}s average interval", average_interval));
            }
        }
    }
    
    let recommended_penalty = if is_suspicious {
        Uint128::from(KARMA_PENALTY_MULTIPLIER * 2 * (confidence_score * 100.0) as u128)
    } else {
        Uint128::zero()
    };
    
    Ok(AbuseDetectionResult {
        is_suspicious,
        violation_type: if is_suspicious { Some(ViolationType::BotBehavior) } else { None },
        confidence_score,
        evidence,
        recommended_penalty,
    })
}

/// Detect rating manipulation patterns
pub fn detect_rating_manipulation(
    deps: Deps,
    env: &Env,
    agent_address: &Addr,
) -> StdResult<AbuseDetectionResult> {
    let current_time = env.block.time;
    let window_start = current_time.minus_seconds(RATING_PATTERN_WINDOW * 24); // 24 hour window
    
    // Get ratings given by this agent
    let given_ratings: StdResult<Vec<_>> = ratings()
        .idx
        .rater
        .prefix(agent_address.to_string())
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|item| {
            match item {
                Ok((_, stored_rating)) => stored_rating.rating.timestamp >= window_start,
                Err(_) => true,
            }
        })
        .collect();
    
    // Get ratings received by this agent
    let received_ratings: StdResult<Vec<_>> = ratings()
        .idx
        .rated_agent
        .prefix(agent_address.to_string())
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|item| {
            match item {
                Ok((_, stored_rating)) => stored_rating.rating.timestamp >= window_start,
                Err(_) => true,
            }
        })
        .collect();
    
    let given_list = given_ratings?;
    let received_list = received_ratings?;
    
    let mut evidence = Vec::new();
    let mut is_suspicious = false;
    let mut confidence_score = 0.0;
    
    // Check for reciprocal rating patterns (quid pro quo)
    let mut reciprocal_count = 0;
    for (_, given_rating) in &given_list {
        let target = &given_rating.rating.rated_address;
        
        // Check if target also rated this agent recently
        for (_, received_rating) in &received_list {
            if received_rating.rating.rater_address == *target {
                let time_diff = if given_rating.rating.timestamp > received_rating.rating.timestamp {
                    given_rating.rating.timestamp.seconds() - received_rating.rating.timestamp.seconds()
                } else {
                    received_rating.rating.timestamp.seconds() - given_rating.rating.timestamp.seconds()
                };
                
                // If ratings are within 1 hour of each other, it's suspicious
                if time_diff < 3600 {
                    reciprocal_count += 1;
                }
            }
        }
    }
    
    if reciprocal_count > 3 && given_list.len() > 5 {
        let reciprocal_ratio = (reciprocal_count as f64) / (given_list.len() as f64);
        if reciprocal_ratio > 0.3 {
            is_suspicious = true;
            confidence_score += 0.4;
            evidence.push(format!("Reciprocal rating pattern: {} reciprocal ratings", reciprocal_count));
        }
    }
    
    // Check for coordinated rating attacks (multiple low ratings in short time)
    let low_ratings_given = given_list
        .iter()
        .filter(|(_, rating)| rating.rating.score <= 3)
        .count();
    
    if low_ratings_given > 5 && given_list.len() > 0 {
        let low_rating_ratio = (low_ratings_given as f64) / (given_list.len() as f64);
        if low_rating_ratio > 0.7 {
            is_suspicious = true;
            confidence_score += 0.3;
            evidence.push(format!("Coordinated low rating pattern: {:.2} ratio", low_rating_ratio));
        }
    }
    
    let recommended_penalty = if is_suspicious {
        Uint128::from(KARMA_PENALTY_MULTIPLIER * 3 * (confidence_score * 100.0) as u128)
    } else {
        Uint128::zero()
    };
    
    Ok(AbuseDetectionResult {
        is_suspicious,
        violation_type: if is_suspicious { Some(ViolationType::RatingManipulation) } else { None },
        confidence_score,
        evidence,
        recommended_penalty,
    })
}

/// Apply karma penalty for detected abuse
pub fn apply_abuse_penalty(
    deps: DepsMut,
    env: &Env,
    agent_address: &Addr,
    violation: &ComplianceViolation,
) -> Result<(), ContractError> {
    // Load current karma score
    let mut karma_score = KARMA_SCORES
        .may_load(deps.storage, agent_address.as_str())?
        .unwrap_or_default();
    
    // Apply penalty
    let penalty_amount = violation.penalty_applied;
    karma_score.current_score = karma_score.current_score.saturating_sub(penalty_amount);
    karma_score.last_updated = env.block.time;
    
    // Save updated karma score
    KARMA_SCORES.save(deps.storage, agent_address.as_str(), &karma_score)?;
    
    // Record penalty in tracking
    let penalty_key = format!("{}:{}", agent_address, env.block.time.seconds());
    KARMA_PENALTIES.save(deps.storage, &penalty_key, &penalty_amount)?;
    
    Ok(())
}

/// Check reputation-based rate limiting
pub fn check_rate_limit(
    deps: DepsMut,
    env: &Env,
    agent_address: &Addr,
    action_type: &str,
) -> Result<bool, ContractError> {
    let current_time = env.block.time;
    let window_start = current_time.minus_seconds(RATING_PATTERN_WINDOW);
    
    // Get current karma score for reputation-based limits
    let karma_score = KARMA_SCORES
        .may_load(deps.storage, agent_address.as_str())?
        .unwrap_or_default();
    
    // Calculate rate limit based on karma (higher karma = higher limits)
    let base_limit = match action_type {
        "rating" => SPAM_RATING_THRESHOLD,
        "interaction" => BOT_BEHAVIOR_THRESHOLD,
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
    
    // Get or create rate limit tracker
    let tracker_key = agent_address.as_str();
    let mut tracker = RATE_LIMIT_TRACKERS
        .may_load(deps.storage, tracker_key)?
        .unwrap_or(RateLimitTracker {
            agent_address: agent_address.clone(),
            action_type: action_type.to_string(),
            count: 0,
            window_start: current_time,
            last_action: current_time,
        });
    
    // Reset counter if window has passed
    if current_time.seconds() - tracker.window_start.seconds() > RATING_PATTERN_WINDOW {
        tracker.count = 0;
        tracker.window_start = current_time;
    }
    
    // Check if limit would be exceeded
    if tracker.count >= effective_limit {
        return Ok(false); // Rate limit exceeded
    }
    
    // Update tracker
    tracker.count += 1;
    tracker.last_action = current_time;
    RATE_LIMIT_TRACKERS.save(deps.storage, tracker_key, &tracker)?;
    
    Ok(true) // Rate limit OK
}

/// Create a dispute case for a compliance violation
pub fn create_dispute(
    deps: DepsMut,
    env: &Env,
    challenger: &Addr,
    violation_id: String,
    stake_amount: Uint128,
    evidence: String,
) -> Result<String, ContractError> {
    // Verify challenger has enough karma to stake
    let challenger_karma = KARMA_SCORES
        .may_load(deps.storage, challenger.as_str())?
        .unwrap_or_default();
    
    if challenger_karma.current_score < stake_amount {
        return Err(ContractError::InsufficientKarma {
            required: stake_amount.u128(),
            current: challenger_karma.current_score.u128(),
        });
    }
    
    // Generate unique case ID
    let case_id = format!("dispute_{}_{}", violation_id, env.block.time.seconds());
    
    // Create dispute case
    let dispute_case = DisputeCase {
        case_id: case_id.clone(),
        violation_id,
        challenger: challenger.clone(),
        stake_amount,
        evidence,
        status: DisputeStatus::Pending,
        created_at: env.block.time,
        resolved_at: None,
        resolution: None,
    };
    
    // Save dispute case
    DISPUTE_CASES.save(deps.storage, &case_id, &dispute_case)?;
    
    // Deduct stake from challenger's karma
    let mut updated_karma = challenger_karma;
    updated_karma.current_score = updated_karma.current_score.saturating_sub(stake_amount);
    KARMA_SCORES.save(deps.storage, challenger.as_str(), &updated_karma)?;
    
    Ok(case_id)
}

/// Resolve a dispute case
pub fn resolve_dispute(
    deps: DepsMut,
    env: &Env,
    case_id: &str,
    resolution: DisputeResolution,
) -> Result<(), ContractError> {
    let mut dispute_case = DISPUTE_CASES.load(deps.storage, case_id)?;
    
    if dispute_case.status != DisputeStatus::Pending {
        return Err(ContractError::DisputeAlreadyResolved {
            case_id: case_id.to_string(),
        });
    }
    
    // Update dispute case
    dispute_case.status = DisputeStatus::Resolved;
    dispute_case.resolved_at = Some(env.block.time);
    dispute_case.resolution = Some(resolution.clone());
    
    // Handle resolution
    match resolution {
        DisputeResolution::ViolationOverturned => {
            // Return stake to challenger and reverse penalty
            let mut challenger_karma = KARMA_SCORES
                .load(deps.storage, dispute_case.challenger.as_str())?;
            challenger_karma.current_score += dispute_case.stake_amount;
            KARMA_SCORES.save(deps.storage, dispute_case.challenger.as_str(), &challenger_karma)?;
            
            // TODO: Reverse the original violation penalty
        }
        DisputeResolution::ViolationConfirmed => {
            // Stake is forfeited (already deducted)
        }
        DisputeResolution::PartialOverturned => {
            // Return half the stake
            let partial_return = dispute_case.stake_amount.checked_div(Uint128::from(2u128)).unwrap_or_default();
            let mut challenger_karma = KARMA_SCORES
                .load(deps.storage, dispute_case.challenger.as_str())?;
            challenger_karma.current_score += partial_return;
            KARMA_SCORES.save(deps.storage, dispute_case.challenger.as_str(), &challenger_karma)?;
        }
    }
    
    // Save updated dispute case
    DISPUTE_CASES.save(deps.storage, case_id, &dispute_case)?;
    
    Ok(())
}

/// Run comprehensive abuse detection on an agent
pub fn run_abuse_detection(
    deps: Deps,
    env: &Env,
    agent_address: &Addr,
) -> StdResult<Vec<AbuseDetectionResult>> {
    let mut results = Vec::new();
    
    // Run all detection algorithms
    results.push(detect_spam_ratings(deps, env, agent_address)?);
    results.push(detect_bot_behavior(deps, env, agent_address)?);
    results.push(detect_rating_manipulation(deps, env, agent_address)?);
    
    Ok(results)
}

/// Helper function to calculate variance of rating scores
pub fn calculate_variance(scores: &[u8]) -> f64 {
    if scores.len() < 2 {
        return 1.0; // Default to high variance for small samples
    }
    
    let mean = scores.iter().map(|&x| x as f64).sum::<f64>() / scores.len() as f64;
    let variance = scores
        .iter()
        .map(|&x| {
            let diff = x as f64 - mean;
            diff * diff
        })
        .sum::<f64>() / scores.len() as f64;
    
    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{Addr, Timestamp};
    
    #[test]
    fn test_calculate_variance() {
        let scores = vec![5, 5, 5, 5, 5]; // No variance
        assert!(calculate_variance(&scores) < 0.1);
        
        let scores = vec![1, 3, 5, 7, 9]; // High variance
        assert!(calculate_variance(&scores) > 2.0);
    }
    
    #[test]
    fn test_spam_detection_high_frequency() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let agent = Addr::unchecked("agent1");
        
        // This would require setting up mock ratings data
        // Implementation would test the spam detection logic
    }
    
    #[test]
    fn test_rate_limiting() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let agent = Addr::unchecked("agent1");
        
        // Test rate limiting logic
        let result = check_rate_limit(deps.as_mut(), &env, &agent, "rating");
        assert!(result.is_ok());
    }
}