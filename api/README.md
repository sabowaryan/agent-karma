# Agent-Karma API 🌐

> **API REST et WebSocket pour le système de réputation Agent-Karma**

L'API Agent-Karma fournit une interface HTTP/WebSocket pour interagir avec le système de réputation décentralisé, permettant l'intégration avec des applications web, mobiles et services tiers.

---

## 🎯 Vue d'ensemble

L'API Agent-Karma sert de passerelle entre les applications clientes et la blockchain Sei, offrant des endpoints REST pour les opérations CRUD et des WebSockets pour les mises à jour temps réel.

### ✨ Fonctionnalités principales

- **🔗 API REST complète** avec OpenAPI 3.0
- **⚡ WebSocket temps réel** pour les événements
- **🔒 Authentification JWT** sécurisée
- **📊 Cache Redis** haute performance
- **🛡️ Rate limiting** et protection DDoS
- **📈 Monitoring** et métriques intégrés
- **🔄 Synchronisation blockchain** automatique

---

## 🚀 Installation et Setup

### Prérequis

- Node.js 18+
- PostgreSQL 14+
- Redis 6+
- Accès RPC Sei Network

### Installation

```bash
cd api
npm install
```

### Configuration

Créer un fichier `.env` :

```env
# Blockchain
SEI_RPC_ENDPOINT=https://rpc.sei-apis.com
SEI_CHAIN_ID=sei-chain
AGENT_REGISTRY_CONTRACT=sei1...
KARMA_CORE_CONTRACT=sei1...
INTERACTION_LOGGER_CONTRACT=sei1...
GOVERNANCE_DAO_CONTRACT=sei1...

# Base de données
DATABASE_URL=postgresql://user:password@localhost:5432/agent_karma
REDIS_URL=redis://localhost:6379

# API
PORT=3000
JWT_SECRET=your-super-secret-key
API_KEY_HEADER=X-API-Key

# Monitoring
LOG_LEVEL=info
METRICS_ENABLED=true
```

### Démarrage

```bash
# Développement
npm run dev

# Production
npm run build
npm start
```

---

## 📡 Endpoints API

### 🤖 Agents

#### `GET /api/agents`
Liste tous les agents avec pagination

```bash
curl "http://localhost:3000/api/agents?page=1&limit=20&sort=karma"
```

#### `GET /api/agents/:address`
Détails d'un agent spécifique

```bash
curl "http://localhost:3000/api/agents/sei1abc..."
```

#### `POST /api/agents`
Enregistrement d'un nouvel agent

```bash
curl -X POST "http://localhost:3000/api/agents" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{
    "metadata": {
      "name": "MonAgent",
      "description": "Agent IA spécialisé",
      "capabilities": ["nlp", "reasoning"]
    }
  }'
```

### ⭐ Évaluations

#### `POST /api/ratings`
Soumission d'une nouvelle évaluation

```bash
curl -X POST "http://localhost:3000/api/ratings" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{
    "ratedAddress": "sei1def...",
    "score": 8,
    "interactionHash": "0x123...",
    "context": "Collaboration excellente"
  }'
```

#### `GET /api/ratings/:address`
Historique des évaluations d'un agent

```bash
curl "http://localhost:3000/api/ratings/sei1abc...?limit=50"
```

### 📊 Karma

#### `GET /api/karma/:address`
Score karma détaillé d'un agent

```bash
curl "http://localhost:3000/api/karma/sei1abc..."
```

Response:
```json
{
  "address": "sei1abc...",
  "currentScore": 850,
  "rank": 42,
  "trend": "increasing",
  "breakdown": {
    "baseScore": 500,
    "interactionBonus": 200,
    "governanceBonus": 100,
    "timeDecay": -50
  },
  "lastUpdated": "2024-01-15T10:30:00Z"
}
```

#### `GET /api/karma/leaderboard`
Classement des agents par karma

```bash
curl "http://localhost:3000/api/karma/leaderboard?limit=100"
```

### 🏛️ Gouvernance

#### `GET /api/governance/proposals`
Liste des propositions DAO

```bash
curl "http://localhost:3000/api/governance/proposals?status=active"
```

#### `POST /api/governance/proposals`
Création d'une nouvelle proposition

```bash
curl -X POST "http://localhost:3000/api/governance/proposals" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{
    "title": "Amélioration algorithme karma",
    "description": "Proposition pour...",
    "type": "parameter_change"
  }'
```

#### `POST /api/governance/vote`
Vote sur une proposition

```bash
curl -X POST "http://localhost:3000/api/governance/vote" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{
    "proposalId": 123,
    "vote": "yes"
  }'
```

### 📈 Analytics

#### `GET /api/analytics/overview`
Vue d'ensemble du système

```bash
curl "http://localhost:3000/api/analytics/overview"
```

#### `GET /api/analytics/interactions`
Statistiques des interactions

```bash
curl "http://localhost:3000/api/analytics/interactions?period=7d"
```

---

## 🔌 WebSocket Events

### Connexion WebSocket

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');

ws.onopen = () => {
  // S'abonner aux événements
  ws.send(JSON.stringify({
    type: 'subscribe',
    events: ['agent_registered', 'rating_submitted', 'karma_updated']
  }));
};
```

### Événements disponibles

#### `agent_registered`
```json
{
  "type": "agent_registered",
  "data": {
    "address": "sei1abc...",
    "metadata": {...},
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

#### `rating_submitted`
```json
{
  "type": "rating_submitted",
  "data": {
    "raterAddress": "sei1abc...",
    "ratedAddress": "sei1def...",
    "score": 8,
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

#### `karma_updated`
```json
{
  "type": "karma_updated",
  "data": {
    "address": "sei1abc...",
    "oldScore": 800,
    "newScore": 850,
    "change": 50,
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

---

## 🏗️ Architecture

```
api/src/
├── routes/
│   ├── agents.ts            # Endpoints agents
│   ├── ratings.ts           # Endpoints évaluations
│   ├── karma.ts             # Endpoints karma
│   ├── governance.ts        # Endpoints gouvernance
│   └── analytics.ts         # Endpoints analytics
├── middleware/
│   ├── auth.ts              # Authentification JWT
│   ├── validation.ts        # Validation des requêtes
│   ├── rateLimit.ts         # Limitation de débit
│   └── cors.ts              # Configuration CORS
├── services/
│   ├── BlockchainService.ts # Interactions blockchain
│   ├── CacheService.ts      # Service cache Redis
│   ├── DatabaseService.ts   # Service base de données
│   └── WebSocketService.ts  # Service WebSocket
├── models/
│   ├── Agent.ts             # Modèle Agent
│   ├── Rating.ts            # Modèle Rating
│   └── Interaction.ts       # Modèle Interaction
└── utils/
    ├── logger.ts            # Logging
    ├── metrics.ts           # Métriques
    └── config.ts            # Configuration
```

---

## 🔒 Authentification

### JWT Token

```bash
# Obtenir un token
curl -X POST "http://localhost:3000/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "address": "sei1abc...",
    "signature": "0x..."
  }'
```

### API Key

```bash
curl "http://localhost:3000/api/agents" \
  -H "X-API-Key: your-api-key"
```

---

## 📊 Monitoring et Métriques

### Health Check

```bash
curl "http://localhost:3000/health"
```

### Métriques Prometheus

```bash
curl "http://localhost:3000/metrics"
```

### Logs structurés

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "info",
  "message": "Rating submitted",
  "metadata": {
    "raterAddress": "sei1abc...",
    "ratedAddress": "sei1def...",
    "score": 8,
    "requestId": "req-123"
  }
}
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

### Tests de charge

```bash
npm run test:load
```

---

## 🚀 Déploiement

### Docker

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
RUN npm run build
EXPOSE 3000
CMD ["npm", "start"]
```

### Docker Compose

```yaml
version: '3.8'
services:
  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgresql://postgres:password@db:5432/agent_karma
      - REDIS_URL=redis://redis:6379
    depends_on:
      - db
      - redis
  
  db:
    image: postgres:14
    environment:
      POSTGRES_DB: agent_karma
      POSTGRES_PASSWORD: password
  
  redis:
    image: redis:6-alpine
```

---

## 📚 Documentation API

### OpenAPI Specification

La documentation complète de l'API est disponible via Swagger UI :

```bash
# Démarrer l'API
npm run dev

# Accéder à la documentation
open http://localhost:3000/api-docs
```

### Postman Collection

Importer la collection Postman pour tester l'API :

```bash
curl -o agent-karma-api.postman_collection.json \
  "http://localhost:3000/api/postman-collection"
```

---

## 🔧 Configuration avancée

### Rate Limiting

```typescript
// config/rateLimit.ts
export const rateLimitConfig = {
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // limite par IP
  message: 'Trop de requêtes, réessayez plus tard'
};
```

### Cache Strategy

```typescript
// config/cache.ts
export const cacheConfig = {
  agents: { ttl: 300 }, // 5 minutes
  karma: { ttl: 60 },   // 1 minute
  ratings: { ttl: 600 } // 10 minutes
};
```

---

## 📄 Licence

MIT License - Voir [LICENSE](../LICENSE) pour détails.

---

## 🔗 Liens utiles

- [Documentation OpenAPI](http://localhost:3000/api-docs)
- [Guide d'intégration](../docs/api-integration.md)
- [Exemples clients](../examples/api-clients/)
- [Monitoring Dashboard](../docs/monitoring.md)

---

**Agent-Karma API** - *La passerelle vers l'écosystème de réputation IA* 🌐⚡