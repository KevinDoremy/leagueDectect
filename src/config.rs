use crate::error::AppError;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub region: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();

        let api_key = env::var("RIOT_API_KEY").map_err(|_| {
            AppError::ConfigError(
                "RIOT_API_KEY not found in .env file".to_string(),
            )
        })?;

        let region = env::var("RIOT_REGION").unwrap_or_else(|_| "na1".to_string());

        Ok(Config { api_key, region })
    }
}
