# Analyse de l'architecture des contrats CosmWasm

Ce document détaille l'architecture des contrats intelligents CosmWasm du projet `agent-karma`, en vue de leur migration vers Solidity pour une compatibilité EVM sur le réseau Sei.

## 1. Vue d'ensemble des modules

Le répertoire `src` des contrats CosmWasm contient les modules suivants, comme indiqué par `lib.rs`:

- `types.rs`: Définit les structures de données utilisées à travers les contrats.
- `interfaces.rs`: Définit les traits (interfaces) que les contrats doivent implémenter, décrivant les fonctionnalités principales.
- `events.rs`: Définit les événements émis par les contrats.
- `errors.rs`: Définit les types d'erreurs personnalisés.
- `messages.rs`: Définit les messages d'exécution et de requête pour interagir avec les contrats.
- `docs.rs`: Documentation.
- `tests.rs`: Tests unitaires.

## 2. Interfaces principales (`interfaces.rs`)

Le fichier `interfaces.rs` est crucial car il expose les fonctionnalités de haut niveau des contrats. Il définit cinq traits principaux, chacun représentant un aspect fondamental du système `agent-karma`:

### 2.1. `IAgentRegistry`

Ce trait gère l'enregistrement et la vérification de l'identité des agents dans le système. Ses fonctions clés sont:

- `register_agent(agent_address: Addr, metadata: AgentMetadata)`: Enregistre un nouvel agent avec ses métadonnées.
- `get_agent_info(agent_address: Addr)`: Récupère les informations d'un agent.
- `is_registered_agent(agent_address: Addr)`: Vérifie si un agent est enregistré.
- `update_agent_metadata(agent_address: Addr, metadata: AgentMetadata)`: Met à jour les métadonnées d'un agent.

### 2.2. `IKarmaCore`

Ce trait est responsable du calcul du score de karma, de la soumission des évaluations et des requêtes de score. Ses fonctions clés sont:

- `submit_rating(rater: Addr, rated_agent: Addr, score: u8, feedback: Option<String>, interaction_hash: String)`: Soumet une évaluation pour un autre agent.
- `calculate_karma(agent_address: Addr)`: Calcule le score de karma détaillé pour un agent.
- `get_karma_score(agent_address: Addr)`: Obtient le score de karma actuel d'un agent.
- `get_karma_history(agent_address: Addr, limit: Option<u32>)`: Obtient l'historique du karma d'un agent.
- `get_agent_ratings(agent_address: Addr, limit: Option<u32>)`: Obtient toutes les évaluations reçues par un agent.

### 2.3. `IInteractionLogger`

Ce trait enregistre toutes les interactions entre agents pour la transparence et l'audit. Ses fonctions clés sont:

- `log_interaction(participants: Vec<Addr>, interaction_type: String, metadata: InteractionMetadata)`: Enregistre une interaction.
- `get_interaction_history(agent_address: Addr, limit: Option<u32>)`: Récupère l'historique des interactions d'un agent.
- `verify_interaction(interaction_hash: String)`: Vérifie l'existence et la validité d'une interaction.
- `get_interaction_by_hash(interaction_hash: String)`: Obtient les détails d'une interaction par son hachage.

### 2.4. `IGovernanceDAO`

Ce trait gère les propositions de gouvernance, le vote et l'exécution des décisions. Ses fonctions clés sont:

- `create_proposal(proposer: Addr, title: String, description: String, calldata: String, voting_period: u64)`: Crée une nouvelle proposition de gouvernance.
- `vote_proposal(voter: Addr, proposal_id: u64, support: bool)`: Vote sur une proposition.
- `finalize_proposal(proposal_id: u64)`: Finalise une proposition après la période de vote.
- `get_proposal(proposal_id: u64)`: Obtient les détails d'une proposition.
- `calculate_voting_power(voter: Addr)`: Calcule le pouvoir de vote basé sur le karma.
- `get_active_proposals()`: Obtient toutes les propositions actives.

### 2.5. `IOracleIntegration`

Ce trait gère l'intégration des données d'oracle externes et leur validation. Ses fonctions clés sont:

- `submit_oracle_data(provider: Addr, data_type: String, data: String, signatures: Vec<String>)`: Soumet des données externes d'un fournisseur d'oracle.
- `verify_oracle_consensus(data_hash: String)`: Vérifie le consensus des données d'oracle.
- `get_oracle_data(data_type: String, timestamp: Option<Timestamp>)`: Obtient les données d'oracle vérifiées.

## 3. Prochaines étapes pour la migration vers Solidity

La migration vers Solidity nécessitera de traduire ces interfaces et les logiques sous-jacentes en contrats Solidity. Chaque trait CosmWasm pourrait potentiellement devenir un contrat Solidity distinct ou un module au sein d'un contrat plus grand, en fonction de la complexité et des interdépendances. Une attention particulière devra être portée à la gestion des types de données (par exemple, `Addr` en `address`, `Uint128` en `uint256`), à la gestion des erreurs, et à la traduction des concepts spécifiques à CosmWasm (comme `Response` et `StdResult`) en idiomes Solidity. Les événements et les structures de données définis dans `events.rs` et `types.rs` devront également être adaptés pour Solidity.



## 3. Types de données (`types.rs`)

Le fichier `types.rs` définit les structures de données fondamentales utilisées par les contrats. La compréhension de ces types est essentielle pour leur traduction en Solidity.

### 3.1. `Agent`

Représente un agent AI enregistré dans le système. Il contient les champs suivants:

- `address`: `Addr` (Adresse blockchain de l'agent)
- `registration_date`: `Timestamp` (Horodatage de l'enregistrement)
- `metadata`: `AgentMetadata` (Métadonnées de l'agent)
- `karma_score`: `Uint128` (Score de karma actuel)
- `interaction_count`: `u64` (Nombre total d'interactions)
- `ratings_received`: `u64` (Nombre total d'évaluations reçues)

### 3.2. `AgentMetadata`

Métadonnées associées à un agent:

- `name`: `String` (Nom d'affichage de l'agent)
- `description`: `String` (Description de l'objectif de l'agent)
- `framework`: `String` (Framework AI utilisé)
- `version`: `String` (Version de l'agent)
- `ipfs_hash`: `Option<String>` (Hash IPFS optionnel pour les métadonnées étendues)

### 3.3. `Rating`

Représente une évaluation donnée par un agent à un autre:

- `id`: `String` (Identifiant unique de l'évaluation)
- `rater_address`: `Addr` (Adresse de l'agent évaluateur)
- `rated_address`: `Addr` (Adresse de l'agent évalué)
- `score`: `u8` (Score d'évaluation (1-10))
- `feedback`: `Option<String>` (Texte de feedback optionnel)
- `interaction_hash`: `String` (Hash de l'interaction à laquelle cette évaluation se réfère)
- `timestamp`: `Timestamp` (Horodatage de la soumission de l'évaluation)
- `block_height`: `u64` (Hauteur du bloc blockchain Sei)

### 3.4. `Interaction`

Représente une interaction entre agents:

- `id`: `String` (Identifiant unique de l'interaction)
- `participants`: `Vec<Addr>` (Adresses des agents impliqués)
- `interaction_type`: `String` (Type d'interaction)
- `timestamp`: `Timestamp` (Horodatage de l'interaction)
- `block_height`: `u64` (Hauteur du bloc blockchain Sei)
- `metadata`: `InteractionMetadata` (Métadonnées supplémentaires sur l'interaction)

### 3.5. `InteractionMetadata`

Métadonnées pour les interactions:

- `duration`: `Option<u64>` (Durée de l'interaction en secondes)
- `outcome`: `Option<String>` (Résultat de l'interaction)
- `context`: `Option<String>` (Contexte ou informations supplémentaires)

### 3.6. `KarmaCalculation`

Détails du calcul du karma:

- `agent_address`: `Addr` (Adresse de l'agent)
- `current_score`: `Uint128` (Score de karma actuel)
- `previous_score`: `Uint128` (Score de karma précédent)
- `factors`: `KarmaFactors` (Facteurs ayant contribué au calcul)
- `last_updated`: `Timestamp` (Horodatage de la dernière mise à jour)
- `calculation_hash`: `String` (Hash à des fins de vérification)

### 3.7. `KarmaFactors`

Facteurs utilisés dans le calcul du karma:

- `average_rating`: `String` (Moyenne des évaluations reçues)
- `rating_count`: `u64` (Nombre total d'évaluations)
- `interaction_frequency`: `Uint128` (Score de fréquence d'interaction)
- `time_decay`: `String` (Facteur de décroissance temporelle)
- `external_factors`: `Option<Uint128>` (Facteurs externes des oracles)

### 3.8. `Proposal`

Proposition de gouvernance:

- `id`: `u64` (ID unique de la proposition)
- `title`: `String` (Titre de la proposition)
- `description`: `String` (Description détaillée)
- `proposer`: `Addr` (Adresse du proposant)
- `calldata`: `String` (Appel de fonction encodé à exécuter si la proposition est acceptée)
- `created_at`: `Timestamp` (Horodatage de la création)
- `voting_deadline`: `Timestamp` (Date limite de vote)
- `executed`: `bool` (Indique si la proposition a été exécutée)
- `votes_for`: `Uint128` (Votes pondérés par le karma en faveur)
- `votes_against`: `Uint128` (Votes pondérés par le karma contre)
- `quorum_required`: `Uint128` (Quorum minimum requis)
- `status`: `ProposalStatus` (Statut actuel de la proposition)

### 3.9. `ProposalStatus`

Statut d'une proposition de gouvernance (énumération):

- `Active`
- `Passed`
- `Failed`
- `Executed`

### 3.10. `Vote`

Vote sur une proposition de gouvernance:

- `proposal_id`: `u64` (ID de la proposition votée)
- `voter`: `Addr` (Adresse du votant)
- `support`: `bool` (Soutien à la proposition (true = oui, false = non))
- `voting_power`: `Uint128` (Pouvoir de vote au moment du vote)
- `timestamp`: `Timestamp` (Horodatage du vote)
- `block_height`: `u64` (Hauteur du bloc lors du vote)

### 3.11. `OracleData`

Soumission de données d'oracle:

- `provider`: `Addr` (Adresse du fournisseur d'oracle)
- `data_type`: `String` (Type de données fournies)
- `data`: `String` (Charge utile des données)
- `timestamp`: `Timestamp` (Horodatage de la soumission des données)
- `signatures`: `Vec<String>` (Signatures des nœuds validateurs)
- `verified`: `bool` (Indique si les données ont été vérifiées)

### 3.12. `KarmaConfig`

Configuration des paramètres de calcul du karma:

- `min_karma_for_rating`: `Uint128` (Karma minimum requis pour la soumission d'évaluation)
- `min_karma_for_voting`: `Uint128` (Karma minimum requis pour le vote de gouvernance)
- `min_karma_for_proposal`: `Uint128` (Karma minimum requis pour la création de proposition)
- `rating_window`: `u64` (Fenêtre de temps pour la soumission d'évaluation après interaction (en secondes))
- `max_ratings_per_interaction`: `u8` (Nombre maximum d'évaluations par interaction)
- `rating_fee`: `Uint128` (Frais de karma pour la soumission d'évaluation)

## 4. Correspondance des types CosmWasm/Rust vers Solidity

Voici une proposition de correspondance des types de données CosmWasm/Rust vers leurs équivalents Solidity:

| Type CosmWasm/Rust | Type Solidity | Notes |
|---|---|---|
| `Addr` | `address` | Représente une adresse blockchain. |
| `Timestamp` | `uint256` ou `uint64` | Représente un horodatage. Solidity utilise `uint` pour les timestamps Unix. |
| `Uint128` | `uint256` | Un entier non signé de 128 bits, équivalent à `uint256` en Solidity. |
| `String` | `string` | Chaînes de caractères. |
| `u8`, `u64` | `uint8`, `uint64` | Entiers non signés de différentes tailles. |
| `bool` | `bool` | Valeurs booléennes. |
| `Vec<T>` | `T[]` ou `mapping(uint => T)` | Tableaux dynamiques ou mappings. Le choix dépendra de l'utilisation. |
| `Option<T>` | `T` ou `(bool, T)` | Peut être représenté par la valeur par défaut du type en Solidity (par exemple, 0 pour `uint`, `address(0)` pour `address`) ou par un tuple `(bool, T)` pour indiquer la présence/absence. |
| `enum` | `uint8` ou `enum` | Les énumérations peuvent être mappées à des entiers ou à des types `enum` en Solidity. |

Cette analyse des types de données est fondamentale pour la phase de conception de l'architecture Solidity, où chaque structure CosmWasm sera traduite en `struct` ou en variables d'état dans les contrats Solidity.

