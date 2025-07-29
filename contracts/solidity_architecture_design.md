# Conception de l'architecture Solidity pour Agent-Karma

Ce document décrit la conception de l'architecture des contrats intelligents Solidity, équivalente aux contrats CosmWasm existants du projet `agent-karma`. L'objectif est d'assurer la compatibilité avec l'EVM du réseau Sei tout en conservant les fonctionnalités principales du système de karma.

## 1. Principes de conception généraux

La migration de CosmWasm (Rust) vers Solidity (EVM) implique des considérations architecturales importantes. Les principes suivants guideront la conception:

- **Modularité**: Chaque interface CosmWasm (`IAgentRegistry`, `IKarmaCore`, etc.) sera traduite en un contrat Solidity distinct ou en une bibliothèque, favorisant la réutilisabilité et la maintenabilité.
- **Sécurité**: Les meilleures pratiques de sécurité Solidity seront appliquées, y compris la gestion des accès, la prévention des réentrances et la validation des entrées.
- **Efficacité du gaz**: La conception visera à minimiser la consommation de gaz en optimisant le stockage des données et la logique des fonctions.
- **Interopérabilité**: Si nécessaire, des mécanismes seront prévus pour l'interaction entre les différents contrats Solidity.
- **Traduction sémantique**: L'objectif est de traduire la sémantique des opérations CosmWasm en idiomes Solidity, plutôt qu'une simple traduction ligne par ligne.

## 2. Traduction des interfaces CosmWasm en contrats Solidity

Chacune des interfaces CosmWasm identifiées précédemment (`IAgentRegistry`, `IKarmaCore`, `IInteractionLogger`, `IGovernanceDAO`, `IOracleIntegration`) sera mappée à un ou plusieurs contrats Solidity.

### 2.1. `AgentRegistry.sol` (équivalent à `IAgentRegistry`)

Ce contrat gérera l'enregistrement et la gestion des agents. Il contiendra un mapping pour stocker les informations des agents.

**État**: 
- `mapping(address => Agent) public agents;` : Stocke les informations des agents par leur adresse.
- `mapping(address => bool) public isRegistered;` : Vérifie rapidement si une adresse est enregistrée.

**Fonctions (visibilité `public` ou `external`)**:
- `registerAgent(AgentMetadata calldata metadata)`: Enregistre un nouvel agent. Vérifiera si l'appelant est déjà enregistré et si les métadonnées sont valides.
- `getAgentInfo(address agentAddress) view returns (Agent memory)`: Récupère les informations d'un agent. Retournera une structure `Agent`.
- `isAgentRegistered(address agentAddress) view returns (bool)`: Vérifie si un agent est enregistré.
- `updateAgentMetadata(AgentMetadata calldata metadata)`: Met à jour les métadonnées d'un agent. Vérifiera que l'appelant est l'agent concerné et qu'il est enregistré.

**Événements**:
- `AgentRegistered(address indexed agentAddress, string name, string framework)`
- `AgentMetadataUpdated(address indexed agentAddress, string name)`

### 2.2. `KarmaCore.sol` (équivalent à `IKarmaCore`)

Ce contrat gérera le calcul et la gestion du karma. Il interagira potentiellement avec `AgentRegistry` pour vérifier l'existence des agents.

**État**:
- `mapping(address => uint256) public karmaScores;` : Stocke le score de karma de chaque agent.
- `mapping(bytes32 => Rating) public ratings;` : Stocke les évaluations par un hash d'interaction unique.
- `mapping(address => mapping(bytes32 => bool)) public hasRatedInteraction;` : Empêche les doubles évaluations pour une interaction donnée.

**Fonctions**:
- `submitRating(address ratedAgent, uint8 score, string calldata feedback, bytes32 interactionHash)`: Soumet une évaluation. Vérifiera la plage du score, la fenêtre d'évaluation et si l'interaction a déjà été évaluée.
- `calculateKarma(address agentAddress) view returns (KarmaCalculation memory)`: Calcule le karma détaillé. Cette fonction pourrait être complexe et nécessiter des helpers internes.
- `getKarmaScore(address agentAddress) view returns (uint256)`: Retourne le score de karma actuel.
- `getKarmaHistory(address agentAddress, uint32 limit) view returns (KarmaCalculation[] memory)`: Récupère l'historique du karma. Cela pourrait nécessiter un tableau dynamique ou un mapping.
- `getAgentRatings(address agentAddress, uint32 limit) view returns (Rating[] memory)`: Récupère les évaluations reçues par un agent.

**Événements**:
- `RatingSubmitted(address indexed rater, address indexed ratedAgent, uint8 score, bytes32 interactionHash)`
- `KarmaScoreUpdated(address indexed agentAddress, uint256 newScore, uint256 oldScore)`

### 2.3. `InteractionLogger.sol` (équivalent à `IInteractionLogger`)

Ce contrat enregistrera les interactions entre agents.

**État**:
- `mapping(bytes32 => Interaction) public interactions;` : Stocke les interactions par leur hash.

**Fonctions**:
- `logInteraction(address[] calldata participants, string calldata interactionType, InteractionMetadata calldata metadata) returns (bytes32)`: Enregistre une interaction et retourne son hash.
- `getInteractionHistory(address agentAddress, uint32 limit) view returns (Interaction[] memory)`: Récupère l'historique des interactions.
- `verifyInteraction(bytes32 interactionHash) view returns (bool)`: Vérifie l'existence et la validité d'une interaction.
- `getInteractionByHash(bytes32 interactionHash) view returns (Interaction memory)`: Récupère les détails d'une interaction par son hash.

**Événements**:
- `InteractionLogged(bytes32 indexed interactionHash, address[] participants, string interactionType)`

### 2.4. `GovernanceDAO.sol` (équivalent à `IGovernanceDAO`)

Ce contrat gérera les propositions de gouvernance et le vote.

**État**:
- `uint256 public nextProposalId;` : Compteur pour les IDs de proposition.
- `mapping(uint256 => Proposal) public proposals;` : Stocke les propositions par ID.
- `mapping(uint256 => mapping(address => Vote)) public votes;` : Stocke les votes par proposition et par votant.

**Fonctions**:
- `createProposal(string calldata title, string calldata description, bytes calldata calldataToExecute, uint64 votingPeriod)`: Crée une nouvelle proposition. Vérifiera le karma minimum du proposant.
- `voteProposal(uint256 proposalId, bool support)`: Vote sur une proposition. Vérifiera le karma du votant, la période de vote et si le votant a déjà voté.
- `finalizeProposal(uint256 proposalId)`: Finalise une proposition. Exécutera `calldataToExecute` si la proposition est passée.
- `getProposal(uint256 proposalId) view returns (Proposal memory)`: Récupère les détails d'une proposition.
- `calculateVotingPower(address voter) view returns (uint256)`: Calcule le pouvoir de vote basé sur le karma (interagira avec `KarmaCore`).
- `getActiveProposals() view returns (Proposal[] memory)`: Récupère toutes les propositions actives.

**Événements**:
- `ProposalCreated(uint256 indexed proposalId, address indexed proposer, string title)`
- `VoteCast(uint256 indexed proposalId, address indexed voter, bool support, uint256 votingPower)`
- `ProposalFinalized(uint256 indexed proposalId, bool passed, bool executed)`

### 2.5. `OracleIntegration.sol` (équivalent à `IOracleIntegration`)

Ce contrat gérera l'intégration des données d'oracle.

**État**:
- `mapping(bytes32 => OracleData) public oracleData;` : Stocke les données d'oracle par leur hash.

**Fonctions**:
- `submitOracleData(address provider, string calldata dataType, string calldata data, string[] calldata signatures)`: Soumet des données d'oracle.
- `verifyOracleConsensus(bytes32 dataHash) view returns (bool)`: Vérifie le consensus des données.
- `getOracleData(string calldata dataType, uint256 timestamp) view returns (OracleData[] memory)`: Récupère les données d'oracle vérifiées.

**Événements**:
- `OracleDataSubmitted(address indexed provider, string dataType, bytes32 indexed dataHash)`
- `OracleDataVerified(bytes32 indexed dataHash)`

## 3. Traduction des types de données CosmWasm en structures Solidity

Les structures de données définies dans `types.rs` seront traduites en `struct` Solidity. Les mappings de types sont basés sur l'analyse précédente.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Agent.sol
struct Agent {
    address agentAddress;
    uint256 registrationDate;
    AgentMetadata metadata;
    uint256 karmaScore;
    uint64 interactionCount;
    uint64 ratingsReceived;
}

struct AgentMetadata {
    string name;
    string description;
    string framework;
    string version;
    string ipfsHash; // Option<String> becomes string, empty string for None
}

// KarmaCore.sol
struct Rating {
    bytes32 id; // Unique identifier, can be a hash
    address raterAddress;
    address ratedAddress;
    uint8 score;
    string feedback; // Option<String> becomes string, empty string for None
    bytes32 interactionHash;
    uint256 timestamp;
    uint64 blockHeight;
}

struct Interaction {
    bytes32 id; // Unique identifier, can be a hash
    address[] participants;
    string interactionType;
    uint256 timestamp;
    uint64 blockHeight;
    InteractionMetadata metadata;
}

struct InteractionMetadata {
    uint64 duration; // Option<u64> becomes uint64, 0 for None
    string outcome; // Option<String> becomes string, empty string for None
    string context; // Option<String> becomes string, empty string for None
}

struct KarmaCalculation {
    address agentAddress;
    uint256 currentScore;
    uint256 previousScore;
    KarmaFactors factors;
    uint256 lastUpdated;
    bytes32 calculationHash;
}

struct KarmaFactors {
    string averageRating; // Stored as string to avoid float issues
    uint64 ratingCount;
    uint256 interactionFrequency;
    string timeDecay; // Stored as string
    uint256 externalFactors; // Option<Uint128> becomes uint256, 0 for None
}

// GovernanceDAO.sol
struct Proposal {
    uint256 id;
    string title;
    string description;
    address proposer;
    bytes calldataToExecute; // calldata becomes bytes
    uint256 createdAt;
    uint256 votingDeadline;
    bool executed;
    uint256 votesFor;
    uint256 votesAgainst;
    uint256 quorumRequired;
    ProposalStatus status;
}

enum ProposalStatus {
    Active,
    Passed,
    Failed,
    Executed
}

struct Vote {
    uint256 proposalId;
    address voter;
    bool support;
    uint256 votingPower;
    uint256 timestamp;
    uint64 blockHeight;
}

// OracleIntegration.sol
struct OracleData {
    address provider;
    string dataType;
    string data;
    uint256 timestamp;
    string[] signatures; // Vec<String> becomes string[]
    bool verified;
}

// KarmaConfig.sol (potentially a separate config contract or part of KarmaCore)
struct KarmaConfig {
    uint256 minKarmaForRating;
    uint256 minKarmaForVoting;
    uint256 minKarmaForProposal;
    uint64 ratingWindow;
    uint8 maxRatingsPerInteraction;
    uint256 ratingFee;
}
```

## 4. Considérations supplémentaires

- **Gestion des erreurs**: Les `StdResult` de Rust seront remplacés par des `require()` et `revert()` en Solidity, avec des messages d'erreur clairs.
- **Droits d'accès**: Des modificateurs comme `onlyOwner` ou des rôles basés sur des adresses seront implémentés pour contrôler l'accès aux fonctions sensibles.
- **Bibliothèques**: Des bibliothèques Solidity (par exemple, OpenZeppelin) seront utilisées pour des fonctionnalités courantes comme la gestion des accès (`Ownable`) ou les opérations mathématiques sûres (`SafeMath` si nécessaire, bien que Solidity 0.8+ gère l'overflow par défaut).
- **Interactions entre contrats**: Les contrats devront être conscients les uns des autres. Par exemple, `KarmaCore` aura besoin de l'adresse de `AgentRegistry` pour vérifier les agents. Cela peut être géré par des constructeurs qui prennent les adresses des contrats dépendants, ou par des fonctions de `setAddress` après le déploiement.
- **Stockage des données**: Les données seront stockées dans des mappings ou des tableaux dynamiques, en fonction des besoins d'accès et d'itération.

Cette conception fournit une feuille de route pour l'implémentation des contrats Solidity, en s'appuyant sur les fonctionnalités existantes des contrats CosmWasm. La prochaine étape sera l'implémentation effective de ces contrats en code Solidity.

