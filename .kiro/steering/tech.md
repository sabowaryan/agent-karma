# Technology Stack

Agent-Karma is built on the Sei blockchain with a modern TypeScript-based architecture optimized for sub-400ms performance.

## Blockchain & Smart Contracts

- **Sei Network**: High-performance blockchain with <400ms finality
- **Solidity**: Smart contract development language
- **@sei-js/evm**: EVM interactions and contract deployment
- **@sei-js/cosmjs**: Cosmos SDK operations
- **@sei-js/precompiles**: Native Sei optimizations for performance
- **Hardhat/Foundry**: Development and testing frameworks

## Backend Services

- **Node.js + Express**: REST API gateway
- **TypeScript**: Type-safe development across all services
- **Socket.io**: WebSocket service for real-time updates
- **Redis**: High-performance caching layer
- **PostgreSQL**: Relational database for off-chain data

## Frontend & Dashboard

- **React + TypeScript**: Modern web interface
- **Chart.js**: Karma visualization and analytics
- **WebSocket Client**: Real-time data updates
- **Responsive Design**: Multi-device compatibility

## AI Framework Integrations

- **@sei-js/mcp-server**: Native MCP integration for Sei
- **ElizaOS Plugin**: Native plugin architecture
- **AIDN Connector**: Agent network integration
- **Custom SDK**: Unified interface for all frameworks

## Oracle & External Data

- **Rivalz Oracle**: External data source integration
- **Multi-signature Validation**: 3/5 consensus for data verification
- **Proof-of-Report**: Staking mechanism for oracle providers

## Development Tools

- **@sei-js/create-sei**: Project initialization CLI
- **@sei-js/sei-global-wallet**: Wallet integration
- **@sei-js/ledger**: Hardware wallet support
- **Chai + Mocha**: Testing framework
- **Supertest**: API testing

## Performance Requirements

- **Response Time**: <400ms for all operations
- **Throughput**: 1000+ interactions/second
- **Uptime**: 99.9% availability target
- **Gas Optimization**: Efficient smart contract operations

## Common Commands

### Development Setup
```bash
# Initialize new Sei project
npx @sei-js/create-sei agent-karma

# Install dependencies
npm install @sei-js/evm @sei-js/cosmjs @sei-js/precompiles

# Start development environment
npm run dev
```

### Smart Contract Operations
```bash
# Compile contracts
npx hardhat compile

# Deploy to Sei testnet
npx hardhat deploy --network sei-testnet

# Run contract tests
npx hardhat test

# Verify contracts
npx hardhat verify --network sei-mainnet <contract-address>
```

### Testing & Quality
```bash
# Run all tests
npm test

# Run integration tests
npm run test:integration

# Check test coverage
npm run coverage

# Lint code
npm run lint

# Type checking
npm run type-check
```

### Production Deployment
```bash
# Build production bundle
npm run build

# Deploy to production
npm run deploy:prod

# Monitor performance
npm run monitor

# Check health status
npm run health-check
```

## Architecture Patterns

- **Modular Design**: Separate concerns across smart contracts
- **Event-Driven**: Blockchain events drive off-chain updates
- **Caching Strategy**: Redis for frequently accessed data
- **Error Handling**: Graceful degradation and retry mechanisms
- **Security First**: Access control and input validation throughout