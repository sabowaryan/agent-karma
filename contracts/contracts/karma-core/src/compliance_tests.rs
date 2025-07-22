//! Tests unitaires pour le module de conformité et de détection d'abus

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{Addr, Uint128};
    use crate::compliance::*;
    use crate::state::{KARMA_SCORES, KarmaScore, RATE_LIMIT_TRACKERS};
    use crate::error::ContractError;

    #[test]
    fn test_spam_detection_high_frequency() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let agent = Addr::unchecked("agent1");
        
        // Simuler un agent avec un score de karma normal
        let karma_score = KarmaScore {
            current_score: Uint128::from(500u128),
            previous_score: Uint128::from(450u128),
            last_updated: env.block.time,
            total_ratings: 20,
            average_rating: "7.5".to_string(),
            interaction_count: 15,
        };
        KARMA_SCORES.save(deps.as_mut().storage, agent.as_str(), &karma_score).unwrap();
        
        // Tester la détection de spam avec une fréquence normale
        let result = detect_spam_ratings(deps.as_ref(), &env, &agent).unwrap();
        assert!(!result.is_suspicious);
        assert_eq!(result.violation_type, None);
        assert!(result.confidence_score < 0.5);
    }

    #[test]
    fn test_bot_behavior_detection() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let agent = Addr::unchecked("bot_agent");
        
        // Simuler un tracker de limite de taux avec un comportement de bot
        let tracker = RateLimitTracker {
            agent_address: agent.clone(),
            action_type: "rating".to_string(),
            count: BOT_BEHAVIOR_THRESHOLD + 10, // Dépasser le seuil
            window_start: env.block.time.minus_seconds(1800), // 30 minutes ago
            last_action: env.block.time,
        };
        RATE_LIMIT_TRACKERS.save(deps.as_mut().storage, agent.as_str(), &tracker).unwrap();
        
        // Tester la détection de comportement de bot
        let result = detect_bot_behavior(deps.as_ref(), &env, &agent).unwrap();
        assert!(result.is_suspicious);
        assert_eq!(result.violation_type, Some(ViolationType::BotBehavior));
        assert!(result.confidence_score >= 0.5);
        assert!(!result.evidence.is_empty());
    }

    #[test]
    fn test_rate_limiting_with_karma_multiplier() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let high_karma_agent = Addr::unchecked("high_karma_agent");
        let low_karma_agent = Addr::unchecked("low_karma_agent");
        
        // Agent avec un karma élevé
        let high_karma = KarmaScore {
            current_score: Uint128::from(1500u128), // > 1000, multiplier = 2.0
            previous_score: Uint128::from(1400u128),
            last_updated: env.block.time,
            total_ratings: 100,
            average_rating: "8.5".to_string(),
            interaction_count: 80,
        };
        KARMA_SCORES.save(deps.as_mut().storage, high_karma_agent.as_str(), &high_karma).unwrap();
        
        // Agent avec un karma faible
        let low_karma = KarmaScore {
            current_score: Uint128::from(50u128), // < 100, multiplier = 1.0
            previous_score: Uint128::from(45u128),
            last_updated: env.block.time,
            total_ratings: 5,
            average_rating: "6.0".to_string(),
            interaction_count: 3,
        };
        KARMA_SCORES.save(deps.as_mut().storage, low_karma_agent.as_str(), &low_karma).unwrap();
        
        // Tester la limite de taux pour l'agent avec un karma élevé
        let result_high = check_rate_limit(deps.as_mut(), &env, &high_karma_agent, "rating").unwrap();
        assert!(result_high); // Devrait passer car limite plus élevée
        
        // Tester la limite de taux pour l'agent avec un karma faible
        let result_low = check_rate_limit(deps.as_mut(), &env, &low_karma_agent, "rating").unwrap();
        assert!(result_low); // Devrait passer pour la première action
    }

    #[test]
    fn test_dispute_creation_insufficient_karma() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let challenger = Addr::unchecked("challenger");
        
        // Agent avec un karma insuffisant pour le stake
        let low_karma = KarmaScore {
            current_score: Uint128::from(50u128),
            previous_score: Uint128::from(45u128),
            last_updated: env.block.time,
            total_ratings: 5,
            average_rating: "6.0".to_string(),
            interaction_count: 3,
        };
        KARMA_SCORES.save(deps.as_mut().storage, challenger.as_str(), &low_karma).unwrap();
        
        // Tenter de créer une dispute avec un stake trop élevé
        let result = create_dispute(
            deps.as_mut(),
            &env,
            &challenger,
            "violation_123".to_string(),
            Uint128::from(100u128), // Plus que le karma disponible
            "Evidence of false positive".to_string(),
        );
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ContractError::InsufficientKarma { required, current } => {
                assert_eq!(required, 100);
                assert_eq!(current, 50);
            }
            _ => panic!("Expected InsufficientKarma error"),
        }
    }

    #[test]
    fn test_dispute_creation_success() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let challenger = Addr::unchecked("challenger");
        
        // Agent avec un karma suffisant
        let sufficient_karma = KarmaScore {
            current_score: Uint128::from(200u128),
            previous_score: Uint128::from(180u128),
            last_updated: env.block.time,
            total_ratings: 20,
            average_rating: "7.5".to_string(),
            interaction_count: 15,
        };
        KARMA_SCORES.save(deps.as_mut().storage, challenger.as_str(), &sufficient_karma).unwrap();
        
        // Créer une dispute avec succès
        let result = create_dispute(
            deps.as_mut(),
            &env,
            &challenger,
            "violation_123".to_string(),
            Uint128::from(50u128),
            "Evidence of false positive detection".to_string(),
        );
        
        assert!(result.is_ok());
        let case_id = result.unwrap();
        assert!(case_id.starts_with("dispute_violation_123_"));
        
        // Vérifier que le karma a été déduit
        let updated_karma = KARMA_SCORES.load(deps.as_ref().storage, challenger.as_str()).unwrap();
        assert_eq!(updated_karma.current_score, Uint128::from(150u128)); // 200 - 50
    }

    #[test]
    fn test_dispute_resolution_overturned() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let challenger = Addr::unchecked("challenger");
        
        // Configurer un karma initial
        let initial_karma = KarmaScore {
            current_score: Uint128::from(100u128), // Après déduction du stake
            previous_score: Uint128::from(150u128),
            last_updated: env.block.time,
            total_ratings: 20,
            average_rating: "7.5".to_string(),
            interaction_count: 15,
        };
        KARMA_SCORES.save(deps.as_mut().storage, challenger.as_str(), &initial_karma).unwrap();
        
        // Créer une dispute case manuellement
        let case_id = "dispute_test_case";
        let dispute_case = DisputeCase {
            case_id: case_id.to_string(),
            violation_id: "violation_123".to_string(),
            challenger: challenger.clone(),
            stake_amount: Uint128::from(50u128),
            evidence: "False positive evidence".to_string(),
            status: DisputeStatus::Pending,
            created_at: env.block.time,
            resolved_at: None,
            resolution: None,
        };
        crate::state::DISPUTE_CASES.save(deps.as_mut().storage, case_id, &dispute_case).unwrap();
        
        // Résoudre la dispute en faveur du challenger
        let result = resolve_dispute(
            deps.as_mut(),
            &env,
            case_id,
            DisputeResolution::ViolationOverturned,
        );
        
        assert!(result.is_ok());
        
        // Vérifier que le karma a été restauré
        let updated_karma = KARMA_SCORES.load(deps.as_ref().storage, challenger.as_str()).unwrap();
        assert_eq!(updated_karma.current_score, Uint128::from(150u128)); // 100 + 50 (stake retourné)
        
        // Vérifier que la dispute est marquée comme résolue
        let resolved_case = crate::state::DISPUTE_CASES.load(deps.as_ref().storage, case_id).unwrap();
        assert_eq!(resolved_case.status, DisputeStatus::Resolved);
        assert_eq!(resolved_case.resolution, Some(DisputeResolution::ViolationOverturned));
    }

    #[test]
    fn test_calculate_variance() {
        // Test avec des scores identiques (variance faible)
        let identical_scores = vec![5, 5, 5, 5, 5];
        let variance = calculate_variance(&identical_scores);
        assert!(variance < 0.1);
        
        // Test avec des scores variés (variance élevée)
        let varied_scores = vec![1, 3, 5, 7, 9];
        let variance = calculate_variance(&varied_scores);
        assert!(variance > 2.0);
        
        // Test avec un échantillon trop petit
        let small_sample = vec![5];
        let variance = calculate_variance(&small_sample);
        assert_eq!(variance, 1.0); // Valeur par défaut
    }

    #[test]
    fn test_comprehensive_abuse_detection() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let agent = Addr::unchecked("test_agent");
        
        // Configurer un agent normal
        let karma_score = KarmaScore {
            current_score: Uint128::from(300u128),
            previous_score: Uint128::from(280u128),
            last_updated: env.block.time,
            total_ratings: 15,
            average_rating: "7.0".to_string(),
            interaction_count: 12,
        };
        KARMA_SCORES.save(deps.as_mut().storage, agent.as_str(), &karma_score).unwrap();
        
        // Exécuter la détection d'abus complète
        let results = run_abuse_detection(deps.as_ref(), &env, &agent).unwrap();
        
        // Devrait retourner 3 résultats (spam, bot, manipulation)
        assert_eq!(results.len(), 3);
        
        // Tous les résultats devraient être non suspects pour un agent normal
        for result in results {
            assert!(!result.is_suspicious);
            assert!(result.confidence_score < 0.5);
        }
    }

    #[test]
    fn test_penalty_application() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let agent = Addr::unchecked("violator");
        
        // Configurer un agent avec du karma
        let initial_karma = KarmaScore {
            current_score: Uint128::from(500u128),
            previous_score: Uint128::from(480u128),
            last_updated: env.block.time.minus_seconds(3600),
            total_ratings: 25,
            average_rating: "7.5".to_string(),
            interaction_count: 20,
        };
        KARMA_SCORES.save(deps.as_mut().storage, agent.as_str(), &initial_karma).unwrap();
        
        // Créer une violation
        let violation = ComplianceViolation {
            agent_address: agent.clone(),
            violation_type: ViolationType::SpamRating,
            severity: 7,
            timestamp: env.block.time,
            evidence: "High frequency rating pattern detected".to_string(),
            penalty_applied: Uint128::from(70u128),
            disputed: false,
        };
        
        // Appliquer la pénalité
        let result = apply_abuse_penalty(deps.as_mut(), &env, &agent, &violation);
        assert!(result.is_ok());
        
        // Vérifier que le karma a été réduit
        let updated_karma = KARMA_SCORES.load(deps.as_ref().storage, agent.as_str()).unwrap();
        assert_eq!(updated_karma.current_score, Uint128::from(430u128)); // 500 - 70
        assert_eq!(updated_karma.last_updated, env.block.time);
        
        // Vérifier que la pénalité a été enregistrée
        let penalty_key = format!("{}:{}", agent, env.block.time.seconds());
        let recorded_penalty = crate::state::KARMA_PENALTIES.load(deps.as_ref().storage, &penalty_key).unwrap();
        assert_eq!(recorded_penalty, Uint128::from(70u128));
    }
}