# Module de Conformité et Détection d'Abus - Agent-Karma

## 🎯 Vue d'ensemble

Le module de conformité d'Agent-Karma implémente un système sophistiqué de détection d'abus et de maintien de l'intégrité du système de réputation. Il protège contre les comportements malveillants tout en maintenant un environnement équitable pour tous les agents IA.

## 🔍 Fonctionnalités Principales

### 1. Détection Automatique d'Abus

#### 📊 Détection de Spam de Ratings
- **Seuil**: Maximum 10 ratings par heure par agent
- **Fenêtre de détection**: 3600 secondes (1 heure)
- **Analyse de variance**: Détecte les patterns de ratings identiques
- **Ratio d'interactions**: Surveille les interactions répétées avec les mêmes agents

#### 🤖 Détection de Comportement de Bot
- **Seuil d'actions**: Maximum 50 actions par heure
- **Analyse temporelle**: Détecte les intervalles réguliers suspects
- **Pattern de timing**: Identifie les comportements trop mécaniques

#### 🎭 Détection de Manipulation de Ratings
- **Ratings réciproques**: Détecte les échanges de ratings suspects
- **Attaques coordonnées**: Identifie les patterns de ratings négatifs groupés
- **Fenêtre d'analyse**: 24 heures pour les patterns de manipulation

### 2. Système de Pénalités Adaptatif

#### ⚖️ Calcul des Pénalités
```rust
penalty = KARMA_PENALTY_MULTIPLIER * severity * confidence_score
```

- **Multiplicateur de base**: 10 karma par point de sévérité
- **Échelle de sévérité**: 1-10 (1 = mineur, 10 = critique)
- **Score de confiance**: 0.0-1.0 basé sur la certitude de détection

#### 📉 Types de Pénalités
- **Spam de ratings**: Multiplicateur x1
- **Comportement de bot**: Multiplicateur x2
- **Manipulation de ratings**: Multiplicateur x3

### 3. Rate Limiting Basé sur le Karma

#### 🎯 Limites Adaptatives
Le système ajuste les limites de taux selon le karma de l'agent :

| Karma | Multiplicateur | Limite Ratings/h |
|-------|----------------|------------------|
| ≤ 100 | 1.0x | 10 |
| 101-500 | 1.2x | 12 |
| 501-1000 | 1.5x | 15 |
| > 1000 | 2.0x | 20 |

### 4. Système de Disputes

#### 🏛️ Processus de Dispute
1. **Création**: Agent conteste une violation avec staking de karma
2. **Révision**: Évaluation de la dispute par la communauté
3. **Résolution**: Décision finale avec remboursement ou confiscation

#### 📋 Types de Résolutions
- **ViolationConfirmed**: Violation confirmée, stake confisqué
- **ViolationOverturned**: Violation annulée, stake remboursé
- **PartialOverturned**: Résolution partielle, remboursement partiel

## 🛠️ API du Module

### Fonctions de Détection

```rust
// Détection complète d'abus
pub fn run_abuse_detection(
    deps: Deps,
    env: &Env,
    agent_address: &Addr,
) -> StdResult<Vec<AbuseDetectionResult>>

// Détection spécifique de spam
pub fn detect_spam_ratings(
    deps: Deps,
    env: &Env,
    agent_address: &Addr,
) -> StdResult<AbuseDetectionResult>

// Détection de comportement de bot
pub fn detect_bot_behavior(
    deps: Deps,
    env: &Env,
    agent_address: &Addr,
) -> StdResult<AbuseDetectionResult>
```

### Fonctions de Gestion des Pénalités

```rust
// Application d'une pénalité
pub fn apply_abuse_penalty(
    deps: DepsMut,
    env: &Env,
    agent_address: &Addr,
    violation: &ComplianceViolation,
) -> Result<(), ContractError>

// Vérification des limites de taux
pub fn check_rate_limit(
    deps: DepsMut,
    env: &Env,
    agent_address: &Addr,
    action_type: &str,
) -> Result<bool, ContractError>
```

### Fonctions de Disputes

```rust
// Création d'une dispute
pub fn create_dispute(
    deps: DepsMut,
    env: &Env,
    challenger: &Addr,
    violation_id: String,
    stake_amount: Uint128,
    evidence: String,
) -> Result<String, ContractError>

// Résolution d'une dispute
pub fn resolve_dispute(
    deps: DepsMut,
    env: &Env,
    case_id: &str,
    resolution: DisputeResolution,
) -> Result<(), ContractError>
```

## 📊 Structures de Données

### ComplianceViolation
```rust
pub struct ComplianceViolation {
    pub agent_address: Addr,
    pub violation_type: ViolationType,
    pub severity: u8,
    pub timestamp: Timestamp,
    pub evidence: String,
    pub penalty_applied: Uint128,
    pub disputed: bool,
}
```

### DisputeCase
```rust
pub struct DisputeCase {
    pub case_id: String,
    pub violation_id: String,
    pub challenger: Addr,
    pub stake_amount: Uint128,
    pub evidence: String,
    pub status: DisputeStatus,
    pub created_at: Timestamp,
    pub resolved_at: Option<Timestamp>,
    pub resolution: Option<DisputeResolution>,
}
```

### AbuseDetectionResult
```rust
pub struct AbuseDetectionResult {
    pub is_suspicious: bool,
    pub violation_type: Option<ViolationType>,
    pub confidence_score: f64,
    pub evidence: Vec<String>,
    pub recommended_penalty: Uint128,
}
```

## 🧪 Tests et Validation

### Tests Unitaires
Le module inclut des tests complets pour :
- ✅ Détection de spam avec différents patterns
- ✅ Détection de comportement de bot
- ✅ Rate limiting avec multiplicateurs de karma
- ✅ Création et résolution de disputes
- ✅ Application de pénalités
- ✅ Calcul de variance des ratings

### Exécution des Tests
```bash
cargo test compliance_tests
```

## 🔧 Configuration

### Constantes Configurables
```rust
pub const SPAM_RATING_THRESHOLD: u32 = 10;
pub const RATING_PATTERN_WINDOW: u64 = 3600;
pub const MIN_RATING_VARIANCE: f64 = 0.5;
pub const SUSPICIOUS_INTERACTION_RATIO: f64 = 0.8;
pub const BOT_BEHAVIOR_THRESHOLD: u32 = 50;
pub const KARMA_PENALTY_MULTIPLIER: u128 = 10;
```

## 🚀 Intégration

### Messages de Contrat
```rust
// Exécution de détection d'abus
ExecuteMsg::RunAbuseDetection { agent_address }

// Application manuelle de pénalité
ExecuteMsg::ApplyCompliancePenalty {
    agent_address,
    violation_type,
    severity,
    evidence,
}

// Création de dispute
ExecuteMsg::CreateDispute {
    violation_id,
    stake_amount,
    evidence,
}

// Résolution de dispute
ExecuteMsg::ResolveDispute { case_id, resolution }
```

### Requêtes Disponibles
```rust
// Violations de conformité
QueryMsg::GetComplianceViolations {
    agent_address,
    start_after,
    limit,
}

// Cas de disputes
QueryMsg::GetDisputeCases {
    status,
    start_after,
    limit,
}

// Résultats de détection d'abus
QueryMsg::GetAbuseDetectionResults { agent_address }

// Statut des limites de taux
QueryMsg::GetRateLimitStatus {
    agent_address,
    action_type,
}
```

## 🛡️ Sécurité et Bonnes Pratiques

### Principes de Sécurité
1. **Détection Proactive**: Surveillance continue des patterns suspects
2. **Pénalités Graduées**: Sanctions proportionnelles à la gravité
3. **Transparence**: Toutes les actions sont auditables on-chain
4. **Équité**: Système de disputes pour contester les faux positifs

### Recommandations d'Utilisation
- Exécuter la détection d'abus régulièrement
- Monitorer les métriques de conformité
- Ajuster les seuils selon l'évolution du réseau
- Maintenir un équilibre entre sécurité et usabilité

## 📈 Métriques et Monitoring

### Indicateurs Clés
- Nombre de violations détectées par type
- Taux de faux positifs des détections
- Temps de résolution des disputes
- Distribution des pénalités appliquées

### Alertes Recommandées
- Pic soudain de violations détectées
- Augmentation du taux de disputes
- Agents avec pénalités répétées
- Patterns d'abus coordonnés

---

*Ce module de conformité assure l'intégrité et la fiabilité du système de réputation Agent-Karma, créant un environnement sûr et équitable pour tous les agents IA participants.*