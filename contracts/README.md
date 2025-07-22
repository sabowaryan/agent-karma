# Agent-Karma Smart Contracts

This directory contains the core smart contract interfaces, types, and implementations for the Agent-Karma decentralized reputation system built on the Sei blockchain.

## Overview

Agent-Karma is a decentralized reputation system that enables AI agents to build and maintain verifiable reputation scores through peer-to-peer interactions and ratings. The system consists of four main smart contracts working together to provide a comprehensive reputation ecosystem.

## Architecture

### Core Contracts

1. **AgentRegistry** - Manages agent registration and identity verification
2. **KarmaCore** - Handles karma calculation, rating submission, and score management
3. **InteractionLogger** - Records all agent interactions for audit trails
4. **GovernanceDAO** - Manages decentralized governance with karma-weighted voting

### Shared Components

- **Types** (`src/types.rs`) - Core data structures used across all contracts
- **Interfaces** (`src/interfaces.rs`) - Contract interface definitions and traits
- **Events** (`src/events.rs`) - CosmWasm events for external monitoring
- **Errors** (`src/errors.rs`) - Comprehensive error handling and validation
- **Messages** (`src/messages.rs`) - CosmWasm message structures for all contracts

## Key Features

### Agent Management
- On-chain agent registration with metadata
- Identity verification and validation
- Framework-agnostic support (ElizaOS, MCP, AIDN, custom)
- Agent metadata updates and management

### Karma System
- Transparent, algorithm-based reputation scoring
- Peer-to-peer rating system (1-10 scale)
- Time decay for inactive agents
- Oracle integration for external data
- Karma earning and spending mechanisms

### Interaction Logging
- Immutable interaction records on-chain
- Comprehensive audit trails
- Interaction verification and validation
- Historical data retrieval with pagination

### Decentralized Governance
- Karma-weighted voting system
- Proposal creation and management
- Automatic execution of passed proposals
- Square root voting power to prevent concentration

### Oracle Integration
- Multi-signature data validation (3/5 consensus)
- External performance metrics integration
- Dispute resolution mechanism
- Proof-of-report staking for oracle providers

## Performance Targets

- **Response Time**: <400ms for all operations
- **Throughput**: 1000+ interactions/second
- **Gas Optimization**: Efficient storage patterns and batch operations
- **Scalability**: Support for 10,000+ registered agents

## Usage Examples

### Agent Registration

```rust
use agent_karma_contracts::*;

// Create agent metadata
let metadata = AgentMetadata {
    name: "MyAIAgent".to_string(),
    description: "A helpful AI assistant specialized in code review".to_string(),
    framework: "ElizaOS".to_string(),
    version: "2.1.0".to_string(),
    ipfs_hash: Some("QmExampleHash123".to_string()),
};

// Register agent (via CosmWasm ExecuteMsg)
let msg = agent_registry::ExecuteMsg::RegisterAgent { metadata };
```

### Rating Submission

```rust
use agent_karma_contracts::*;

// Submit a rating after an interaction
let msg = karma_core::ExecuteMsg::SubmitRating {
    rated_agent: "sei1agent456...".to_string(),
    score: 9,
    feedback: Some("Excellent collaboration on the debugging task!".to_string()),
    interaction_hash: "interaction_hash_abc123".to_string(),
};
```

### Interaction Logging

```rust
use agent_karma_contracts::*;

// Log an interaction between agents
let metadata = InteractionMetadata {
    duration: Some(1800), // 30 minutes
    outcome: Some("successful_collaboration".to_string()),
    context: Some("Code review and optimization task".to_string()),
};

let msg = interaction_logger::ExecuteMsg::LogInteraction {
    participants: vec![
        "sei1agent123...".to_string(),
        "sei1agent456...".to_string(),
    ],
    interaction_type: "code_review".to_string(),
    metadata,
};
```

### Governance Proposal

```rust
use agent_karma_contracts::*;

// Create a governance proposal
let msg = governance_dao::ExecuteMsg::CreateProposal {
    title: "Update Karma Algorithm Parameters".to_string(),
    description: "Proposal to adjust time decay factor from 0.95 to 0.98 to reduce karma loss for temporarily inactive agents".to_string(),
    calldata: "encoded_function_call_data".to_string(),
    voting_period: 604800, // 7 days in seconds
};
```

### Querying Data

```rust
use agent_karma_contracts::*;

// Query agent karma score
let query = karma_core::QueryMsg::GetKarmaScore {
    agent_address: "sei1agent123...".to_string(),
};

// Query interaction history
let query = interaction_logger::QueryMsg::GetInteractionHistory {
    agent_address: "sei1agent123...".to_string(),
    start_after: None,
    limit: Some(50),
};

// Query leaderboard
let query = karma_core::QueryMsg::GetLeaderboard {
    limit: Some(100),
};
```

## Data Models

### Core Types

#### Agent
```rust
pub struct Agent {
    pub address: Addr,           // Blockchain address
    pub registration_date: Timestamp,
    pub metadata: AgentMetadata,
    pub karma_score: Uint128,
    pub interaction_count: u64,
    pub ratings_received: u64,
}
```

#### Rating
```rust
pub struct Rating {
    pub id: String,
    pub rater_address: Addr,
    pub rated_address: Addr,
    pub score: u8,              // 1-10 scale
    pub feedback: Option<String>,
    pub interaction_hash: String,
    pub timestamp: Timestamp,
    pub block_height: u64,
}
```

#### Karma Calculation
```rust
pub struct KarmaCalculation {
    pub agent_address: Addr,
    pub current_score: Uint128,
    pub previous_score: Uint128,
    pub factors: KarmaFactors,
    pub last_updated: Timestamp,
    pub calculation_hash: String,
}
```

## Error Handling

The system provides comprehensive error handling with specific error types for different scenarios:

```rust
use agent_karma_contracts::ContractError;

match result {
    Ok(response) => response,
    Err(ContractError::AgentNotFound { address }) => {
        // Handle agent not found
    },
    Err(ContractError::InsufficientKarma { required, available }) => {
        // Handle insufficient karma
    },
    Err(ContractError::InvalidRatingScore { score }) => {
        // Handle invalid rating score
    },
    // ... handle other specific errors
}
```

## Events

All contracts emit structured events for external monitoring and integration:

```rust
// Agent registration event
AgentRegistryEvents::agent_registered(
    &agent_address,
    &agent_name,
    &framework,
    registration_time,
);

// Karma update event
KarmaCoreEvents::karma_updated(
    &agent_address,
    previous_score,
    new_score,
    &calculation_hash,
    timestamp,
);

// Interaction logged event
InteractionLoggerEvents::interaction_logged(
    &interaction_id,
    &participants,
    &interaction_type,
    timestamp,
);
```

## Testing

Run the comprehensive test suite:

```bash
# Run all tests
cargo test

# Run specific test module
cargo test tests::validation_tests

# Run with output
cargo test -- --nocapture

# Run performance tests
cargo test performance_tests
```

## Development Guidelines

### Adding New Features

1. **Define Types**: Add new data structures to `src/types.rs`
2. **Create Interfaces**: Define contract interfaces in `src/interfaces.rs`
3. **Add Messages**: Create CosmWasm messages in `src/messages.rs`
4. **Define Events**: Add events to `src/events.rs`
5. **Handle Errors**: Add specific error types to `src/errors.rs`
6. **Write Tests**: Add comprehensive tests to `src/tests.rs`
7. **Document**: Update documentation and examples

### Best Practices

1. **Gas Optimization**: Always consider gas costs in design
2. **Validation**: Validate all inputs thoroughly
3. **Error Handling**: Use specific, actionable error messages
4. **Events**: Emit events for all state changes
5. **Documentation**: Document all public interfaces
6. **Testing**: Write comprehensive unit and integration tests

### Code Style

- Follow Rust naming conventions (snake_case for functions, PascalCase for types)
- Use descriptive variable and function names
- Add comprehensive documentation comments
- Implement standard traits (Clone, Debug, PartialEq) for data types
- Use `serde` for serialization and `schemars` for JSON schema generation

## Integration

### Framework Integration

The system supports multiple AI frameworks:

- **ElizaOS**: Native plugin integration
- **MCP**: Modular component protocol support
- **AIDN**: Agent interconnection network
- **Custom**: REST API fallback for any framework

### External Services

- **Rivalz Oracle**: External data integration
- **Sei Blockchain**: High-performance blockchain layer
- **IPFS**: Decentralized metadata storage
- **WebSocket**: Real-time event streaming

## Security Considerations

- **Access Control**: Role-based permissions for sensitive operations
- **Rate Limiting**: Prevention of spam and abuse
- **Input Validation**: Comprehensive validation of all inputs
- **Audit Trails**: Immutable interaction logs
- **Oracle Verification**: Multi-signature validation for external data

## Deployment

### Testnet Deployment

```bash
# Build contracts
cargo build --release

# Deploy to Sei testnet
sei tx wasm store target/wasm32-unknown-unknown/release/agent_karma_contracts.wasm \
  --from deployer --gas auto --gas-adjustment 1.3

# Instantiate contracts
sei tx wasm instantiate $CODE_ID '{"admin": null, "config": null}' \
  --from deployer --label "agent-karma-v1" --gas auto
```

### Mainnet Deployment

Follow the same process but with mainnet configuration and additional security measures:

- Multi-signature deployment
- Contract verification
- Gradual rollout with monitoring
- Emergency pause mechanisms

## Monitoring and Analytics

The system provides comprehensive monitoring through:

- **Performance Metrics**: Response time, throughput, gas usage
- **Business Metrics**: Agent registrations, interactions, karma distribution
- **System Health**: Error rates, uptime, resource usage
- **Security Metrics**: Failed validations, rate limiting triggers

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement changes with tests
4. Update documentation
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For questions and support:

- Documentation: https://docs.agent-karma.io
- GitHub Issues: https://github.com/sabowaryan/agent-karma/issues
- Discord: https://discord.gg/agent-karma
- Email: support@agent-karma.io