use super::champion_stats::ChampionStats;

#[derive(Debug, Clone)]
pub struct BanRecommendation {
    pub champion_name: String,
    pub score: f64,
    pub frequency: f64,
    pub win_rate: f64,
    pub times_faced: usize,
}

#[derive(Debug, Clone)]
pub struct AllyAnalysis {
    pub champion_name: String,
    pub times_played_together: usize,
    pub wins_together: usize,
    pub win_rate: f64,
}

impl BanRecommendation {
    pub fn new(
        champion_name: String,
        score: f64,
        frequency: f64,
        win_rate: f64,
        times_faced: usize,
    ) -> Self {
        BanRecommendation {
            champion_name,
            score,
            frequency,
            win_rate,
            times_faced,
        }
    }
}

impl AllyAnalysis {
    pub fn new(
        champion_name: String,
        times_played_together: usize,
        wins_together: usize,
    ) -> Self {
        let win_rate = if times_played_together == 0 {
            0.0
        } else {
            wins_together as f64 / times_played_together as f64
        };

        AllyAnalysis {
            champion_name,
            times_played_together,
            wins_together,
            win_rate,
        }
    }
}

pub struct BanRecommender;

impl BanRecommender {
    /// Calculate ban score based on:
    /// - 0.4 × frequency
    /// - 0.5 × (1 - win_rate)
    /// - 0.1 × recency (normalized to 0-1)
    pub fn calculate_score(
        stats: &ChampionStats,
        total_games: usize,
        max_recency: f64,
    ) -> f64 {
        let frequency = stats.frequency(total_games) / 100.0;
        let win_rate = stats.win_rate();
        let recency_normalized = if max_recency > 0.0 {
            stats.recency_score / max_recency
        } else {
            0.0
        };

        (0.4 * frequency) + (0.5 * (1.0 - win_rate)) + (0.1 * recency_normalized)
    }

    pub fn get_recommendations(
        stats: Vec<ChampionStats>,
        total_games: usize,
        top_n: usize,
    ) -> Vec<BanRecommendation> {
        let max_recency = stats
            .iter()
            .map(|s| s.recency_score)
            .fold(f64::NEG_INFINITY, f64::max);

        let mut recommendations: Vec<BanRecommendation> = stats
            .iter()
            .map(|s| {
                let score = Self::calculate_score(s, total_games, max_recency);
                let frequency = s.frequency(total_games);
                let win_rate = s.win_rate();
                BanRecommendation::new(
                    s.name.clone(),
                    score,
                    frequency,
                    win_rate,
                    s.times_faced,
                )
            })
            .collect();

        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        recommendations.truncate(top_n);

        recommendations
    }

    pub fn analyze_allies(
        ally_stats: Vec<ChampionStats>,
        min_games_together: usize,
    ) -> Vec<AllyAnalysis> {
        let mut analyses: Vec<AllyAnalysis> = ally_stats
            .iter()
            .filter(|s| s.times_faced >= min_games_together)
            .map(|s| {
                AllyAnalysis::new(
                    s.name.clone(),
                    s.times_faced,
                    s.wins_against,
                )
            })
            .collect();

        // Sort by win rate (worst first)
        analyses.sort_by(|a, b| a.win_rate.partial_cmp(&b.win_rate).unwrap_or(std::cmp::Ordering::Equal));

        analyses
    }
}
