# Project Structure

Agent-Karma follows a monorepo structure with clear separation between blockchain, backend, frontend, and integration components.

## Root Directory Structure

```
agent-karma/
├── contracts/              # Smart contracts (Solidity)
├── sdk/                    # TypeScript SDK and integrations
├── api/                    # REST API and WebSocket services
├── dashboard/              # React frontend dashboard
├── docs/                   # Documentation and guides
├── scripts/                # Deployment and utility scripts
├── tests/                  # Integration and E2E tests
└── .kiro/                  # Kiro configuration and specs
```

## Smart Contracts (`/contracts`)

```
contracts/
├── src/
│   ├── core/
│   │   ├── AgentRegistry.sol       # Agent identity management
│   │   ├── KarmaCore.sol          # Reputation calculation engine
│   │   ├── InteractionLogger.sol  # Audit trail logging
│   │   └── GovernanceDAO.sol      # Decentralized governance
│   ├── oracles/
│   │   └── RivalzIntegration.sol  # External data integration
│   ├── interfaces/
│   │   ├── IAgentRegistry.sol
│   │   ├── IKarmaCore.sol
│   │   └── IGovernanceDAO.sol
│   └── libraries/
│       ├── KarmaCalculation.sol   # Karma algorithm library
│       └── SafeOperations.sol     # Security utilities
├── test/                          # Contract unit tests
├── deploy/                        # Deployment scripts
└── hardhat.config.ts             # Hardhat configuration
```

## SDK and Integrations (`/sdk`)

```
sdk/
├── src/
│   ├── core/
│   │   ├── AgentKarmaSDK.ts      # Main SDK interface
│   │   ├── BlockchainClient.ts   # Sei blockchain interactions
│   │   └── types.ts              # TypeScript type definitions
│   ├── integrations/
│   │   ├── eliza/
│   │   │   ├── ElizaPlugin.ts    # ElizaOS plugin
│   │   │   └── index.ts
│   │   ├── mcp/
│   │   │   ├── MCPModule.ts      # MCP integration
│   │   │   └── server.ts         # @sei-js/mcp-server setup
│   │   ├── aidn/
│   │   │   ├── AIDNConnector.ts  # AIDN network integration
│   │   │   └── index.ts
│   │   └── rest/
│   │       └── RestClient.ts     # Fallback REST client
│   └── utils/
│       ├── validation.ts         # Input validation utilities
│       ├── crypto.ts            # Cryptographic helpers
│       └── errors.ts            # Error handling
├── test/                        # SDK unit tests
└── package.json
```

## API Services (`/api`)

```
api/
├── src/
│   ├── routes/
│   │   ├── agents.ts            # Agent management endpoints
│   │   ├── karma.ts             # Karma query endpoints
│   │   ├── ratings.ts           # Rating submission endpoints
│   │   ├── governance.ts        # DAO governance endpoints
│   │   └── interactions.ts      # Interaction history endpoints
│   ├── middleware/
│   │   ├── auth.ts              # JWT authentication
│   │   ├── validation.ts        # Request validation
│   │   ├── rateLimit.ts         # Rate limiting
│   │   └── cors.ts              # CORS configuration
│   ├── services/
│   │   ├── BlockchainService.ts # Smart contract interactions
│   │   ├── CacheService.ts      # Redis caching
│   │   ├── DatabaseService.ts   # PostgreSQL operations
│   │   └── WebSocketService.ts  # Real-time updates
│   ├── models/
│   │   ├── Agent.ts             # Agent data model
│   │   ├── Rating.ts            # Rating data model
│   │   └── Interaction.ts       # Interaction data model
│   └── utils/
│       ├── logger.ts            # Logging utilities
│       └── config.ts            # Configuration management
├── test/                        # API integration tests
└── package.json
```

## Dashboard Frontend (`/dashboard`)

```
dashboard/
├── src/
│   ├── components/
│   │   ├── common/
│   │   │   ├── Header.tsx       # Navigation header
│   │   │   ├── Sidebar.tsx      # Navigation sidebar
│   │   │   └── Loading.tsx      # Loading components
│   │   ├── agents/
│   │   │   ├── AgentCard.tsx    # Agent display card
│   │   │   ├── AgentList.tsx    # Agent listing
│   │   │   └── AgentDetail.tsx  # Detailed agent view
│   │   ├── karma/
│   │   │   ├── KarmaChart.tsx   # Karma visualization
│   │   │   └── KarmaTrends.tsx  # Historical trends
│   │   └── governance/
│   │       ├── ProposalList.tsx # Governance proposals
│   │       └── VotingPanel.tsx  # Voting interface
│   ├── pages/
│   │   ├── Dashboard.tsx        # Main dashboard
│   │   ├── Agents.tsx           # Agent explorer
│   │   ├── Governance.tsx       # DAO governance
│   │   └── Analytics.tsx        # System analytics
│   ├── hooks/
│   │   ├── useWebSocket.ts      # WebSocket connection
│   │   ├── useKarma.ts          # Karma data fetching
│   │   └── useAgents.ts         # Agent data management
│   ├── services/
│   │   ├── api.ts               # API client
│   │   └── websocket.ts         # WebSocket client
│   └── utils/
│       ├── formatters.ts        # Data formatting
│       └── constants.ts         # Application constants
├── public/                      # Static assets
├── test/                        # Frontend unit tests
└── package.json
```

## Documentation (`/docs`)

```
docs/
├── api/                         # API documentation
│   ├── openapi.yaml            # OpenAPI specification
│   └── endpoints.md            # Endpoint documentation
├── contracts/                   # Smart contract docs
│   ├── AgentRegistry.md        # Registry contract guide
│   ├── KarmaCore.md           # Karma calculation guide
│   └── GovernanceDAO.md       # Governance guide
├── integrations/               # Framework integration guides
│   ├── eliza-plugin.md        # ElizaOS integration
│   ├── mcp-module.md          # MCP integration
│   └── aidn-connector.md      # AIDN integration
├── deployment/                 # Deployment guides
│   ├── testnet.md             # Testnet deployment
│   └── mainnet.md             # Production deployment
└── user-guides/               # End-user documentation
    ├── getting-started.md     # Quick start guide
    └── governance.md          # Governance participation
```

## Configuration Files

- **package.json**: Root package configuration with workspaces
- **tsconfig.json**: TypeScript configuration for monorepo
- **hardhat.config.ts**: Blockchain development configuration
- **.env.example**: Environment variable template
- **docker-compose.yml**: Local development environment
- **.github/workflows/**: CI/CD pipeline configuration

## Naming Conventions

### Files and Directories
- **PascalCase**: React components, TypeScript classes (`AgentCard.tsx`, `KarmaCore.sol`)
- **camelCase**: Functions, variables, non-component files (`karmaCalculation.ts`)
- **kebab-case**: Directory names, configuration files (`smart-contracts/`, `hardhat.config.ts`)

### Smart Contracts
- **Contracts**: PascalCase with descriptive names (`AgentRegistry`, `KarmaCore`)
- **Functions**: camelCase with clear action verbs (`registerAgent`, `calculateKarma`)
- **Events**: PascalCase with past tense (`AgentRegistered`, `KarmaUpdated`)
- **Variables**: camelCase with descriptive names (`karmaScore`, `lastInteraction`)

### API Endpoints
- **REST Routes**: kebab-case with resource-based naming (`/agents`, `/karma-scores`)
- **Query Parameters**: camelCase (`agentAddress`, `includeHistory`)
- **Response Fields**: camelCase matching TypeScript conventions

## Import/Export Patterns

### Barrel Exports
```typescript
// src/index.ts - Main SDK export
export { AgentKarmaSDK } from './core/AgentKarmaSDK';
export { ElizaPlugin } from './integrations/eliza';
export { MCPModule } from './integrations/mcp';
export * from './types';
```

### Relative Imports
```typescript
// Use relative imports within same module
import { KarmaCalculation } from '../utils/karmaCalculation';
import { Agent } from './types';

// Use absolute imports for cross-module dependencies
import { AgentKarmaSDK } from '@agent-karma/sdk';
```

## Testing Structure

- **Unit Tests**: Co-located with source files (`*.test.ts`)
- **Integration Tests**: Separate `/tests` directory
- **E2E Tests**: `/tests/e2e` for full workflow testing
- **Contract Tests**: `/contracts/test` for smart contract testing

## Environment Configuration

- **Development**: Local blockchain, test databases
- **Staging**: Sei testnet, staging services
- **Production**: Sei mainnet, production infrastructure

This structure ensures clear separation of concerns, maintainable code organization, and efficient development workflows.