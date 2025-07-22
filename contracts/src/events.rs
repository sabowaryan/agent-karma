//! CosmWasm events for Agent-Karma smart contracts
//! 
//! This module defines all the events emitted by the smart contracts
//! for external monitoring and integration purposes.

use cosmwasm_std::{Addr, Event, Timestamp, Uint128};


/// Creates a standardized event with the contract name prefix
pub fn create_event(contract_name: &str, event_type: &str) -> Event {
    Event::new(format!("{}-{}", contract_name, event_type))
}

/// Agent Registry Events
pub struct AgentRegistryEvents;

impl AgentRegistryEvents {
    /// Event emitted when a new agent is registered
    /// 
    /// # Arguments
    /// * `agent_address` - Address of the registered agent
    /// * `name` - Display name of the agent
    /// * `framework` - AI framework used by the agent
    /// * `registration_time` - Timestamp of registration
    pub fn agent_registered(
        agent_address: &Addr,
        name: &str,
        framework: &str,
        registration_time: Timestamp,
    ) -> Event {
        create_event("agent-registry", "agent-registered")
            .add_attribute("agent_address", agent_address.to_string())
            .add_attribute("name", name)
            .add_attribute("framework", framework)
            .add_attribute("registration_time", registration_time.to_string())
    }

    /// Event emitted when agent metadata is updated
    /// 
    /// # Arguments
    /// * `agent_address` - Address of the agent
    /// * `updated_fields` - List of fields that were updated
    pub fn agent_metadata_updated(
        agent_address: &Addr,
        updated_fields: Vec<String>,
    ) -> Event {
        create_event("agent-registry", "metadata-updated")
            .add_attribute("agent_address", agent_address.to_string())
            .add_attribute("updated_fields", updated_fields.join(","))
    }
}

/// Karma Core Events
pub struct KarmaCoreEvents;

impl KarmaCoreEvents {
    /// Event emitted when a rating is submitted
    /// 
    /// # Arguments
    /// * `rater` - Address of the agent giving the rating
    /// * `rated_agent` - Address of the agent being rated
    /// * `score` - Rating score (1-10)
    /// * `interaction_hash` - Hash of the interaction being rated
    /// * `timestamp` - When the rating was submitted
    pub fn rating_submitted(
        rater: &Addr,
        rated_agent: &Addr,
        score: u8,
        interaction_hash: &str,
        timestamp: Timestamp,
    ) -> Event {
        create_event("karma-core", "rating-submitted")
            .add_attribute("rater", rater.to_string())
            .add_attribute("rated_agent", rated_agent.to_string())
            .add_attribute("score", score.to_string())
            .add_attribute("interaction_hash", interaction_hash)
            .add_attribute("timestamp", timestamp.to_string())
    }

    /// Event emitted when karma score is updated
    /// 
    /// # Arguments
    /// * `agent_address` - Address of the agent
    /// * `previous_score` - Previous karma score
    /// * `new_score` - New karma score
    /// * `calculation_hash` - Hash of the calculation for verification
    /// * `timestamp` - When the update occurred
    pub fn karma_updated(
        agent_address: &Addr,
        previous_score: Uint128,
        new_score: Uint128,
        calculation_hash: &str,
        timestamp: Timestamp,
    ) -> Event {
        create_event("karma-core", "karma-updated")
            .add_attribute("agent_address", agent_address.to_string())
            .add_attribute("previous_score", previous_score.to_string())
            .add_attribute("new_score", new_score.to_string())
            .add_attribute("calculation_hash", calculation_hash)
            .add_attribute("timestamp", timestamp.to_string())
    }

    /// Event emitted when karma is earned through positive interactions
    /// 
    /// # Arguments
    /// * `agent_address` - Address of the agent earning karma
    /// * `karma_earned` - Amount of karma earned
    /// * `source` - Source of karma (rating, interaction, governance, etc.)
    pub fn karma_earned(
        agent_address: &Addr,
        karma_earned: Uint128,
        source: &str,
    ) -> Event {
        create_event("karma-core", "karma-earned")
            .add_attribute("agent_address", agent_address.to_string())
            .add_attribute("karma_earned", karma_earned.to_string())
            .add_attribute("source", source)
    }

    /// Event emitted when karma is spent or burned
    /// 
    /// # Arguments
    /// * `agent_address` - Address of the agent losing karma
    /// * `karma_spent` - Amount of karma spent/burned
    /// * `reason` - Reason for karma loss (fee, penalty, etc.)
    pub fn karma_spent(
        agent_address: &Addr,
        karma_spent: Uint128,
        reason: &str,
    ) -> Event {
        create_event("karma-core", "karma-spent")
            .add_attribute("agent_address", agent_address.to_string())
            .add_attribute("karma_spent", karma_spent.to_string())
            .add_attribute("reason", reason)
    }
}

/// Interaction Logger Events
pub struct InteractionLoggerEvents;

impl InteractionLoggerEvents {
    /// Event emitted when an interaction is logged
    /// 
    /// # Arguments
    /// * `interaction_id` - Unique identifier for the interaction
    /// * `participants` - Addresses of agents involved
    /// * `interaction_type` - Type of interaction
    /// * `timestamp` - When the interaction occurred
    pub fn interaction_logged(
        interaction_id: &str,
        participants: &[Addr],
        interaction_type: &str,
        timestamp: Timestamp,
    ) -> Event {
        let participant_addresses: Vec<String> = participants
            .iter()
            .map(|addr| addr.to_string())
            .collect();

        create_event("interaction-logger", "interaction-logged")
            .add_attribute("interaction_id", interaction_id)
            .add_attribute("participants", participant_addresses.join(","))
            .add_attribute("interaction_type", interaction_type)
            .add_attribute("timestamp", timestamp.to_string())
    }

    /// Event emitted when an interaction is verified
    /// 
    /// # Arguments
    /// * `interaction_hash` - Hash of the verified interaction
    /// * `verifier` - Address of the verifying agent/contract
    /// * `verification_result` - Result of verification (true/false)
    pub fn interaction_verified(
        interaction_hash: &str,
        verifier: &Addr,
        verification_result: bool,
    ) -> Event {
        create_event("interaction-logger", "interaction-verified")
            .add_attribute("interaction_hash", interaction_hash)
            .add_attribute("verifier", verifier.to_string())
            .add_attribute("verification_result", verification_result.to_string())
    }
}

/// Governance DAO Events
pub struct GovernanceEvents;

impl GovernanceEvents {
    /// Event emitted when a new proposal is created
    /// 
    /// # Arguments
    /// * `proposal_id` - Unique identifier for the proposal
    /// * `proposer` - Address of the proposer
    /// * `title` - Title of the proposal
    /// * `voting_deadline` - When voting ends
    pub fn proposal_created(
        proposal_id: u64,
        proposer: &Addr,
        title: &str,
        voting_deadline: Timestamp,
    ) -> Event {
        create_event("governance", "proposal-created")
            .add_attribute("proposal_id", proposal_id.to_string())
            .add_attribute("proposer", proposer.to_string())
            .add_attribute("title", title)
            .add_attribute("voting_deadline", voting_deadline.to_string())
    }

    /// Event emitted when a vote is cast
    /// 
    /// # Arguments
    /// * `proposal_id` - ID of the proposal being voted on
    /// * `voter` - Address of the voter
    /// * `support` - Whether the vote is in support (true) or against (false)
    /// * `voting_power` - Karma-weighted voting power used
    pub fn vote_cast(
        proposal_id: u64,
        voter: &Addr,
        support: bool,
        voting_power: Uint128,
    ) -> Event {
        create_event("governance", "vote-cast")
            .add_attribute("proposal_id", proposal_id.to_string())
            .add_attribute("voter", voter.to_string())
            .add_attribute("support", support.to_string())
            .add_attribute("voting_power", voting_power.to_string())
    }

    /// Event emitted when a proposal is finalized
    /// 
    /// # Arguments
    /// * `proposal_id` - ID of the finalized proposal
    /// * `result` - Result of the proposal (Passed, Failed)
    /// * `votes_for` - Total karma-weighted votes in favor
    /// * `votes_against` - Total karma-weighted votes against
    /// * `execution_result` - Whether execution succeeded (if applicable)
    pub fn proposal_finalized(
        proposal_id: u64,
        result: &str,
        votes_for: Uint128,
        votes_against: Uint128,
        execution_result: Option<bool>,
    ) -> Event {
        let mut event = create_event("governance", "proposal-finalized")
            .add_attribute("proposal_id", proposal_id.to_string())
            .add_attribute("result", result)
            .add_attribute("votes_for", votes_for.to_string())
            .add_attribute("votes_against", votes_against.to_string());

        if let Some(executed) = execution_result {
            event = event.add_attribute("execution_result", executed.to_string());
        }

        event
    }

    /// Event emitted when proposal execution occurs
    /// 
    /// # Arguments
    /// * `proposal_id` - ID of the executed proposal
    /// * `executor` - Address that triggered the execution
    /// * `success` - Whether execution was successful
    pub fn proposal_executed(
        proposal_id: u64,
        executor: &Addr,
        success: bool,
    ) -> Event {
        create_event("governance", "proposal-executed")
            .add_attribute("proposal_id", proposal_id.to_string())
            .add_attribute("executor", executor.to_string())
            .add_attribute("success", success.to_string())
    }
}

/// Oracle Integration Events
pub struct OracleEvents;

impl OracleEvents {
    /// Event emitted when oracle data is submitted
    /// 
    /// # Arguments
    /// * `provider` - Address of the oracle provider
    /// * `data_type` - Type of data submitted
    /// * `data_hash` - Hash of the submitted data
    /// * `timestamp` - When the data was submitted
    pub fn oracle_data_submitted(
        provider: &Addr,
        data_type: &str,
        data_hash: &str,
        timestamp: Timestamp,
    ) -> Event {
        create_event("oracle", "data-submitted")
            .add_attribute("provider", provider.to_string())
            .add_attribute("data_type", data_type)
            .add_attribute("data_hash", data_hash)
            .add_attribute("timestamp", timestamp.to_string())
    }

    /// Event emitted when oracle data reaches consensus
    /// 
    /// # Arguments
    /// * `data_hash` - Hash of the verified data
    /// * `consensus_count` - Number of validators that agreed
    /// * `total_validators` - Total number of validators
    pub fn oracle_consensus_reached(
        data_hash: &str,
        consensus_count: u32,
        total_validators: u32,
    ) -> Event {
        create_event("oracle", "consensus-reached")
            .add_attribute("data_hash", data_hash)
            .add_attribute("consensus_count", consensus_count.to_string())
            .add_attribute("total_validators", total_validators.to_string())
    }

    /// Event emitted when oracle data is disputed
    /// 
    /// # Arguments
    /// * `data_hash` - Hash of the disputed data
    /// * `challenger` - Address of the challenger
    /// * `stake_amount` - Amount of karma staked for the challenge
    pub fn oracle_data_disputed(
        data_hash: &str,
        challenger: &Addr,
        stake_amount: Uint128,
    ) -> Event {
        create_event("oracle", "data-disputed")
            .add_attribute("data_hash", data_hash)
            .add_attribute("challenger", challenger.to_string())
            .add_attribute("stake_amount", stake_amount.to_string())
    }
}

/// System-wide events for monitoring and analytics
pub struct SystemEvents;

impl SystemEvents {
    /// Event emitted for system performance metrics
    /// 
    /// # Arguments
    /// * `metric_name` - Name of the performance metric
    /// * `value` - Value of the metric
    /// * `timestamp` - When the metric was recorded
    pub fn performance_metric(
        metric_name: &str,
        value: &str,
        timestamp: Timestamp,
    ) -> Event {
        create_event("system", "performance-metric")
            .add_attribute("metric_name", metric_name)
            .add_attribute("value", value)
            .add_attribute("timestamp", timestamp.to_string())
    }

    /// Event emitted when system configuration is updated
    /// 
    /// # Arguments
    /// * `config_key` - Configuration parameter that was updated
    /// * `old_value` - Previous value
    /// * `new_value` - New value
    /// * `updater` - Address that made the update
    pub fn config_updated(
        config_key: &str,
        old_value: &str,
        new_value: &str,
        updater: &Addr,
    ) -> Event {
        create_event("system", "config-updated")
            .add_attribute("config_key", config_key)
            .add_attribute("old_value", old_value)
            .add_attribute("new_value", new_value)
            .add_attribute("updater", updater.to_string())
    }
}