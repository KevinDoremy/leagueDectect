use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ChampionStats {
    pub name: String,
    pub times_faced: usize,
    pub wins_against: usize,
    pub recency_score: f64, // weighted by match index
}

impl ChampionStats {
    pub fn new(name: String) -> Self {
        ChampionStats {
            name,
            times_faced: 0,
            wins_against: 0,
            recency_score: 0.0,
        }
    }

    pub fn win_rate(&self) -> f64 {
        if self.times_faced == 0 {
            0.0
        } else {
            self.wins_against as f64 / self.times_faced as f64
        }
    }

    pub fn frequency(&self, total_games: usize) -> f64 {
        if total_games == 0 {
            0.0
        } else {
            (self.times_faced as f64 / total_games as f64) * 100.0
        }
    }
}

pub struct ChampionStatsTracker {
    stats: HashMap<String, ChampionStats>,
}

impl ChampionStatsTracker {
    pub fn new() -> Self {
        ChampionStatsTracker {
            stats: HashMap::new(),
        }
    }

    pub fn add_champion_encounter(
        &mut self,
        champion_name: String,
        won_against: bool,
        recency_weight: f64,
    ) {
        let entry = self.stats.entry(champion_name.clone()).or_insert_with(|| {
            ChampionStats::new(champion_name)
        });

        entry.times_faced += 1;
        if won_against {
            entry.wins_against += 1;
        }
        entry.recency_score += recency_weight;
    }

    pub fn get_stats(&self) -> Vec<ChampionStats> {
        self.stats.values().cloned().collect()
    }

    #[allow(dead_code)]
    pub fn get_champion(&self, name: &str) -> Option<ChampionStats> {
        self.stats.get(name).cloned()
    }
}
