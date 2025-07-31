# Tâches conçues pour Agent-Karma

Ce document détaille les tâches de développement dérivées des exigences fonctionnelles et non fonctionnelles du projet Agent-Karma. Chaque tâche est conçue pour être une unité de travail actionable, contribuant directement à la réalisation des objectifs du système de réputation décentralisé pour agents IA sur la blockchain Sei.

## Tâches basées sur l'Exigence 1 : Enregistrement d'identité d'agent

### Tâche 1.1 : Développement du Smart Contract AgentRegistry

**Description :** Développer le smart contract `AgentRegistry` en Solidity qui gérera l'enregistrement des identités des agents IA sur la blockchain Sei. Ce contrat doit inclure une fonction `registerAgent` qui prend en paramètre les métadonnées de l'agent et crée une identité unique associée à son adresse de portefeuille. Le score de karma initial de l'agent doit être défini à zéro lors de l'enregistrement.

**Critères d'Acceptation :**
- Le contrat `AgentRegistry` est déployable sur Sei.
- La fonction `registerAgent` crée une nouvelle entrée pour un agent non enregistré.
- Le score de karma de l'agent est initialisé à 0 après l'enregistrement.
- La fonction `registerAgent` rejette les tentatives d'enregistrement avec une identité existante.
- Des tests unitaires sont écrits pour valider le comportement du contrat.

### Tâche 1.2 : Intégration SDK pour l'enregistrement d'agent

**Description :** Mettre à jour le SDK TypeScript (`@agent-karma/sdk`) pour inclure une méthode `registerAgent` qui interagit avec le smart contract `AgentRegistry`. Cette méthode doit encapsuler la logique d'appel au contrat et gérer la confirmation de l'enregistrement pour l'agent.

**Critères d'Acceptation :**
- La méthode `sdk.registerAgent` est disponible et fonctionnelle.
- L'appel à `sdk.registerAgent` déclenche correctement l'enregistrement sur la blockchain.
- L'SDK gère les réponses de confirmation et les erreurs de doublon d'identité.
- Des exemples d'utilisation sont fournis dans la documentation de l'SDK.

## Tâches basées sur l'Exigence 2 : Évaluation des agents

### Tâche 2.1 : Développement de la fonction de soumission de notation dans KarmaCore

**Description :** Ajouter une fonction au smart contract `KarmaCore` qui permet aux agents de soumettre une notation pour d'autres agents après une interaction. Cette fonction doit valider le score (entre 1 et 10) et stocker la notation de manière immuable sur la blockchain, en incluant un horodatage et des métadonnées optionnelles.

**Critères d'Acceptation :**
- La fonction de soumission de notation est implémentée dans `KarmaCore`.
- La fonction valide que le score est dans la plage 1-10.
- La notation est stockée avec un horodatage et l'identifiant de l'interaction.
- La fonction rejette les tentatives de notation en double pour la même interaction.
- Des tests unitaires couvrent les scénarios de soumission valide et invalide.

### Tâche 2.2 : Intégration SDK pour la soumission de notation

**Description :** Étendre le SDK TypeScript pour inclure une méthode `submitRating` qui permet aux agents de soumettre des évaluations via le smart contract `KarmaCore`. Cette méthode doit faciliter la soumission du score, de l'identifiant de l'interaction et des commentaires optionnels.

**Critères d'Acceptation :**
- La méthode `sdk.submitRating` est disponible et fonctionnelle.
- L'appel à `sdk.submitRating` interagit correctement avec le smart contract.
- L'SDK gère les validations de score et les rejets de doublons.
- Des exemples d'utilisation sont fournis.

## Tâches basées sur l'Exigence 3 : Calcul transparent du score de karma

### Tâche 3.1 : Implémentation de l'algorithme de calcul de karma dans KarmaCore

**Description :** Développer l'algorithme de calcul du score de karma au sein du smart contract `KarmaCore`. Cet algorithme doit être publiquement vérifiable et déclenché automatiquement lors de la réception de nouvelles notations. L'objectif est de recalculer le karma d'un agent en moins de 400ms.

**Critères d'Acceptation :**
- L'algorithme de calcul de karma est implémenté et vérifiable.
- Le recalcul du karma est déclenché automatiquement après une nouvelle notation.
- Le temps de recalcul est inférieur à 400ms sur le testnet Sei.
- Des événements sont émis avec le nouveau score et les détails du calcul.
- Des tests de performance sont effectués pour valider la latence.

### Tâche 3.2 : Gestion des erreurs de calcul de karma

**Description :** Implémenter des mécanismes de gestion des erreurs dans `KarmaCore` pour les cas où le calcul du karma échoue. En cas d'échec, le système doit maintenir le score précédent de l'agent et enregistrer l'erreur pour audit.

**Critères d'Acceptation :**
- Le score de karma précédent est conservé en cas d'échec de calcul.
- Les erreurs de calcul sont loggées de manière appropriée.
- Des tests simulent des échecs de calcul et vérifient la résilience du système.

## Tâches basées sur l'Exigence 4 : Requête du score de karma

### Tâche 4.1 : Développement de la fonction de requête de karma dans KarmaCore

**Description :** Ajouter une fonction au smart contract `KarmaCore` qui permet de requêter le score de karma actuel d'un agent, le nombre d'interactions et l'horodatage de la dernière mise à jour. Cette fonction doit être optimisée pour une réponse rapide.

**Critères d'Acceptation :**
- La fonction de requête de karma est implémentée et retourne les informations requises.
- Le temps de réponse est inférieur à 400ms.
- La fonction gère les requêtes pour des agents inexistants en retournant un message d'erreur approprié.

### Tâche 4.2 : Intégration SDK pour la requête de karma et l'historique

**Description :** Étendre le SDK TypeScript pour inclure des méthodes permettant de requêter le score de karma actuel d'un agent (`getKarmaScore`) et son historique chronologique (`getKarmaHistory`).

**Critères d'Acceptation :**
- Les méthodes `sdk.getKarmaScore` et `sdk.getKarmaHistory` sont disponibles.
- `getKarmaScore` retourne les informations correctes.
- `getKarmaHistory` fournit l'évolution chronologique du score.
- L'SDK gère les erreurs pour les agents inexistants.

## Tâches basées sur l'Exigence 5 : Tableau de bord de réputation

### Tâche 5.1 : Développement de l'API Backend pour le tableau de bord

**Description :** Développer les endpoints de l'API REST (`api/src/routes`) qui fourniront les données nécessaires au tableau de bord. Cela inclut des endpoints pour les agents les mieux notés, les interactions récentes, les détails d'un agent spécifique (tendances de karma, historique, notations reçues) et le filtrage.

**Critères d'Acceptation :**
- Les endpoints de l'API sont fonctionnels et retournent les données structurées.
- Les requêtes sont optimisées pour une réponse rapide.
- Les données incluent les agents les mieux notés, les interactions récentes et les détails d'agent.
- Le filtrage par score, interactions et date d'enregistrement est supporté.

### Tâche 5.2 : Développement du Frontend du tableau de bord React

**Description :** Développer l'interface utilisateur du tableau de bord (`dashboard/src`) en utilisant React et TypeScript. Le tableau de bord doit afficher les informations fournies par l'API backend, permettre le filtrage et la visualisation des tendances de karma.

**Critères d'Acceptation :**
- Le tableau de bord affiche correctement les agents les mieux notés et les interactions.
- La page de détails d'un agent montre les tendances de karma et l'historique.
- Les fonctionnalités de filtrage et de tri sont implémentées et fonctionnelles.
- Le tableau de bord se charge et affiche les données initiales en moins de 2 secondes.
- Le design est responsive et convivial.

## Tâches basées sur l'Exigence 6 : Auditabilité des interactions

### Tâche 6.1 : Développement du Smart Contract InteractionLogger

**Description :** Développer le smart contract `InteractionLogger` qui enregistrera de manière immuable toutes les interactions des agents sur la blockchain Sei. Chaque log doit inclure un horodatage, les participants et le type d'interaction.

**Critères d'Acceptation :**
- Le contrat `InteractionLogger` est déployable et fonctionnel.
- Toutes les interactions sont loggées avec les informations requises.
- Les logs sont immuables et accessibles pour audit.
- Des tests unitaires valident l'enregistrement des logs.

### Tâche 6.2 : Intégration du Logger dans les flux d'interaction

**Description :** Intégrer le `InteractionLogger` dans les flux d'interaction clés du système (enregistrement, soumission de notation, etc.) pour garantir que toutes les actions sont auditables. Implémenter une logique de re-tentative en cas d'échec de stockage sur la blockchain.

**Critères d'Acceptation :**
- Le logger est appelé à chaque interaction pertinente.
- La logique de re-tentative (jusqu'à 3 fois) est implémentée en cas d'échec.
- Des alertes sont générées si le stockage échoue après les re-tentatives.
- Des tests d'intégration valident l'auditabilité complète.

## Tâches basées sur l'Exigence 7 : Intégration de données externes (Oracles)

### Tâche 7.1 : Intégration de l'Oracle Rivalz

**Description :** Implémenter l'intégration avec l'oracle Rivalz pour récupérer des données externes qui peuvent enrichir les calculs de karma. Cela implique le développement de connecteurs et de mécanismes de vérification de l'authenticité des données.

**Critères d'Acceptation :**
- L'intégration avec l'oracle Rivalz est fonctionnelle.
- Les données de l'oracle sont vérifiées avant utilisation dans le calcul de karma.
- Le système continue de fonctionner avec les données on-chain si l'intégration oracle échoue.
- Les données on-chain sont prioritaires en cas de conflit avec les données oracle.

### Tâche 7.2 : Mise à jour de KarmaCore pour utiliser les données Oracle

**Description :** Modifier le smart contract `KarmaCore` pour incorporer les données provenant de l'oracle dans l'algorithme de calcul de karma, en tenant compte des priorités et des mécanismes de fallback.

**Critères d'Acceptation :**
- L'algorithme de karma utilise les données oracle si disponibles et vérifiées.
- La logique de fallback aux données on-chain est correcte.
- Des tests valident l'impact des données oracle sur le karma.

## Tâches basées sur l'Exigence 8 : Compatibilité avec les frameworks IA

### Tâche 8.1 : Développement du Plugin ElizaOS

**Description :** Développer un plugin natif pour ElizaOS qui permettra aux agents basés sur ce framework d'interagir facilement avec le système Agent-Karma (enregistrement, notation, requête de karma).

**Critères d'Acceptation :**
- Le plugin ElizaOS est fonctionnel et facile à intégrer.
- Les agents ElizaOS peuvent effectuer les opérations clés de karma.
- Des exemples d'utilisation du plugin sont fournis.

### Tâche 8.2 : Intégration MCP et AIDN

**Description :** Exposer les composants modulaires du système Agent-Karma pour l'interopérabilité avec le protocole MCP et développer un connecteur pour le réseau AIDN, assurant une compatibilité étendue avec les frameworks IA.

**Critères d'Acceptation :**
- Les composants MCP sont exposés et documentés.
- Le connecteur AIDN est développé et permet l'interconnexion.
- Le système fournit un accès API REST de fallback si les intégrations directes échouent.
- Des tests d'intégration sont effectués avec des agents MCP et AIDN.

