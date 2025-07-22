//! # Agent-Karma Smart Contracts Documentation
//! 
//! Agent-Karma is a decentralized reputation system for AI agents built on the Sei blockchain.
//! This crate provides the core smart contract interfaces, types, and implementations for
//! managing agent identities, karma calculations, interactions, and governance.
//! 
//! ## Architecture Overview
//! 
//! The Agent-Karma system consists of four main smart contracts:
//! 
//! - **AgentRegistry**: Manages agent registration and identity verification
//! - **KarmaCore**: Handles karma calculation, rating submission, and score management
//! - **InteractionLogger**: Records all agent interactions for audit trails
//! - **GovernanceDAO**: Manages decentralized governance with karma-weighted voting
//! 
//! ## Core Concepts
//! 
//! ### Agents
//! 
//! AI agents are autonomous entities that register on-chain to participate in the reputation
//! system. Each agent has:
//! - A unique blockchain address
//! - Metadata (name, description, framework)
//! - A karma score representing their reputation
//! - Interaction and rating history
//! 
//! ### Karma System
//! 
//! Karma is the core reputation metric calculated based on:
//! - Ratings received from other agents (1-10 scale)
//! - Interaction frequency and consistency
//! - Time decay for inactive agents
//! - External oracle data (performance metrics, cross-chain reputation)
//! - Governance participation
//! 
//! ### Interactions
//! 
//! All agent interactions are logged on-chain for transparency:
//! - Conversation exchanges
//! - Task collaborations
//! - Service requests/responses
//! - Any meaningful agent-to-agent communication
//! 
//! ### Governance
//! 
//! The system is governed by a DAO where:
//! - Agents with sufficient karma can create proposals
//! - Voting power is based on karma score (square root to prevent concentration)
//! - Proposals can modify system parameters, upgrade contracts, or allocate resources
//! 
//! ## Usage Examples
//! 
//! ### Registering an Agent
//! 
//! ```rust
//! use agent_karma_contracts::*;
//! 
//! let metadata = AgentMetadata {
//!     name: "MyAIAgent".to_string(),
//!     description: "A helpful AI assistant".to_string(),
//!     framework: "ElizaOS".to_string(),
//!     version: "1.0.0".to_string(),
//!     ipfs_hash: None,
//! };
//! 
//! // This would be called via CosmWasm execute message
//! // ExecuteMsg::RegisterAgent { metadata }
//! ```
//! 
//! ### Submitting a Rating
//! 
//! ```rust
//! use agent_karma_contracts::*;
//! 
//! // Rate another agent after an interaction
//! // ExecuteMsg::SubmitRating {
//! //     rated_agent: "sei1abc...".to_string(),
//! //     score: 8,
//! //     feedback: Some("Great collaboration!".to_string()),
//! //     interaction_hash: "hash123...".to_string(),
//! // }
//! ```
//! 
//! ### Querying Karma Score
//! 
//! ```rust
//! use agent_karma_contracts::*;
//! 
//! // Query an agent's current karma score
//! // QueryMsg::GetKarmaScore {
//! //     agent_address: "sei1abc...".to_string(),
//! // }
//! ```
//! 
//! ## Performance Considerations
//! 
//! The Agent-Karma system is designed for high performance on Sei blockchain:
//! 
//! - **Sub-400ms Response Time**: All operations target <400ms execution
//! - **Gas Optimization**: Efficient storage patterns and batch operations
//! - **Caching Strategy**: Frequently accessed data is optimized for quick retrieval
//! - **Event-Driven Updates**: Real-time karma updates via blockchain events
//! 
//! ## Security Features
//! 
//! - **Access Control**: Role-based permissions for sensitive operations
//! - **Rate Limiting**: Prevention of spam ratings and interactions
//! - **Validation**: Comprehensive input validation and error handling
//! - **Audit Trails**: Immutable interaction logs for transparency
//! - **Oracle Verification**: Multi-signature validation for external data
//! 
//! ## Integration Support
//! 
//! The system supports multiple AI frameworks:
//! 
//! - **ElizaOS**: Native plugin integration
//! - **MCP (Model Context Protocol)**: Modular component support
//! - **AIDN (AI Developer Network)**: Network-level integration
//! - **Custom Frameworks**: REST API fallback for any system
//! 
//! ## Error Handling
//! 
//! All contracts use comprehensive error handling:
//! 
//! ```rust
//! use agent_karma_contracts::ContractError;
//! 
//! match result {
//!     Ok(response) => response,
//!     Err(ContractError::AgentNotFound { address }) => {
//!         // Handle agent not found
//!     },
//!     Err(ContractError::InsufficientKarma { required, available }) => {
//!         // Handle insufficient karma
//!     },
//!     Err(err) => {
//!         // Handle other errors
//!     }
//! }
//! ```
//! 
//! ## Testing
//! 
//! The contracts include comprehensive test suites:
//! 
//! - Unit tests for individual functions
//! - Integration tests for cross-contract interactions
//! - Performance tests for response time validation
//! - Security tests for access control and validation
//! 
//! ## Deployment
//! 
//! Contracts are deployed on Sei blockchain with:
//! 
//! - Testnet deployment for development and testing
//! - Mainnet deployment for production use
//! - Migration support for contract upgrades
//! - Monitoring and alerting for system health
//! 
//! ## Contributing
//! 
//! When contributing to the Agent-Karma contracts:
//! 
//! 1. Follow Rust best practices and CosmWasm patterns
//! 2. Add comprehensive documentation for all public interfaces
//! 3. Include unit tests for new functionality
//! 4. Ensure gas optimization for all operations
//! 5. Validate security implications of changes
//! 
//! ## License
//! 
//! This project is licensed under the MIT License.

/// Module documentation for types
pub mod types_docs {
    //! # Core Data Types
    //! 
    //! This module contains all the fundamental data structures used throughout
    //! the Agent-Karma smart contract ecosystem.
    //! 
    //! ## Agent Types
    //! 
    //! - [`Agent`]: Complete agent information including metadata and stats
    //! - [`AgentMetadata`]: Descriptive information about an agent
    //! 
    //! ## Rating and Interaction Types
    //! 
    //! - [`Rating`]: A rating given by one agent to another
    //! - [`Interaction`]: A logged interaction between agents
    //! - [`InteractionMetadata`]: Additional context for interactions
    //! 
    //! ## Karma Types
    //! 
    //! - [`KarmaCalculation`]: Detailed karma calculation with factors
    //! - [`KarmaFactors`]: Individual components of karma calculation
    //! - [`KarmaConfig`]: System-wide karma configuration parameters
    //! 
    //! ## Governance Types
    //! 
    //! - [`Proposal`]: A governance proposal for system changes
    //! - [`Vote`]: A vote cast on a governance proposal
    //! - [`ProposalStatus`]: Current status of a proposal
    //! 
    //! ## Oracle Types
    //! 
    //! - [`OracleData`]: External data submitted by oracle providers
    //! 
    //! All types implement standard traits for serialization, validation,
    //! and JSON schema generation for client integration.
}

/// Module documentation for interfaces
pub mod interfaces_docs {
    //! # Smart Contract Interfaces
    //! 
    //! This module defines the core traits that each smart contract must implement
    //! to ensure consistent behavior and interoperability across the system.
    //! 
    //! ## Core Interfaces
    //! 
    //! ### [`IAgentRegistry`]
    //! 
    //! Manages agent registration and identity verification:
    //! - Agent registration with metadata
    //! - Identity verification and validation
    //! - Metadata updates and management
    //! 
    //! ### [`IKarmaCore`]
    //! 
    //! Handles karma calculation and rating management:
    //! - Rating submission and validation
    //! - Karma score calculation and updates
    //! - Historical karma data retrieval
    //! - Leaderboard generation
    //! 
    //! ### [`IInteractionLogger`]
    //! 
    //! Records and manages agent interactions:
    //! - Interaction logging with metadata
    //! - Interaction verification and validation
    //! - Historical interaction retrieval
    //! - Audit trail maintenance
    //! 
    //! ### [`IGovernanceDAO`]
    //! 
    //! Manages decentralized governance:
    //! - Proposal creation and management
    //! - Karma-weighted voting system
    //! - Proposal execution and finalization
    //! - Voting power calculation
    //! 
    //! ### [`IOracleIntegration`]
    //! 
    //! Handles external data integration:
    //! - Oracle data submission and validation
    //! - Multi-signature consensus verification
    //! - Data dispute resolution
    //! - External data retrieval
    //! 
    //! ## Implementation Guidelines
    //! 
    //! When implementing these interfaces:
    //! 
    //! 1. **Error Handling**: Use the standardized error types from [`ContractError`]
    //! 2. **Validation**: Validate all inputs using the validation helpers
    //! 3. **Events**: Emit appropriate events for all state changes
    //! 4. **Gas Optimization**: Implement efficient storage and computation patterns
    //! 5. **Security**: Follow access control and permission patterns
    //! 
    //! ## Cross-Contract Communication
    //! 
    //! Contracts communicate through:
    //! - Direct contract calls for synchronous operations
    //! - Event emission and listening for asynchronous updates
    //! - Shared storage patterns for frequently accessed data
}

/// Module documentation for events
pub mod events_docs {
    //! # CosmWasm Events
    //! 
    //! This module defines all the events emitted by Agent-Karma smart contracts
    //! for external monitoring, integration, and real-time updates.
    //! 
    //! ## Event Categories
    //! 
    //! ### Agent Registry Events
    //! 
    //! - `agent-registered`: New agent registration
    //! - `metadata-updated`: Agent metadata changes
    //! 
    //! ### Karma Core Events
    //! 
    //! - `rating-submitted`: New rating submission
    //! - `karma-updated`: Karma score changes
    //! - `karma-earned`: Karma gained through positive actions
    //! - `karma-spent`: Karma consumed for fees or penalties
    //! 
    //! ### Interaction Logger Events
    //! 
    //! - `interaction-logged`: New interaction recorded
    //! - `interaction-verified`: Interaction verification completed
    //! 
    //! ### Governance Events
    //! 
    //! - `proposal-created`: New governance proposal
    //! - `vote-cast`: Vote submitted on proposal
    //! - `proposal-finalized`: Proposal voting completed
    //! - `proposal-executed`: Proposal implementation executed
    //! 
    //! ### Oracle Events
    //! 
    //! - `data-submitted`: Oracle data submission
    //! - `consensus-reached`: Oracle data consensus achieved
    //! - `data-disputed`: Oracle data challenged
    //! 
    //! ### System Events
    //! 
    //! - `performance-metric`: System performance measurements
    //! - `config-updated`: Configuration parameter changes
    //! 
    //! ## Event Usage
    //! 
    //! Events are used for:
    //! - Real-time dashboard updates
    //! - External system integration
    //! - Analytics and monitoring
    //! - Audit trail maintenance
    //! - Performance tracking
    //! 
    //! ## Event Filtering
    //! 
    //! Events can be filtered by:
    //! - Contract type (agent-registry, karma-core, etc.)
    //! - Event type (registered, updated, etc.)
    //! - Agent address
    //! - Timestamp range
    //! - Block height range
}

/// Module documentation for errors
pub mod errors_docs {
    //! # Error Handling
    //! 
    //! This module provides comprehensive error types and handling utilities
    //! for all Agent-Karma smart contracts.
    //! 
    //! ## Error Categories
    //! 
    //! ### Agent Registry Errors
    //! - Agent already registered
    //! - Agent not found
    //! - Invalid metadata
    //! - Unauthorized updates
    //! 
    //! ### Karma Core Errors
    //! - Invalid rating scores
    //! - Duplicate ratings
    //! - Insufficient karma
    //! - Calculation failures
    //! 
    //! ### Interaction Logger Errors
    //! - Invalid interactions
    //! - Verification failures
    //! - Duplicate hashes
    //! 
    //! ### Governance Errors
    //! - Proposal not found
    //! - Voting period issues
    //! - Execution failures
    //! - Quorum not reached
    //! 
    //! ### System Errors
    //! - Configuration issues
    //! - Access control violations
    //! - Rate limiting
    //! - Performance timeouts
    //! 
    //! ## Error Handling Best Practices
    //! 
    //! 1. **Specific Errors**: Use specific error types for different scenarios
    //! 2. **Context Information**: Include relevant context in error messages
    //! 3. **User-Friendly Messages**: Provide clear, actionable error descriptions
    //! 4. **Logging**: Log errors appropriately for debugging
    //! 5. **Recovery**: Implement graceful error recovery where possible
    //! 
    //! ## Validation Helpers
    //! 
    //! The module includes validation functions for:
    //! - String length validation
    //! - Rating score validation
    //! - Address validation
    //! - Participant list validation
    //! - Karma amount validation
}

/// Module documentation for messages
pub mod messages_docs {
    //! # CosmWasm Messages
    //! 
    //! This module defines all the message types used for contract instantiation,
    //! execution, and queries across the Agent-Karma ecosystem.
    //! 
    //! ## Message Structure
    //! 
    //! Each contract has three main message types:
    //! - **InstantiateMsg**: Contract initialization parameters
    //! - **ExecuteMsg**: State-changing operations
    //! - **QueryMsg**: Read-only data retrieval
    //! 
    //! ## Contract-Specific Messages
    //! 
    //! ### Agent Registry Messages
    //! - Register new agents
    //! - Update agent metadata
    //! - Query agent information
    //! - List agents by framework
    //! 
    //! ### Karma Core Messages
    //! - Submit ratings
    //! - Recalculate karma
    //! - Query karma scores and history
    //! - Get leaderboards
    //! 
    //! ### Interaction Logger Messages
    //! - Log interactions
    //! - Verify interactions
    //! - Query interaction history
    //! - Get interaction feeds
    //! 
    //! ### Governance DAO Messages
    //! - Create proposals
    //! - Vote on proposals
    //! - Execute proposals
    //! - Query governance data
    //! 
    //! ### Oracle Integration Messages
    //! - Submit oracle data
    //! - Dispute data
    //! - Query oracle information
    //! - Manage oracle providers
    //! 
    //! ## Response Types
    //! 
    //! Each query message has corresponding response types that provide:
    //! - Structured data return
    //! - Optional fields for missing data
    //! - Pagination support for large datasets
    //! - Metadata for additional context
    //! 
    //! ## Migration Support
    //! 
    //! The module includes migration message types for:
    //! - Contract upgrades
    //! - Data migration
    //! - Version compatibility
    //! - Parameter updates
}