use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc, Duration};
use crate::error::AppError;

const MAX_REQUESTS_PER_DAY: u32 = 50;
const MAX_REQUESTS_PER_HOUR: u32 = 20;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestLog {
    pub player: String,
    pub requests_today: u32,
    pub requests_this_hour: u32,
    pub last_request: DateTime<Utc>,
    pub day_reset: DateTime<Utc>,
    pub hour_reset: DateTime<Utc>,
}

impl RequestLog {
    pub fn new(player: &str) -> Self {
        let now = Utc::now();
        RequestLog {
            player: player.to_string(),
            requests_today: 0,
            requests_this_hour: 0,
            last_request: now,
            day_reset: now + Duration::days(1),
            hour_reset: now + Duration::hours(1),
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

                // Reset if day has passed
                let now = Utc::now();
                if now > log.day_reset {
                    log.requests_today = 0;
                    log.day_reset = now + Duration::days(1);
                }

                // Reset if hour has passed
                if now > log.hour_reset {
                    log.requests_this_hour = 0;
                    log.hour_reset = now + Duration::hours(1);
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
        self.requests_today < MAX_REQUESTS_PER_DAY && self.requests_this_hour < MAX_REQUESTS_PER_HOUR
    }

    pub fn record_request(&mut self) {
        self.requests_today += 1;
        self.requests_this_hour += 1;
        self.last_request = Utc::now();
    }

    pub fn get_remaining(&self) -> u32 {
        (MAX_REQUESTS_PER_DAY - self.requests_today).min(MAX_REQUESTS_PER_HOUR - self.requests_this_hour)
    }

    pub fn get_reset_time(&self) -> DateTime<Utc> {
        self.day_reset.min(self.hour_reset)
    }

    pub fn display_status(&self) {
        let remaining = self.get_remaining();
        let reset_time = self.get_reset_time();
        let time_until_reset = reset_time.signed_duration_since(Utc::now());

        println!("\n📊 API Usage (Player: {})", self.player);
        println!("   Daily:  {}/{} requests", self.requests_today, MAX_REQUESTS_PER_DAY);
        println!("   Hourly: {}/{} requests", self.requests_this_hour, MAX_REQUESTS_PER_HOUR);
        println!("   Remaining: {} requests today", remaining);
        println!("   Reset in: {}h {}m\n",
            time_until_reset.num_hours(),
            time_until_reset.num_minutes() % 60
        );
    }
}
