use cosmwasm_std::{Addr, Deps, DepsMut, Env, StdResult, Timestamp, Uint128};
use crate::error::ContractError;
use crate::state::{KARMA_SCORES, KARMA_HISTORY, ratings, CONFIG, ORACLE_DATA, KarmaScore};
use crate::helpers::validate_interaction_hash;
use agent_karma_contracts::types::{KarmaCalculation, KarmaFactors, Rating};
use sha2::{Sha256, Digest};
use std::str::FromStr;

/// Time decay constants
const DECAY_WEEK_THRESHOLD: u64 = 7 * 24 * 60 * 60; // 7 days in seconds
const DECAY_MONTH_THRESHOLD: u64 = 30 * 24 * 60 * 60; // 30 days in seconds
const DECAY_QUARTER_THRESHOLD: u64 = 90 * 24 * 60 * 60; // 90 days in seconds

/// Karma calculation weights
const BASE_RATING_WEIGHT: u128 = 100;
const INTERACTION_BONUS_WEIGHT: u128 = 10;
const HIGH_KARMA_INTERACTION_BONUS: u128 = 20;
const LOW_KARMA_INTERACTION_PENALTY: u128 = 5;
const CONSISTENCY_BONUS_THRESHOLD: u64 = 10;
const CONSISTENCY_BONUS_VALUE: u128 = 50;

/// Oracle data weights (as percentages)
const PERFORMANCE_WEIGHT: u8 = 15;
const CROSS_CHAIN_WEIGHT: u8 = 10;
const SENTIMENT_WEIGHT: u8 = 5;

/// Calculate karma score for an agent using the comprehensive algorithm
/// 
/// This function implements the core karma calculation algorithm with:
/// - Base score from ratings (weighted by rating quality)
/// - Time decay factor (reduces karma for inactive agents)
/// - Interaction bonus (rewards active participation)
/// - Contextual modifiers (bonuses/penalties based on behavior patterns)
/// - External factors from oracle data integration
pub fn calculate_karma_score(
    deps: Deps,
    env: &Env,
    agent_address: &Addr,
) -> Result<KarmaCalculation, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    // Get current karma score or initialize
    let current_karma = KARMA_SCORES
        .may_load(deps.storage, &agent_address.to_string())?
        .unwrap_or_else(|| KarmaScore {
            current_score: Uint128::zero(),
            previous_score: Uint128::zero(),
            last_updated: env.block.time,
            total_ratings: 0,
            average_rating: "0.0".to_string(),
            interaction_count: 0,
        });

    // Get all ratings for this agent
    let ratings_result: StdResult<Vec<_>> = ratings()
        .idx
        .rated_agent
        .prefix(agent_address.to_string())
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect();
    
    let agent_ratings = ratings_result?;
    
    if agent_ratings.is_empty() {
        // No ratings yet, return zero karma but preserve any existing score for time decay
        let time_decay = calculate_time_decay(deps, env, agent_address)?;
        let decayed_score = apply_time_decay(current_karma.current_score, time_decay);
        
        return Ok(KarmaCalculation {
            agent_address: agent_address.clone(),
            current_score: decayed_score,
            previous_score: current_karma.current_score,
            factors: KarmaFactors {
                average_rating: "0.0".to_string(),
                rating_count: 0,
                interaction_frequency: Uint128::zero(),
                time_decay: format!("{:.3}", time_decay),
                external_factors: Some(Uint128::zero()),
            },
            last_updated: env.block.time,
            calculation_hash: generate_calculation_hash(agent_address, &env.block.time, &decayed_score),
        });
    }

    // Calculate base score from ratings with improved weighting
    let (base_score, average_rating, rating_count) = calculate_base_score_enhanced(&agent_ratings)?;
    
    // Calculate time decay factor based on last activity
    let time_decay = calculate_time_decay(deps, env, agent_address)?;
    
    // Calculate interaction bonus with frequency and quality considerations
    let interaction_bonus = calculate_interaction_bonus_enhanced(deps, agent_address, &agent_ratings)?;
    
    // Calculate contextual modifiers with comprehensive behavior analysis
    let contextual_modifier = calculate_contextual_modifiers_enhanced(deps, agent_address, &agent_ratings)?;
    
    // Calculate external factors from oracle data with proper weighting
    let external_factors = calculate_external_factors_enhanced(deps, agent_address)?;
    
    // Apply sophisticated karma combination algorithm
    let base_with_decay = apply_time_decay(base_score, time_decay);
    let interaction_adjusted = base_with_decay.checked_add(interaction_bonus)?;
    let context_adjusted = apply_contextual_modifiers(interaction_adjusted, contextual_modifier)?;
    let final_score = context_adjusted.checked_add(external_factors)?;
    
    // Ensure minimum score is 0 and apply maximum cap if needed
    let final_score = std::cmp::max(final_score, Uint128::zero());
    let final_score = std::cmp::min(final_score, Uint128::from(10000u128)); // Max karma cap
    
    let calculation = KarmaCalculation {
        agent_address: agent_address.clone(),
        current_score: final_score,
        previous_score: current_karma.current_score,
        factors: KarmaFactors {
            average_rating,
            rating_count,
            interaction_frequency: interaction_bonus,
            time_decay: format!("{:.3}", time_decay),
            external_factors: Some(external_factors),
        },
        last_updated: env.block.time,
        calculation_hash: generate_calculation_hash(agent_address, &env.block.time, &final_score),
    };
    
    Ok(calculation)
}

/// Calculate base karma score from ratings with enhanced weighting
fn calculate_base_score_enhanced(ratings: &[(String, crate::state::StoredRating)]) -> Result<(Uint128, String, u64), ContractError> {
    if ratings.is_empty() {
        return Ok((Uint128::zero(), "0.0".to_string(), 0));
    }
    
    let rating_count = ratings.len() as u64;
    
    // Calculate weighted average with recency bias
    let mut total_weighted_score = 0.0;
    let mut total_weight = 0.0;
    let current_time = cosmwasm_std::Timestamp::from_seconds(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );
    
    for (_, stored_rating) in ratings.iter() {
        let rating_score = stored_rating.rating.score as f64;
        
        // Apply recency weight (more recent ratings have higher weight)
        let age_seconds = current_time.seconds().saturating_sub(stored_rating.rating.timestamp.seconds());
        let age_days = age_seconds as f64 / (24.0 * 60.0 * 60.0);
        let recency_weight = 1.0 / (1.0 + age_days * 0.01); // Gradual decay over time
        
        // Apply quality weight (extreme ratings get slightly less weight to prevent gaming)
        let quality_weight = if rating_score <= 2.0 || rating_score >= 9.0 {
            0.8 // Reduce weight for extreme ratings
        } else {
            1.0
        };
        
        let final_weight = recency_weight * quality_weight;
        total_weighted_score += rating_score * final_weight;
        total_weight += final_weight;
    }
    
    let weighted_average = if total_weight > 0.0 {
        total_weighted_score / total_weight
    } else {
        0.0
    };
    
    // Enhanced karma calculation with non-linear scaling
    let base_karma = if weighted_average >= 5.0 {
        // Positive karma with exponential scaling for high ratings
        let positive_factor = (weighted_average - 5.0) / 5.0; // 0.0 to 1.0
        let exponential_factor = positive_factor * positive_factor; // Square for exponential growth
        let karma_points = (exponential_factor * BASE_RATING_WEIGHT as f64 * rating_count as f64) as u128;
        Uint128::from(karma_points)
    } else {
        // Minimal karma for poor ratings (handled in contextual modifiers)
        let poor_factor = (5.0 - weighted_average) / 4.0;
        let minimal_karma = ((1.0 - poor_factor) * BASE_RATING_WEIGHT as f64 * 0.1) as u128;
        Uint128::from(minimal_karma.max(0))
    };
    
    Ok((base_karma, format!("{:.2}", weighted_average), rating_count))
}

/// Legacy function for backward compatibility
fn calculate_base_score(ratings: &[(String, crate::state::StoredRating)]) -> Result<(Uint128, String, u64), ContractError> {
    calculate_base_score_enhanced(ratings)
}

/// Calculate time decay factor based on last activity
fn calculate_time_decay(deps: Deps, env: &Env, agent_address: &Addr) -> Result<f64, ContractError> {
    let current_karma = KARMA_SCORES
        .may_load(deps.storage, &agent_address.to_string())?;
    
    let last_activity = match current_karma {
        Some(karma) => karma.last_updated,
        None => env.block.time, // New agent, no decay
    };
    
    let time_since_activity = env.block.time.seconds() - last_activity.seconds();
    
    let decay_factor = if time_since_activity <= DECAY_WEEK_THRESHOLD {
        1.0 // No decay for 1 week
    } else if time_since_activity <= DECAY_MONTH_THRESHOLD {
        0.95 // 5% decay after 1 month
    } else if time_since_activity <= DECAY_QUARTER_THRESHOLD {
        0.85 // 15% decay after 3 months
    } else {
        0.7 // 30% decay after 3+ months
    };
    
    Ok(decay_factor)
}

/// Apply time decay to base score
fn apply_time_decay(base_score: Uint128, decay_factor: f64) -> Uint128 {
    let decayed_score = (base_score.u128() as f64 * decay_factor) as u128;
    Uint128::from(decayed_score)
}

/// Calculate interaction bonus with enhanced frequency and quality considerations
fn calculate_interaction_bonus_enhanced(
    deps: Deps,
    agent_address: &Addr,
    ratings: &[(String, crate::state::StoredRating)],
) -> Result<Uint128, ContractError> {
    let interaction_count = ratings.len() as u128;
    
    if interaction_count == 0 {
        return Ok(Uint128::zero());
    }
    
    // Base interaction bonus with diminishing returns
    let base_bonus = if interaction_count <= 10 {
        interaction_count * INTERACTION_BONUS_WEIGHT
    } else if interaction_count <= 50 {
        10 * INTERACTION_BONUS_WEIGHT + (interaction_count - 10) * (INTERACTION_BONUS_WEIGHT / 2)
    } else {
        10 * INTERACTION_BONUS_WEIGHT + 40 * (INTERACTION_BONUS_WEIGHT / 2) + (interaction_count - 50) * (INTERACTION_BONUS_WEIGHT / 4)
    };
    
    // Frequency bonus for consistent activity
    let current_time = cosmwasm_std::Timestamp::from_seconds(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );
    
    // Check activity in last 30 days
    let recent_ratings = ratings.iter()
        .filter(|(_, stored_rating)| {
            let age_seconds = current_time.seconds().saturating_sub(stored_rating.rating.timestamp.seconds());
            age_seconds <= (30 * 24 * 60 * 60) // 30 days
        })
        .count() as u128;
    
    let frequency_bonus = if recent_ratings >= 5 {
        INTERACTION_BONUS_WEIGHT * 2 // Bonus for recent activity
    } else {
        0
    };
    
    // Quality interaction bonus (interacting with diverse agents)
    let unique_raters: std::collections::HashSet<_> = ratings.iter()
        .map(|(_, stored_rating)| stored_rating.rating.rater_address.clone())
        .collect();
    
    let diversity_bonus = if unique_raters.len() >= 5 {
        INTERACTION_BONUS_WEIGHT // Bonus for diverse interactions
    } else {
        0
    };
    
    let total_bonus = base_bonus + frequency_bonus + diversity_bonus;
    Ok(Uint128::from(total_bonus))
}

/// Legacy function for backward compatibility
fn calculate_interaction_bonus(
    deps: Deps,
    agent_address: &Addr,
    ratings: &[(String, crate::state::StoredRating)],
) -> Result<Uint128, ContractError> {
    calculate_interaction_bonus_enhanced(deps, agent_address, ratings)
}

/// Calculate contextual modifiers with comprehensive behavior analysis
fn calculate_contextual_modifiers_enhanced(
    deps: Deps,
    agent_address: &Addr,
    ratings: &[(String, crate::state::StoredRating)],
) -> Result<i128, ContractError> {
    let mut modifier = 0i128;
    
    if ratings.is_empty() {
        return Ok(0);
    }
    
    // Consistency bonus for agents with many high ratings
    let high_ratings = ratings.iter()
        .filter(|(_, stored_rating)| stored_rating.rating.score >= 8)
        .count() as u64;
    
    if high_ratings >= CONSISTENCY_BONUS_THRESHOLD {
        modifier += CONSISTENCY_BONUS_VALUE as i128;
    }
    
    // Excellence bonus for exceptional performance (90%+ ratings above 8)
    let total_ratings = ratings.len() as u64;
    if total_ratings >= 20 && high_ratings as f64 / total_ratings as f64 >= 0.9 {
        modifier += (CONSISTENCY_BONUS_VALUE * 2) as i128; // Double bonus for excellence
    }
    
    // Penalty for poor ratings with escalating severity
    let poor_ratings = ratings.iter()
        .filter(|(_, stored_rating)| stored_rating.rating.score < 4)
        .count() as u64;
    
    let very_poor_ratings = ratings.iter()
        .filter(|(_, stored_rating)| stored_rating.rating.score <= 2)
        .count() as u64;
    
    // Progressive penalty system
    modifier -= (poor_ratings as u128 * LOW_KARMA_INTERACTION_PENALTY) as i128;
    modifier -= (very_poor_ratings as u128 * LOW_KARMA_INTERACTION_PENALTY * 2) as i128; // Double penalty for very poor ratings
    
    // Improvement bonus for agents showing positive trend
    if ratings.len() >= 10 {
        let recent_half = &ratings[ratings.len()/2..];
        let early_half = &ratings[..ratings.len()/2];
        
        let recent_avg: f64 = recent_half.iter()
            .map(|(_, r)| r.rating.score as f64)
            .sum::<f64>() / recent_half.len() as f64;
        
        let early_avg: f64 = early_half.iter()
            .map(|(_, r)| r.rating.score as f64)
            .sum::<f64>() / early_half.len() as f64;
        
        if recent_avg > early_avg + 1.0 { // Significant improvement
            modifier += (INTERACTION_BONUS_WEIGHT * 3) as i128; // Improvement bonus
        }
    }
    
    // Interaction quality bonus based on rater karma
    let mut high_karma_interactions = 0u64;
    let mut low_karma_interactions = 0u64;
    
    for (_, stored_rating) in ratings.iter() {
        let rater_karma = KARMA_SCORES
            .may_load(deps.storage, &stored_rating.rating.rater_address.to_string())?
            .map(|k| k.current_score.u128())
            .unwrap_or(0);
        
        if rater_karma >= 500 {
            high_karma_interactions += 1;
        } else if rater_karma < 100 {
            low_karma_interactions += 1;
        }
    }
    
    // Bonus for interactions with high-karma agents
    modifier += (high_karma_interactions as u128 * HIGH_KARMA_INTERACTION_BONUS) as i128;
    
    // Small penalty for too many interactions with low-karma agents (potential gaming)
    if low_karma_interactions > total_ratings / 2 {
        modifier -= (LOW_KARMA_INTERACTION_PENALTY * 2) as i128;
    }
    
    Ok(modifier)
}

/// Apply contextual modifiers to karma score
fn apply_contextual_modifiers(base_score: Uint128, modifier: i128) -> Result<Uint128, ContractError> {
    if modifier >= 0 {
        Ok(base_score.checked_add(Uint128::from(modifier as u128))?)
    } else {
        let penalty = (-modifier) as u128;
        let penalty_amount = Uint128::from(penalty);
        if base_score >= penalty_amount {
            Ok(base_score.checked_sub(penalty_amount)?)
        } else {
            Ok(Uint128::zero()) // Don't go below zero
        }
    }
}

/// Legacy function for backward compatibility
fn calculate_contextual_modifiers(
    deps: Deps,
    agent_address: &Addr,
    ratings: &[(String, crate::state::StoredRating)],
) -> Result<Uint128, ContractError> {
    let modifier = calculate_contextual_modifiers_enhanced(deps, agent_address, ratings)?;
    Ok(Uint128::from(std::cmp::max(modifier, 0) as u128))
}

/// Calculate external factors from oracle data with proper weighting
fn calculate_external_factors_enhanced(deps: Deps, agent_address: &Addr) -> Result<Uint128, ContractError> {
    let agent_str = agent_address.to_string();
    
    // Get performance metrics (15% weight)
    let performance_data = ORACLE_DATA
        .may_load(deps.storage, (&agent_str, "performance"))?;
    
    // Get cross-chain reputation (10% weight)
    let cross_chain_data = ORACLE_DATA
        .may_load(deps.storage, (&agent_str, "cross_chain"))?;
    
    // Get sentiment data (5% weight)
    let sentiment_data = ORACLE_DATA
        .may_load(deps.storage, (&agent_str, "sentiment"))?;
    
    let mut external_bonus = 0u128;
    
    // Process performance data with enhanced calculation
    if let Some(perf_hash) = performance_data {
        // Parse performance data from hash (simplified implementation)
        // In production, this would decode actual oracle data
        let performance_score = parse_oracle_performance_data(&perf_hash)?;
        let performance_bonus = (performance_score as f64 * PERFORMANCE_WEIGHT as f64 * 2.0) as u128;
        external_bonus += performance_bonus;
    }
    
    // Process cross-chain data with reputation scaling
    if let Some(cross_hash) = cross_chain_data {
        let cross_chain_score = parse_oracle_cross_chain_data(&cross_hash)?;
        let cross_chain_bonus = (cross_chain_score as f64 * CROSS_CHAIN_WEIGHT as f64 * 1.5) as u128;
        external_bonus += cross_chain_bonus;
    }
    
    // Process sentiment data with community weighting
    if let Some(sentiment_hash) = sentiment_data {
        let sentiment_score = parse_oracle_sentiment_data(&sentiment_hash)?;
        let sentiment_bonus = (sentiment_score as f64 * SENTIMENT_WEIGHT as f64 * 1.0) as u128;
        external_bonus += sentiment_bonus;
    }
    
    // Apply oracle data freshness factor
    let freshness_factor = calculate_oracle_freshness_factor(deps, agent_address)?;
    let adjusted_bonus = (external_bonus as f64 * freshness_factor) as u128;
    
    Ok(Uint128::from(adjusted_bonus))
}

/// Parse performance data from oracle hash (simplified implementation)
fn parse_oracle_performance_data(data_hash: &str) -> Result<u32, ContractError> {
    // In a real implementation, this would decode actual oracle data
    // For now, we'll use a hash-based score calculation that gives meaningful results
    let hash_sum: u32 = data_hash.chars()
        .filter_map(|c| c.to_digit(16))
        .sum();
    
    // Normalize to 60-100 range (performance data tends to be positive)
    let performance_score = ((hash_sum % 41) + 60) as u32;
    Ok(performance_score)
}

/// Parse cross-chain reputation data from oracle hash
fn parse_oracle_cross_chain_data(data_hash: &str) -> Result<u32, ContractError> {
    // Simplified cross-chain reputation calculation
    let hash_sum: u32 = data_hash.chars()
        .filter_map(|c| c.to_digit(16))
        .skip(1) // Different offset for variety
        .sum();
    
    let cross_chain_score = ((hash_sum % 81) + 20) as u32; // 20-100 range
    Ok(cross_chain_score)
}

/// Parse sentiment data from oracle hash
fn parse_oracle_sentiment_data(data_hash: &str) -> Result<u32, ContractError> {
    // Simplified sentiment calculation
    let hash_sum: u32 = data_hash.chars()
        .filter_map(|c| c.to_digit(16))
        .skip(2) // Different offset for variety
        .sum();
    
    let sentiment_score = ((hash_sum % 61) + 40) as u32; // 40-100 range (sentiment tends to be positive)
    Ok(sentiment_score)
}

/// Calculate oracle data freshness factor
fn calculate_oracle_freshness_factor(deps: Deps, agent_address: &Addr) -> Result<f64, ContractError> {
    // In a real implementation, this would check oracle data timestamps
    // For now, we'll return a default freshness factor
    Ok(0.9) // 90% freshness factor
}

/// Legacy function for backward compatibility
fn calculate_external_factors(deps: Deps, agent_address: &Addr) -> Result<Uint128, ContractError> {
    calculate_external_factors_enhanced(deps, agent_address)
}

/// Generate a hash for karma calculation verification
fn generate_calculation_hash(agent_address: &Addr, timestamp: &Timestamp, score: &Uint128) -> String {
    let mut hasher = Sha256::new();
    hasher.update(agent_address.as_bytes());
    hasher.update(timestamp.seconds().to_be_bytes());
    hasher.update(score.u128().to_be_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Update karma score and save to storage
pub fn update_karma_score(
    deps: DepsMut,
    env: &Env,
    agent_address: &Addr,
    calculation: &KarmaCalculation,
) -> Result<(), ContractError> {
    let agent_str = agent_address.to_string();
    
    // Get current karma or create new
    let current_karma = KARMA_SCORES
        .may_load(deps.storage, &agent_str)?
        .unwrap_or_else(|| KarmaScore {
            current_score: Uint128::zero(),
            previous_score: Uint128::zero(),
            last_updated: env.block.time,
            total_ratings: 0,
            average_rating: "0.0".to_string(),
            interaction_count: 0,
        });
    
    // Update karma score
    let updated_karma = KarmaScore {
        current_score: calculation.current_score,
        previous_score: current_karma.current_score,
        last_updated: env.block.time,
        total_ratings: calculation.factors.rating_count,
        average_rating: calculation.factors.average_rating.clone(),
        interaction_count: current_karma.interaction_count + 1,
    };
    
    // Save updated karma
    KARMA_SCORES.save(deps.storage, &agent_str, &updated_karma)?;
    
    // Save to history
    let history_key = (agent_str.as_str(), env.block.time.seconds());
    KARMA_HISTORY.save(deps.storage, history_key, calculation)?;
    
    Ok(())
}

/// Validate rating score is within acceptable range
pub fn validate_rating_score(score: u8) -> Result<(), ContractError> {
    if score < 1 || score > 10 {
        return Err(ContractError::InvalidRatingScore { score });
    }
    Ok(())
}

/// Check if rating window is still valid (24 hours)
pub fn validate_rating_window(
    interaction_timestamp: &Timestamp,
    current_time: &Timestamp,
    window_seconds: u64,
) -> Result<(), ContractError> {
    let time_diff = current_time.seconds() - interaction_timestamp.seconds();
    if time_diff > window_seconds {
        return Err(ContractError::RatingWindowExpired {
            interaction_hash: "unknown".to_string(), // This would be passed in real implementation
        });
    }
    Ok(())
}

/// Enhanced 24-hour window validation with interaction hash lookup
pub fn validate_rating_window_with_hash(
    deps: Deps,
    interaction_hash: &str,
    current_time: &Timestamp,
    window_seconds: u64,
) -> Result<(), ContractError> {
    // In a real implementation, this would query the interaction logger contract
    // to get the actual interaction timestamp. For now, we'll simulate this
    // by checking if the interaction hash is valid format and assume recent timing
    
    validate_interaction_hash(interaction_hash)?;
    
    // For demonstration, we'll extract a timestamp from the hash pattern
    // In production, this would be a proper contract call
    let simulated_interaction_time = simulate_interaction_timestamp_from_hash(interaction_hash, current_time)?;
    
    validate_rating_window(&simulated_interaction_time, current_time, window_seconds)
}

/// Simulate interaction timestamp extraction from hash (for testing/demo)
fn simulate_interaction_timestamp_from_hash(
    interaction_hash: &str,
    current_time: &Timestamp,
) -> Result<Timestamp, ContractError> {
    // Extract last 8 characters and convert to a time offset
    let hash_suffix = &interaction_hash[interaction_hash.len()-8..];
    let hash_value = u64::from_str_radix(hash_suffix, 16)
        .map_err(|_| ContractError::InteractionNotFound {
            interaction_hash: interaction_hash.to_string(),
        })?;
    
    // Create a timestamp within the last 48 hours based on hash
    let offset_seconds = hash_value % (48 * 60 * 60); // Within 48 hours
    let interaction_time = current_time.minus_seconds(offset_seconds);
    
    Ok(interaction_time)
}