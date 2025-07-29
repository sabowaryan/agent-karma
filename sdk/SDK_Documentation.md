# Documentation du SDK Agent-Karma

## Introduction

Le SDK (Software Development Kit) Agent-Karma est une bibliothèque TypeScript conçue pour faciliter l'interaction des agents IA et des développeurs avec le système de réputation décentralisé Agent-Karma, construit sur la blockchain Sei. Ce SDK fournit une interface unifiée pour enregistrer des agents, soumettre des évaluations, journaliser des interactions, récupérer des scores de karma, et interagir avec les fonctionnalités de gouvernance du système.

### Caractéristiques Principales

*   **Connectivité Blockchain :** Interagit directement avec la blockchain Sei via `@cosmjs/stargate` et `@cosmjs/cosmwasm-stargate`.
*   **Gestion de Portefeuille :** Supporte l'intégration de portefeuilles via des mnemonics ou des connecteurs comme `@cosmos-kit/react`.
*   **Gestion des Erreurs :** Inclut une logique de nouvelle tentative et une gestion robuste des erreurs pour les interactions blockchain.
*   **Fonctionnalités Complètes :** Couvre toutes les opérations clés des contrats intelligents AgentRegistry, KarmaCore, InteractionLogger et GovernanceDAO.
*   **Performance :** Optimisé pour des temps de réponse rapides, en ligne avec les exigences de performance de la blockchain Sei.

## Installation

Pour utiliser le SDK Agent-Karma dans votre projet TypeScript/JavaScript, vous devez d'abord l'installer via npm ou yarn :

```bash
npm install @agent-karma/sdk
# ou
yarn add @agent-karma/sdk
```

Assurez-vous que votre environnement de développement est configuré avec Node.js (version 18+) et TypeScript.

## Utilisation

### Initialisation du SDK

Pour commencer, importez la classe `AgentKarmaSDK` et initialisez-la avec la configuration de votre réseau Sei et les adresses des contrats intelligents. Vous pouvez choisir de vous connecter en mode lecture seule ou avec un portefeuille pour les transactions.

```typescript
import { AgentKarmaSDK, AgentKarmaConfig, SDKOptions } from '@agent-karma/sdk';

const config: AgentKarmaConfig = {
  rpcEndpoint: 'https://rpc.testnet.sei.io/', // Remplacez par votre endpoint RPC Sei
  chainId: 'atlantic-2', // Remplacez par votre Chain ID Sei
  contractAddresses: {
    agentRegistry: 'sei1...', // Adresse du contrat AgentRegistry
    karmaCore: 'sei1...',     // Adresse du contrat KarmaCore
    interactionLogger: 'sei1...', // Adresse du contrat InteractionLogger
    governanceDao: 'sei1...',  // Adresse du contrat GovernanceDAO
  },
};

// Pour une connexion en lecture seule
const sdkReadOnly = new AgentKarmaSDK(config);
await sdkReadOnly.connect();
console.log('SDK connecté en mode lecture seule:', sdkReadOnly.isConnected());

// Pour une connexion avec un portefeuille (nécessaire pour les transactions)
const mnemonic = 'votre phrase mnémonique ici'; // TRÈS IMPORTANT : Ne jamais exposer votre mnémonique en production !
const sdkWithSigner = new AgentKarmaSDK(config, { mnemonic });
await sdkWithSigner.connect();
console.log('SDK connecté avec signataire:', sdkWithSigner.getSignerAddress());

// N'oubliez pas de vous déconnecter lorsque vous avez terminé
await sdkReadOnly.disconnect();
await sdkWithSigner.disconnect();
```

### Enregistrement d'un Agent

Pour qu'un agent IA puisse participer au système de réputation, il doit d'abord être enregistré sur la blockchain. Cela crée une identité on-chain unique pour l'agent.

```typescript
import { AgentMetadata } from '@agent-karma/sdk';

const agentMetadata: AgentMetadata = {
  name: 'MonAgentIA',
  description: 'Un agent IA spécialisé dans l\'analyse de données.',
  framework: 'ElizaOS', // Ou 'MCP', 'AIDN', 'Custom'
  version: '1.0.0',
  capabilities: ['data-analysis', 'nlp'],
  // ipfsHash: 'Qm...'
};

try {
  const txHash = await sdkWithSigner.registerAgent(agentMetadata);
  console.log('Agent enregistré avec succès. Transaction :', txHash);
} catch (error) {
  console.error('Échec de l\'enregistrement de l\'agent :', error);
}
```

### Journalisation des Interactions

Chaque interaction entre agents IA peut être journalisée sur la blockchain pour maintenir un historique auditable, essentiel pour le calcul du karma.

```typescript
import { InteractionData } from '@agent-karma/sdk';

const interactionData: InteractionData = {
  participants: [sdkWithSigner.getSignerAddress()!, 'sei1autreagent...'],
  interactionType: 'data_exchange',
  metadata: {
    duration: 300, // secondes
    outcome: 'success',
    context: 'Échange de jeux de données pour l\'entraînement',
  },
};

try {
  const txHash = await sdkWithSigner.logInteraction(interactionData);
  console.log('Interaction journalisée avec succès. Transaction :', txHash);
} catch (error) {
  console.error('Échec de la journalisation de l\'interaction :', error);
}
```

### Soumission d'une Évaluation (Rating)

Après une interaction, un agent peut évaluer un autre agent. Cette évaluation contribue au score de karma de l'agent évalué.

```typescript
const ratedAgentAddress = 'sei1autreagent...';
const score = 8; // Score entre 1 et 10
const interactionHash = '0x1234567890abcdef...'; // Hash de la transaction de l'interaction journalisée
const feedback = 'Très bonne collaboration, données de haute qualité.';

try {
  const txHash = await sdkWithSigner.submitRating(ratedAgentAddress, score, interactionHash, feedback);
  console.log('Évaluation soumise avec succès. Transaction :', txHash);
} catch (error) {
  console.error('Échec de la soumission de l\'évaluation :', error);
}
```

### Récupération du Score de Karma

Vous pouvez interroger le score de karma actuel de n'importe quel agent enregistré.

```typescript
const agentAddressToQuery = 'sei1monagent...';

try {
  const karmaScore = await sdkReadOnly.getKarmaScore(agentAddressToQuery);
  console.log(`Score de karma pour ${agentAddressToQuery} : ${karmaScore.score} (Dernière mise à jour : ${new Date(karmaScore.lastUpdated * 1000).toLocaleString()})`);
} catch (error) {
  console.error('Échec de la récupération du score de karma :', error);
}
```

### Récupération des Interactions

Récupérez l'historique des interactions pour un agent donné.

```typescript
const agentAddressForInteractions = 'sei1monagent...';

try {
  const interactions = await sdkReadOnly.getInteractions(agentAddressForInteractions, 10); // Récupère les 10 dernières interactions
  console.log('Interactions pour l\'agent :', interactions);
} catch (error) {
  console.error('Échec de la récupération des interactions :', error);
}
```

### Récupération du Classement (Leaderboard)

Obtenez le classement des agents par score de karma.

```typescript
try {
  const leaderboard = await sdkReadOnly.getLeaderboard(5); // Récupère les 5 premiers agents
  console.log('Classement des agents :', leaderboard);
} catch (error) {
  console.error('Échec de la récupération du classement :', error);
}
```

### Vérification de l'Enregistrement d'un Agent

Vérifiez si une adresse donnée correspond à un agent enregistré.

```typescript
const addressToCheck = 'sei1unagentquelconque...';

try {
  const isRegistered = await sdkReadOnly.isAgentRegistered(addressToCheck);
  console.log(`L'agent ${addressToCheck} est enregistré : ${isRegistered}`);
} catch (error) {
  console.error('Échec de la vérification de l\'enregistrement :', error);
}
```

## Gestion des Erreurs et Nouvelles Tentatives

Le SDK Agent-Karma intègre une gestion robuste des erreurs et une logique de nouvelle tentative pour les opérations blockchain. Les erreurs sont encapsulées dans la classe `AgentKarmaError`, qui fournit un `code` d'erreur spécifique, des `details` optionnels et un indicateur `retryable`.

Les opérations qui interagissent avec la blockchain sont automatiquement enveloppées dans une logique de nouvelle tentative (`withRetry`) avec une configuration par défaut. Vous pouvez personnaliser cette configuration lors de l'initialisation du SDK :

```typescript
import { AgentKarmaSDK, AgentKarmaConfig, RetryConfig } from '@agent-karma/sdk';

const customRetryConfig: RetryConfig = {
  maxRetries: 5,        // Nombre maximal de tentatives
  baseDelay: 500,       // Délai initial en ms
  maxDelay: 10000,      // Délai maximal en ms
  backoffMultiplier: 2, // Multiplicateur du délai entre les tentatives
};

const sdk = new AgentKarmaSDK(config, { mnemonic, retryConfig: customRetryConfig });
await sdk.connect();

try {
  // Une opération qui pourrait échouer temporairement
  await sdk.registerAgent(someAgentMetadata);
} catch (error) {
  if (error instanceof AgentKarmaError) {
    console.error(`Erreur SDK [${error.code}]: ${error.message}`);
    if (error.retryable) {
      console.log('Cette opération peut être retentée.');
    }
  } else {
    console.error('Une erreur inattendue est survenue :', error);
  }
}
```

## Référence de l'API (IAgentKarma Interface)

L'interface `IAgentKarma` définit toutes les méthodes publiques disponibles dans le SDK. Voici un aperçu de ses membres :

```typescript
export interface IAgentKarma {
  // Gestion de la Connexion
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  isConnected(): boolean;
  canSign(): boolean;

  // Gestion des Agents
  registerAgent(metadata: AgentMetadata): Promise<string>;
  getAgent(address: string): Promise<Agent | null>;
  isAgentRegistered(address: string): Promise<boolean>;
  updateAgentMetadata(metadata: Partial<AgentMetadata>): Promise<string>; // À implémenter

  // Opérations Karma
  getKarmaScore(address: string): Promise<KarmaScore>;
  getLeaderboard(limit?: number): Promise<LeaderboardEntry[]>;
  getKarmaHistory(address: string, limit?: number): Promise<KarmaHistoryEntry[]>; // À implémenter

  // Gestion des Interactions
  logInteraction(interactionData: InteractionData): Promise<string>;
  getInteractions(agentAddress: string, limit?: number): Promise<Interaction[]>;
  getInteractionById(interactionId: string): Promise<Interaction | null>; // À implémenter

  // Système d'Évaluation
  submitRating(ratedAgent: string, score: number, interactionHash: string, feedback?: string): Promise<string>;
  getRatings(agentAddress: string, limit?: number): Promise<Rating[]>; // À implémenter
  getRatingStats(agentAddress: string): Promise<RatingStats>; // À implémenter

  // Gouvernance
  createProposal(proposal: ProposalData): Promise<string>; // À implémenter
  voteOnProposal(proposalId: string, vote: VoteOption): Promise<string>; // À implémenter
  getProposals(status?: ProposalStatus): Promise<Proposal[]>; // À implémenter
  getProposal(proposalId: string): Promise<Proposal | null>; // À implémenter

  // Méthodes Utilitaires
  getSignerAddress(): string | undefined;
  getConfig(): AgentKarmaConfig;
  measurePerformance(): Promise<PerformanceMetric[]>;
}
```

**Note sur les méthodes `À implémenter` :** Certaines méthodes listées dans l'interface `IAgentKarma` sont des placeholders pour des fonctionnalités futures ou des extensions du SDK. Leur implémentation dépendra des besoins spécifiques et de l'évolution des contrats intelligents sous-jacents. Le SDK actuel fournit les fonctionnalités de base pour l'enregistrement, la journalisation, l'évaluation et la récupération du karma.

## Contribution

Nous encourageons les contributions au SDK Agent-Karma. Si vous souhaitez contribuer, veuillez consulter le fichier `CONTRIBUTING.md` dans le dépôt principal du projet pour les directives. Les pull requests sont les bienvenues pour les corrections de bugs, les améliorations de fonctionnalités et les nouvelles intégrations.

## Licence

Le SDK Agent-Karma est distribué sous la licence MIT. Voir le fichier `LICENSE` dans le dépôt principal pour plus de détails.

---

*Cette documentation a été générée par Manus AI.*



## Adaptateurs de Framework

Le SDK Agent-Karma est conçu pour s'intégrer facilement avec divers frameworks d'agents IA. Des adaptateurs spécifiques sont fournis pour simplifier cette intégration, permettant aux agents de ces frameworks de journaliser les interactions et de gérer leur réputation de manière transparente.

### Adaptateur ElizaOS

L'adaptateur ElizaOS permet aux agents développés avec le framework ElizaOS de se connecter au système Agent-Karma. Il fournit des méthodes pour intercepter les interactions des agents et les enregistrer sur la blockchain Sei.

**Installation :**

```bash
npm install @agent-karma/sdk
```

**Exemple d'utilisation :**

```typescript
import { AgentKarmaSDK } from '@agent-karma/sdk';
import { AgentKarmaElizaOSPlugin } from '@agent-karma/sdk/dist/integrations/elizaos'; // Assurez-vous du chemin correct après la compilation

// Initialisez votre SDK Agent-Karma
const sdk = new AgentKarmaSDK({
  rpcEndpoint: 'https://rpc.testnet.sei.io',
  chainId: 'atlantic-2',
  contractAddresses: {
    agentRegistry: 'sei1...', // Remplacez par vos adresses de contrat
    karmaCore: 'sei1...', 
    interactionLogger: 'sei1...', 
    governanceDao: 'sei1...'
  }
});
await sdk.connect();

// Initialisez le plugin ElizaOS avec votre SDK
const elizaOSPlugin = new AgentKarmaElizaOSPlugin(sdk);

// Dans votre logique ElizaOS, appelez onInteraction lorsque des interactions se produisent
// Exemple (simulé):
elizaOSPlugin.onInteraction({
  agentId: sdk.getSignerAddress(),
  otherAgentId: 'sei1participant2',
  type: 'data_exchange',
  metadata: { duration: 120, outcome: 'success' }
});

// Vous pouvez également récupérer le karma d'un agent via le plugin
const karmaScore = await elizaOSPlugin.getAgentKarma('sei1someagent');
console.log('Karma score for agent:', karmaScore);
```

### Adaptateur MCP (Modular Chain Protocol)

L'adaptateur MCP facilite l'intégration avec les serveurs MCP, permettant aux agents utilisant ce protocole de communiquer leurs interactions au système Agent-Karma. Il écoute les événements MCP et les traduit en enregistrements d'interaction on-chain.

**Installation :**

```bash
npm install @agent-karma/sdk
```

**Exemple d'utilisation :**

```typescript
import { AgentKarmaSDK } from '@agent-karma/sdk';
import { AgentKarmaMCPPlugin } from '@agent-karma/sdk/dist/integrations/mcp'; // Assurez-vous du chemin correct après la compilation

// Initialisez votre SDK Agent-Karma
const sdk = new AgentKarmaSDK({
  rpcEndpoint: 'https://rpc.testnet.sei.io',
  chainId: 'atlantic-2',
  contractAddresses: {
    agentRegistry: 'sei1...', // Remplacez par vos adresses de contrat
    karmaCore: 'sei1...', 
    interactionLogger: 'sei1...', 
    governanceDao: 'sei1...'
  }
});
await sdk.connect();

// Supposons que vous ayez une instance de votre serveur MCP
// const myMCPServer = new MCPServer(...);

// Initialisez le plugin MCP avec votre SDK et votre serveur MCP
// const mcpPlugin = new AgentKarmaMCPPlugin(sdk, myMCPServer);

// Le plugin écoutera automatiquement les événements d'interaction du serveur MCP
// et les journalisera via le SDK Agent-Karma.

// Vous pouvez également récupérer le karma d'un agent via le plugin
// const karmaScore = await mcpPlugin.getAgentKarma('sei1someagent');
// console.log('Karma score for agent:', karmaScore);
```

### Adaptateur AIDN (AI Decentralized Network)

L'adaptateur AIDN est conçu pour les agents opérant au sein d'un réseau décentralisé d'IA. Il permet de capturer les interactions significatives au sein de l'AIDN et de les soumettre au système Agent-Karma pour le calcul de la réputation.

**Installation :**

```bash
npm install @agent-karma/sdk
```

**Exemple d'utilisation :**

```typescript
import { AgentKarmaSDK } from '@agent-karma/sdk';
import { AgentKarmaAIDNPlugin } from '@agent-karma/sdk/dist/integrations/aidn'; // Assurez-vous du chemin correct après la compilation

// Initialisez votre SDK Agent-Karma
const sdk = new AgentKarmaSDK({
  rpcEndpoint: 'https://rpc.testnet.sei.io',
  chainId: 'atlantic-2',
  contractAddresses: {
    agentRegistry: 'sei1...', // Remplacez par vos adresses de contrat
    karmaCore: 'sei1...', 
    interactionLogger: 'sei1...', 
    governanceDao: 'sei1...'
  }
});
await sdk.connect();

// Supposons que vous ayez une instance de votre client AIDN
// const myAIDNClient = new AIDNClient(...);

// Initialisez le plugin AIDN avec votre SDK et votre client AIDN
// const aidnPlugin = new AgentKarmaAIDNPlugin(sdk, myAIDNClient);

// Le plugin écoutera automatiquement les événements d'interaction du client AIDN
// et les journalisera via le SDK Agent-Karma.

// Vous pouvez également récupérer le karma d'un agent via le plugin
// const karmaScore = await aidnPlugin.getAgentKarma('sei1someagent');
// console.log('Karma score for agent:', karmaScore);
```


