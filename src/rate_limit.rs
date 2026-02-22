use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc, Duration};
use crate::error::AppError;

// Riot API rate limits for development keys:
// - 20 requests per second
// - 100 requests per 2 minutes (120 seconds)
const MAX_REQUESTS_PER_2MIN: u32 = 100;
const MAX_REQUESTS_PER_SEC: u32 = 20;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestLog {
    pub player: String,
    pub requests_per_2min: u32,
    pub requests_per_sec: u32,
    pub last_request: DateTime<Utc>,
    pub window_2min_start: DateTime<Utc>,
    pub window_1sec_start: DateTime<Utc>,
}

impl RequestLog {
    pub fn new(player: &str) -> Self {
        let now = Utc::now();
        RequestLog {
            player: player.to_string(),
            requests_per_2min: 0,
            requests_per_sec: 0,
            last_request: now,
            window_2min_start: now,
            window_1sec_start: now,
        }
    }

    pub fn get_log_path(player: &str) -> PathBuf {
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".league_detect");

        let _ = fs::create_dir_all(&cache_dir);

        cache_dir.join(format!("{}.ratelimit.json", player.replace("#", "_")))
    }

    pub fn load(player: &str) -> Result<Self, AppError> {
        let path = Self::get_log_path(player);

        match fs::read_to_string(&path) {
            Ok(content) => {
                let mut log: RequestLog = serde_json::from_str(&content)
                    .map_err(|e| AppError::JsonError(format!("Failed to parse rate limit log: {}", e)))?;

                // Reset windows if time has passed
                let now = Utc::now();

                // Reset 2-minute window
                if now.signed_duration_since(log.window_2min_start).num_seconds() > 120 {
                    log.requests_per_2min = 0;
                    log.window_2min_start = now;
                }

                // Reset 1-second window
                if now.signed_duration_since(log.window_1sec_start).num_seconds() > 1 {
                    log.requests_per_sec = 0;
                    log.window_1sec_start = now;
                }

                Ok(log)
            }
            Err(_) => Ok(RequestLog::new(player)),
        }
    }

    pub fn save(&self) -> Result<(), AppError> {
        let path = Self::get_log_path(&self.player);
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::JsonError(format!("Failed to serialize rate limit log: {}", e)))?;

        fs::write(&path, json)
            .map_err(|e| AppError::JsonError(format!("Failed to write rate limit log: {}", e)))?;

        Ok(())
    }

    pub fn can_make_request(&self) -> bool {
        self.requests_per_2min < MAX_REQUESTS_PER_2MIN && self.requests_per_sec < MAX_REQUESTS_PER_SEC
    }

    pub fn record_request(&mut self) {
        self.requests_per_2min += 1;
        self.requests_per_sec += 1;
        self.last_request = Utc::now();
    }

    pub fn get_remaining(&self) -> (u32, u32) {
        (
            MAX_REQUESTS_PER_2MIN - self.requests_per_2min,
            MAX_REQUESTS_PER_SEC - self.requests_per_sec,
        )
    }

    pub fn get_reset_times(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        (
            self.window_2min_start + Duration::seconds(120),
            self.window_1sec_start + Duration::seconds(1),
        )
    }

    pub fn display_status(&self) {
        let (rem_2min, rem_sec) = self.get_remaining();
        let (reset_2min, reset_sec) = self.get_reset_times();
        let now = Utc::now();

        let time_2min = reset_2min.signed_duration_since(now);
        let time_sec = reset_sec.signed_duration_since(now);

        println!("\n📊 API Usage (Player: {})", self.player);
        println!("   Per 2 min: {}/{} requests (reset in {}s)",
            self.requests_per_2min, MAX_REQUESTS_PER_2MIN,
            time_2min.num_seconds().max(0));
        println!("   Per 1 sec: {}/{} requests (reset in {}ms)",
            self.requests_per_sec, MAX_REQUESTS_PER_SEC,
            time_sec.num_milliseconds().max(0));
        println!("   Status: {} ✅\n",
            if self.can_make_request() { "Ready" } else { "Rate Limited" }
        );
    }
}
