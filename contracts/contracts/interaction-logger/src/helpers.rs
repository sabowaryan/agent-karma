use cosmwasm_std::{Addr, Deps, DepsMut, Env, StdResult, Timestamp};
use sha2::{Sha256, Digest};
use crate::error::ContractError;
use crate::state::{Config, StoredInteraction, FailedOperation, CONFIG, FAILED_OPERATIONS};
use agent_karma_contracts::types::{Interaction, InteractionMetadata};

/// Generate a cryptographic hash for an interaction
pub fn generate_interaction_hash(
    participants: &[Addr],
    interaction_type: &str,
    timestamp: &Timestamp,
    metadata: &InteractionMetadata,
) -> String {
    let mut hasher = Sha256::new();
    
    // Hash participants (sorted for consistency)
    let mut sorted_participants: Vec<String> = participants.iter()
        .map(|addr| addr.to_string())
        .collect();
    sorted_participants.sort();
    
    for participant in sorted_participants {
        hasher.update(participant.as_bytes());
    }
    
    // Hash interaction type
    hasher.update(interaction_type.as_bytes());
    
    // Hash timestamp
    hasher.update(timestamp.seconds().to_be_bytes());
    
    // Hash metadata
    if let Some(duration) = metadata.duration {
        hasher.update(duration.to_be_bytes());
    }
    if let Some(ref outcome) = metadata.outcome {
        hasher.update(outcome.as_bytes());
    }
    if let Some(ref context) = metadata.context {
        hasher.update(context.as_bytes());
    }
    
    let result = hasher.finalize();
    hex::encode(result)
}

/// Validate interaction hash format
pub fn validate_interaction_hash(hash: &str) -> Result<(), ContractError> {
    // Check length (SHA256 produces 64 hex characters)
    if hash.len() != 64 {
        return Err(ContractError::InvalidInteractionHash {});
    }
    
    // Check if all characters are valid hex
    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ContractError::InvalidInteractionHash {});
    }
    
    Ok(())
}

/// Validate participants list
pub fn validate_participants(
    participants: &[String],
    config: &Config,
) -> Result<Vec<Addr>, ContractError> {
    // Check minimum participants
    if participants.is_empty() {
        return Err(ContractError::InvalidParticipants {
            reason: "At least one participant is required".to_string(),
        });
    }
    
    // Check maximum participants
    if participants.len() > config.max_participants as usize {
        return Err(ContractError::InvalidParticipants {
            reason: format!(
                "Too many participants: max {}, provided {}",
                config.max_participants,
                participants.len()
            ),
        });
    }
    
    // Validate addresses and remove duplicates
    let mut validated_participants = Vec::new();
    let mut seen_addresses = std::collections::HashSet::new();
    
    for participant_str in participants {
        // Validate address format
        let addr = cosmwasm_std::Addr::unchecked(participant_str);
        
        // Check for duplicates
        if seen_addresses.contains(&addr.to_string()) {
            return Err(ContractError::InvalidParticipants {
                reason: format!("Duplicate participant: {}", addr),
            });
        }
        
        seen_addresses.insert(addr.to_string());
        validated_participants.push(addr);
    }
    
    Ok(validated_participants)
}

/// Validate interaction type
pub fn validate_interaction_type(
    interaction_type: &str,
    config: &Config,
) -> Result<(), ContractError> {
    if interaction_type.is_empty() {
        return Err(ContractError::InvalidInteractionType {
            interaction_type: interaction_type.to_string(),
        });
    }
    
    if !config.valid_interaction_types.contains(&interaction_type.to_string()) {
        return Err(ContractError::InvalidInteractionType {
            interaction_type: interaction_type.to_string(),
        });
    }
    
    Ok(())
}

/// Validate interaction metadata
pub fn validate_metadata(metadata: &InteractionMetadata) -> Result<(), ContractError> {
    // Validate duration if provided
    if let Some(duration) = metadata.duration {
        if duration == 0 {
            return Err(ContractError::MetadataValidationFailed {
                reason: "Duration cannot be zero".to_string(),
            });
        }
        
        // Reasonable upper limit (24 hours in seconds)
        if duration > 24 * 60 * 60 {
            return Err(ContractError::MetadataValidationFailed {
                reason: "Duration exceeds maximum allowed (24 hours)".to_string(),
            });
        }
    }
    
    // Validate outcome length if provided
    if let Some(ref outcome) = metadata.outcome {
        if outcome.len() > 500 {
            return Err(ContractError::MetadataValidationFailed {
                reason: "Outcome text too long (max 500 characters)".to_string(),
            });
        }
    }
    
    // Validate context length if provided
    if let Some(ref context) = metadata.context {
        if context.len() > 1000 {
            return Err(ContractError::MetadataValidationFailed {
                reason: "Context text too long (max 1000 characters)".to_string(),
            });
        }
    }
    
    Ok(())
}

/// Validate pagination parameters
pub fn validate_pagination(
    limit: Option<u32>,
    config: &Config,
) -> Result<u32, ContractError> {
    let limit = limit.unwrap_or(50); // Default limit
    
    if limit == 0 {
        return Err(ContractError::PaginationLimitExceeded {
            max: config.max_pagination_limit,
            requested: limit,
        });
    }
    
    if limit > config.max_pagination_limit {
        return Err(ContractError::PaginationLimitExceeded {
            max: config.max_pagination_limit,
            requested: limit,
        });
    }
    
    Ok(limit)
}

/// Verify interaction cryptographically
pub fn verify_interaction_integrity(
    interaction: &Interaction,
    expected_hash: &str,
) -> Result<bool, ContractError> {
    let calculated_hash = generate_interaction_hash(
        &interaction.participants,
        &interaction.interaction_type,
        &interaction.timestamp,
        &interaction.metadata,
    );
    
    Ok(calculated_hash == expected_hash)
}

/// Retry mechanism for storage operations
pub fn retry_storage_operation<F, T>(
    mut deps: DepsMut,
    env: &Env,
    operation_id: &str,
    mut operation: F,
    max_attempts: u32,
) -> Result<T, ContractError>
where
    F: FnMut() -> StdResult<T>,
{
    let mut attempts = 0;
    let mut last_error = String::new();
    
    // Check if this operation has been attempted before
    if let Ok(failed_op) = FAILED_OPERATIONS.load(deps.storage, operation_id) {
        attempts = failed_op.attempts;
        last_error = failed_op.last_error;
    }
    
    while attempts < max_attempts {
        attempts += 1;
        
        match operation() {
            Ok(result) => {
                // Success - remove from failed operations if it was there
                FAILED_OPERATIONS.remove(deps.storage, operation_id);
                return Ok(result);
            }
            Err(err) => {
                last_error = err.to_string();
                
                // Save failed operation for potential retry
                let failed_op = FailedOperation {
                    operation_type: "storage".to_string(),
                    data: operation_id.to_string(),
                    attempts,
                    last_error: last_error.clone(),
                    last_attempt: env.block.time,
                };
                
                // Don't fail if we can't save the failed operation
                let _ = FAILED_OPERATIONS.save(deps.storage, operation_id, &failed_op);
            }
        }
    }
    
    Err(ContractError::StorageOperationFailed {
        attempts,
        reason: last_error,
    })
}

/// Generate unique interaction ID
pub fn generate_interaction_id(counter: u64, timestamp: &Timestamp) -> String {
    format!("interaction_{}_{}", counter, timestamp.seconds())
}

/// Check if agent is registered (placeholder - would query agent registry in production)
pub fn check_agent_registered(
    _deps: Deps,
    _agent_address: &Addr,
) -> Result<(), ContractError> {
    // In a real implementation, this would query the agent registry contract
    // For now, we'll assume all agents are registered
    Ok(())
}

/// Validate timestamp is not in the future
pub fn validate_timestamp(
    timestamp: &Timestamp,
    current_time: &Timestamp,
) -> Result<(), ContractError> {
    if timestamp.seconds() > current_time.seconds() {
        return Err(ContractError::InvalidTimestamp {
            reason: "Timestamp cannot be in the future".to_string(),
        });
    }
    
    // Check if timestamp is too old (more than 30 days)
    let thirty_days_ago = current_time.minus_seconds(30 * 24 * 60 * 60);
    if timestamp.seconds() < thirty_days_ago.seconds() {
        return Err(ContractError::InvalidTimestamp {
            reason: "Timestamp is too old (more than 30 days)".to_string(),
        });
    }
    
    Ok(())
}

/// Get configuration with error handling
pub fn get_config(deps: Deps) -> Result<Config, ContractError> {
    CONFIG.load(deps.storage).map_err(ContractError::Std)
}

/// Sanitize string input to prevent injection attacks
pub fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".,!?-_()[]{}:;".contains(*c))
        .collect::<String>()
        .trim()
        .to_string()
}

/// Check if interaction exists
pub fn interaction_exists(deps: Deps, interaction_hash: &str) -> StdResult<bool> {
    use crate::state::interactions;
    Ok(interactions().has(deps.storage, interaction_hash))
}

/// Get interaction by hash with error handling
pub fn get_interaction_by_hash(
    deps: Deps,
    interaction_hash: &str,
) -> Result<Option<StoredInteraction>, ContractError> {
    validate_interaction_hash(interaction_hash)?;
    
    use crate::state::interactions;
    match interactions().may_load(deps.storage, interaction_hash) {
        Ok(interaction) => Ok(interaction),
        Err(err) => Err(ContractError::Std(err)),
    }
}