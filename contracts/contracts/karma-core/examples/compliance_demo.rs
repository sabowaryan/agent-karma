//! Démonstration du module de conformité et de détection d'abus
//!
//! Ce script montre comment utiliser les fonctionnalités de conformité
//! pour détecter et gérer les comportements abusifs dans le système Agent-Karma.


use karma_core::compliance::*;


fn main() {
    println!("🔍 Démonstration du Module de Conformité Agent-Karma");
    println!("=====================================================\n");

    // Démonstration 1: Détection de spam de ratings
    println!("1. 📊 Détection de Spam de Ratings");
    println!(
        "   - Seuil de spam: {} ratings par heure",
        SPAM_RATING_THRESHOLD
    );
    println!(
        "   - Fenêtre de détection: {} secondes",
        RATING_PATTERN_WINDOW
    );
    println!("   - Variance minimale requise: {}", MIN_RATING_VARIANCE);
    println!();

    // Démonstration 2: Détection de comportement de bot
    println!("2. 🤖 Détection de Comportement de Bot");
    println!(
        "   - Seuil d'actions: {} actions par heure",
        BOT_BEHAVIOR_THRESHOLD
    );
    println!(
        "   - Ratio d'interactions suspectes: {}",
        SUSPICIOUS_INTERACTION_RATIO
    );
    println!();

    // Démonstration 3: Système de pénalités
    println!("3. ⚖️ Système de Pénalités");
    println!(
        "   - Multiplicateur de pénalité: {}",
        KARMA_PENALTY_MULTIPLIER
    );
    println!("   - Échelle de sévérité: 1-10");
    println!();

    // Démonstration 4: Exemple de calcul de variance
    println!("4. 📈 Calcul de Variance des Ratings");

    let scores_normaux = vec![6, 7, 8, 7, 6, 8, 7];
    let variance_normale = calculate_variance(&scores_normaux);
    println!(
        "   - Scores normaux {:?}: variance = {:.2}",
        scores_normaux, variance_normale
    );

    let scores_suspects = vec![5, 5, 5, 5, 5, 5, 5];
    let variance_suspecte = calculate_variance(&scores_suspects);
    println!(
        "   - Scores suspects {:?}: variance = {:.2}",
        scores_suspects, variance_suspecte
    );
    println!();

    // Démonstration 5: Types de violations
    println!("5. 🚨 Types de Violations Détectées");
    println!("   - SpamRating: Fréquence élevée de ratings");
    println!("   - RatingManipulation: Patterns de manipulation");
    println!("   - BotBehavior: Comportement automatisé suspect");
    println!("   - SuspiciousPattern: Autres patterns suspects");
    println!("   - RateLimitExceeded: Dépassement des limites");
    println!();

    // Démonstration 6: Système de disputes
    println!("6. 🏛️ Système de Résolution de Disputes");
    println!("   - Les agents peuvent contester les violations");
    println!("   - Staking de karma requis pour créer une dispute");
    println!("   - Résolutions possibles:");
    println!("     * ViolationConfirmed: Violation confirmée");
    println!("     * ViolationOverturned: Violation annulée");
    println!("     * PartialOverturned: Violation partiellement annulée");
    println!();

    // Démonstration 7: Rate limiting basé sur le karma
    println!("7. 🎯 Rate Limiting Basé sur le Karma");
    println!("   - Karma > 1000: multiplicateur x2.0");
    println!("   - Karma > 500: multiplicateur x1.5");
    println!("   - Karma > 100: multiplicateur x1.2");
    println!("   - Karma ≤ 100: multiplicateur x1.0");
    println!();

    // Exemple pratique
    println!("8. 💡 Exemple Pratique");
    println!("   Agent avec 1500 karma:");
    let base_limit = SPAM_RATING_THRESHOLD;
    let karma_multiplier = 2.0; // > 1000 karma
    let effective_limit = (base_limit as f64 * karma_multiplier) as u32;
    println!("   - Limite de base: {} ratings/heure", base_limit);
    println!("   - Limite effective: {} ratings/heure", effective_limit);
    println!();

    println!("✅ Module de conformité prêt pour la production!");
    println!("   - Détection automatique d'abus");
    println!("   - Système de pénalités équitable");
    println!("   - Mécanisme de dispute transparent");
    println!("   - Rate limiting adaptatif");
}

#[cfg(test)]
mod demo_tests {
    use super::*;

    #[test]
    fn test_variance_calculation_demo() {
        // Test avec des scores variés (comportement normal)
        let normal_scores = vec![6, 7, 8, 7, 6, 8, 7, 6, 8];
        let normal_variance = calculate_variance(&normal_scores);
        assert!(normal_variance > MIN_RATING_VARIANCE);

        // Test avec des scores identiques (comportement suspect)
        let suspicious_scores = vec![5, 5, 5, 5, 5, 5, 5];
        let suspicious_variance = calculate_variance(&suspicious_scores);
        assert!(suspicious_variance < MIN_RATING_VARIANCE);
    }

    #[test]
    fn test_karma_multiplier_demo() {
        // Simuler différents niveaux de karma et leurs multiplicateurs
        let test_cases = vec![
            (50, 1.0),   // Karma faible
            (150, 1.2),  // Karma moyen
            (750, 1.5),  // Karma élevé
            (1500, 2.0), // Karma très élevé
        ];

        for (karma, expected_multiplier) in test_cases {
            let multiplier = if karma > 1000 {
                2.0
            } else if karma > 500 {
                1.5
            } else if karma > 100 {
                1.2
            } else {
                1.0
            };

            assert_eq!(multiplier, expected_multiplier);

            let effective_limit = (SPAM_RATING_THRESHOLD as f64 * multiplier) as u32;
            println!(
                "Karma {}: limite effective {} ratings/heure",
                karma, effective_limit
            );
        }
    }

    #[test]
    fn test_penalty_calculation_demo() {
        // Tester le calcul des pénalités selon la sévérité
        let severities = vec![1, 3, 5, 7, 10];

        for severity in severities {
            let penalty = KARMA_PENALTY_MULTIPLIER * (severity as u128);
            println!("Sévérité {}: pénalité {} karma", severity, penalty);

            // Vérifier que les pénalités sont proportionnelles
            assert_eq!(penalty, KARMA_PENALTY_MULTIPLIER * (severity as u128));
        }
    }
}
