use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("API error: {0}")]
    #[allow(dead_code)]
    ApiError(String),

    #[error("Rate limit exceeded, please try again later")]
    RateLimited,

    #[error("Invalid Riot ID format. Use format: Name#TAG")]
    #[allow(dead_code)]
    InvalidRiotId,

    #[error("Player not found: {0}")]
    PlayerNotFound(String),

    #[error("No ranked games found for this player")]
    NoRankedGames,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("JSON parsing error: {0}")]
    JsonError(String),
}
