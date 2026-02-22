use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedMatch {
    pub id: String,
    pub champion: String,
    pub won: bool,
    pub enemies: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedAccount {
    pub puuid: String,
    pub summoner_name: String,
    pub summoner_level: i32,
    pub cached_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatchCache {
    pub player: String,
    pub region: String,
    pub last_updated: DateTime<Utc>,
    pub matches: Vec<CachedMatch>,
    pub account: Option<CachedAccount>,
}

impl MatchCache {
    pub fn new(player: &str, region: &str) -> Self {
        MatchCache {
            player: player.to_string(),
            region: region.to_string(),
            last_updated: Utc::now(),
            matches: Vec::new(),
            account: None,
        }
    }

    pub fn set_account(&mut self, puuid: String, summoner_name: String, summoner_level: i32) {
        self.account = Some(CachedAccount {
            puuid,
            summoner_name,
            summoner_level,
            cached_at: Utc::now(),
        });
    }

    pub fn get_cached_account(&self) -> Option<CachedAccount> {
        self.account.clone()
    }

    pub fn get_cache_path(player: &str) -> PathBuf {
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".league_detect");

        let _ = fs::create_dir_all(&cache_dir);

        cache_dir.join(format!("{}.json", player.replace("#", "_")))
    }

    pub fn load(player: &str) -> Result<Self, AppError> {
        let path = Self::get_cache_path(player);

        match fs::read_to_string(&path) {
            Ok(content) => {
                serde_json::from_str(&content).map_err(|e| {
                    AppError::JsonError(format!("Failed to parse cache: {}", e))
                })
            }
            Err(_) => {
                // Cache doesn't exist yet, return empty
                Ok(MatchCache::new(player, "na1"))
            }
        }
    }

    pub fn save(&self) -> Result<(), AppError> {
        let path = Self::get_cache_path(&self.player);
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            AppError::JsonError(format!("Failed to serialize cache: {}", e))
        })?;

        fs::write(&path, json).map_err(|e| {
            AppError::JsonError(format!("Failed to write cache: {}", e))
        })?;

        Ok(())
    }

    pub fn add_matches(&mut self, new_matches: Vec<CachedMatch>) {
        // Remove duplicates and add new matches
        let existing_ids: std::collections::HashSet<_> =
            self.matches.iter().map(|m| m.id.clone()).collect();

        for new_match in new_matches {
            if !existing_ids.contains(&new_match.id) {
                self.matches.push(new_match);
            }
        }

        // Keep most recent matches first
        self.matches.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        self.last_updated = Utc::now();
    }

    pub fn get_recent_matches(&self, count: usize) -> Vec<CachedMatch> {
        self.matches.iter()
            .take(count)
            .cloned()
            .collect()
    }

    pub fn is_stale(&self, max_age_mins: u64) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.last_updated);
        age.num_minutes() > max_age_mins as i64
    }
}
