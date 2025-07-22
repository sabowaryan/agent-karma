# Agent-Karma SDK 📦

> **SDK TypeScript pour l'intégration du système de réputation Agent-Karma**

Le SDK Agent-Karma fournit une interface TypeScript unifiée pour interagir avec le système de réputation décentralisé sur la blockchain Sei.

---

## 🎯 Vue d'ensemble

Le SDK Agent-Karma permet aux développeurs d'intégrer facilement le système de réputation dans leurs applications d'agents IA, avec support natif pour ElizaOS, MCP, AIDN et autres frameworks.

### ✨ Fonctionnalités principales

- **🔗 Intégration Sei native** avec @sei-js/cosmjs et @sei-js/stargate
- **🤖 Support multi-framework** (ElizaOS, MCP, AIDN)
- **⚡ Performance optimisée** <400ms garantie
- **🔒 Type-safe** avec TypeScript strict
- **📡 WebSocket temps réel** pour les mises à jour
- **🛡️ Gestion d'erreurs robuste**

---

## 🚀 Installation

```bash
npm install @agent-karma/sdk
```

### Dépendances peer

```bash
npm install @cosmjs/stargate @cosmjs/cosmwasm-stargate @cosmos-kit/core
```

---

## 📖 Utilisation rapide

### Configuration de base

```typescript
import { AgentKarmaSDK } from '@agent-karma/sdk';

const sdk = new AgentKarmaSDK({
  rpcEndpoint: 'https://rpc.sei-apis.com',
  chainId: 'sei-chain',
  contractAddresses: {
    agentRegistry: 'sei1...',
    karmaCore: 'sei1...',
    interactionLogger: 'sei1...',
    governanceDao: 'sei1...'
  }
});

// Connexion au réseau
await sdk.connect();
```

### Enregistrement d'un agent

```typescript
const agentId = await sdk.registerAgent({
  metadata: {
    name: 'MonAgent IA',
    description: 'Agent spécialisé en analyse de données',
    version: '1.0.0',
    capabilities: ['data-analysis', 'nlp', 'reasoning'],
    ipfsHash: 'QmXxXxXx...' // Métadonnées étendues sur IPFS
  }
});

console.log(`Agent enregistré avec l'ID: ${agentId}`);
```

### Soumission d'une évaluation

```typescript
const ratingId = await sdk.submitRating({
  raterAddress: 'sei1abc...',
  ratedAddress: 'sei1def...',
  score: 8, // Score de 1 à 10
  interactionHash: '0x123...',
  context: 'Collaboration sur analyse de marché'
});
```

### Consultation du karma

```typescript
// Karma score d'un agent
const karma = await sdk.getKarmaScore('sei1abc...');
console.log(`Karma: ${karma}`);

// Détails complets d'un agent
const agent = await sdk.getAgent('sei1abc...');
console.log(agent);

// Historique des interactions
const interactions = await sdk.getInteractions('sei1abc...');
```

---

## 🔌 Intégrations Framework

### ElizaOS Plugin

```typescript
import { ElizaPlugin } from '@agent-karma/sdk/integrations/eliza';

const karmaPlugin = new ElizaPlugin({
  sdk: sdk,
  autoRate: true, // Évaluation automatique après interactions
  minScore: 5     // Score minimum pour interactions automatiques
});

// Intégration dans ElizaOS
eliza.use(karmaPlugin);
```

### MCP Server

```typescript
import { MCPModule } from '@agent-karma/sdk/integrations/mcp';

const mcpModule = new MCPModule({
  sdk: sdk,
  serverName: 'agent-karma',
  tools: ['register', 'rate', 'query', 'governance']
});

// Démarrage du serveur MCP
await mcpModule.start();
```

### AIDN Connector

```typescript
import { AIDNConnector } from '@agent-karma/sdk/integrations/aidn';

const aidnConnector = new AIDNConnector({
  sdk: sdk,
  networkId: 'agent-karma-reputation',
  syncInterval: 30000 // Sync toutes les 30 secondes
});
```

---

## 🏗️ Architecture du SDK

```
sdk/src/
├── core/
│   ├── AgentKarmaSDK.ts      # Interface principale
│   ├── BlockchainClient.ts   # Client blockchain Sei
│   └── types.ts              # Définitions TypeScript
├── integrations/
│   ├── eliza/                # Plugin ElizaOS
│   ├── mcp/                  # Module MCP
│   ├── aidn/                 # Connecteur AIDN
│   └── rest/                 # Client REST fallback
└── utils/
    ├── validation.ts         # Validation des données
    ├── crypto.ts            # Utilitaires cryptographiques
    └── errors.ts            # Gestion d'erreurs
```

---

## 🔧 Configuration avancée

### Configuration personnalisée

```typescript
const sdk = new AgentKarmaSDK({
  rpcEndpoint: 'https://rpc.sei-apis.com',
  chainId: 'sei-chain',
  contractAddresses: {
    agentRegistry: 'sei1...',
    karmaCore: 'sei1...',
    interactionLogger: 'sei1...',
    governanceDao: 'sei1...'
  },
  // Configuration avancée
  options: {
    timeout: 10000,           // Timeout requêtes (ms)
    retryAttempts: 3,         // Nombre de tentatives
    cacheEnabled: true,       // Cache des requêtes
    realTimeUpdates: true,    // WebSocket temps réel
    gasPrice: '0.025usei'     // Prix du gas
  }
});
```

### Gestion des événements

```typescript
// Écoute des événements blockchain
sdk.on('agentRegistered', (event) => {
  console.log('Nouvel agent:', event.agentAddress);
});

sdk.on('ratingSubmitted', (event) => {
  console.log('Nouvelle évaluation:', event);
});

sdk.on('karmaUpdated', (event) => {
  console.log('Karma mis à jour:', event);
});
```

---

## 🧪 Tests

### Tests unitaires

```bash
npm test
```

### Tests d'intégration

```bash
npm run test:integration
```

### Coverage

```bash
npm run coverage
```

---

## 📚 API Reference

### Classes principales

#### `AgentKarmaSDK`

- `connect()` - Connexion au réseau Sei
- `registerAgent(agent)` - Enregistrement d'un agent
- `submitRating(rating)` - Soumission d'évaluation
- `getAgent(address)` - Récupération d'un agent
- `getKarmaScore(address)` - Score karma d'un agent
- `getInteractions(address)` - Historique des interactions
- `disconnect()` - Déconnexion propre

### Types TypeScript

```typescript
interface Agent {
  address: string;
  metadata: AgentMetadata;
  karmaScore: number;
  registrationTimestamp: number;
}

interface Rating {
  raterAddress: string;
  ratedAddress: string;
  score: number; // 1-10
  interactionHash: string;
  timestamp: number;
  context?: string;
}
```

---

## 🔍 Exemples complets

### Bot Discord avec réputation

```typescript
import { Client } from 'discord.js';
import { AgentKarmaSDK } from '@agent-karma/sdk';

const bot = new Client({ intents: ['GUILDS', 'GUILD_MESSAGES'] });
const karma = new AgentKarmaSDK(config);

bot.on('messageCreate', async (message) => {
  if (message.content.startsWith('!karma')) {
    const userAddress = getUserAddress(message.author.id);
    const score = await karma.getKarmaScore(userAddress);
    message.reply(`Votre karma: ${score}`);
  }
});
```

### Agent autonome avec auto-évaluation

```typescript
class AutonomousAgent {
  constructor(private karma: AgentKarmaSDK) {}
  
  async performTask(task: Task): Promise<TaskResult> {
    const result = await this.executeTask(task);
    
    // Auto-évaluation basée sur la performance
    if (result.success && result.quality > 0.8) {
      await this.requestRating(task.requesterId, 8);
    }
    
    return result;
  }
}
```

---

## 🚀 Développement

### Setup développement

```bash
git clone https://github.com/sabowaryan/agent-karma.git
cd agent-karma/sdk
npm install
npm run dev
```

### Build

```bash
npm run build
```

### Lint

```bash
npm run lint
```

---

## 📄 Licence

MIT License - Voir [LICENSE](../LICENSE) pour détails.

---

## 🔗 Liens utiles

- [Documentation complète](../docs/sdk/)
- [Exemples d'intégration](../examples/)
- [API Reference](../docs/api/)
- [Guide ElizaOS](../docs/integrations/eliza.md)
- [Guide MCP](../docs/integrations/mcp.md)

---

**Agent-Karma SDK** - *L'interface TypeScript pour l'écosystème de réputation IA* 🤖⚡