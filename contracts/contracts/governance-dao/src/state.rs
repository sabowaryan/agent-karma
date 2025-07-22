use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use agent_karma_contracts::{Proposal, Vote};
use crate::msg::GovernanceConfig;

/// Configuration for the governance contract
pub const CONFIG: Item<GovernanceConfig> = Item::new("config");

/// Counter for proposal IDs
pub const PROPOSAL_COUNT: Item<u64> = Item::new("proposal_count");

/// Storage for proposals by ID
pub const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");

/// Storage for votes by (proposal_id, voter_address)
pub const VOTES: Map<(u64, &Addr), Vote> = Map::new("votes");

/// Storage for proposal vote tallies by proposal_id
pub const VOTE_TALLIES: Map<u64, VoteTally> = Map::new("vote_tallies");

/// Vote tally information for a proposal
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct VoteTally {
    pub votes_for: Uint128,
    pub votes_against: Uint128,
    pub total_voting_power: Uint128,
    pub unique_voters: u32,
}