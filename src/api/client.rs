use crate::config::Config;
use crate::error::AppError;
use governor::{Quota, RateLimiter, state::{InMemoryState, NotKeyed}, clock::DefaultClock};
use std::num::NonZeroU32;
use std::thread;
use std::time::Duration;

use super::models::*;

pub struct RiotApiClient {
    config: Config,
    rate_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl RiotApiClient {
    pub fn new(config: Config) -> Self {
        // 20 requests per second rate limit
        let rate_limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(20).unwrap()));
        RiotApiClient {
            config,
            rate_limiter,
        }
    }

    fn get_regional_routing(&self) -> &str {
        match self.config.region.as_str() {
            "na1" | "br1" | "la1" | "la2" => "americas",
            "euw1" | "eun1" | "tr1" | "ru" => "europe",
            "kr" | "jp1" => "asia",
            "oc1" | "ph2" | "sg2" | "th2" | "vn2" => "sea",
            _ => "americas", // default
        }
    }

    fn execute_request(&self, url: &str) -> Result<String, AppError> {
        // Rate limiting - respect Riot API limits (20 req/sec, 100 req/2min)
        // Conservative approach: 150ms delay = ~6-7 req/sec
        thread::sleep(Duration::from_millis(150));

        let mut retry_count = 0;
        const MAX_RETRIES: u32 = 3;

        loop {
            let response = ureq::get(url)
                .set("User-Agent", "league_detect/0.1.0")
                .call();

            match response {
                Ok(resp) => {
                    return resp.into_string().map_err(|e| {
                        AppError::HttpError(e.to_string())
                    });
                }
                Err(ureq::Error::Status(429, _)) => {
                    // Rate limited - wait and retry
                    if retry_count >= MAX_RETRIES {
                        return Err(AppError::RateLimited);
                    }
                    let wait_ms = 2000 * (retry_count + 1) as u64;
                    println!("⏳ Rate limited, waiting {}ms before retry...", wait_ms);
                    thread::sleep(Duration::from_millis(wait_ms));
                    retry_count += 1;
                }
                Err(e) => {
                    return Err(AppError::HttpError(e.to_string()));
                }
            }
        }
    }

    pub fn get_account(&self, game_name: &str, tag_line: &str) -> Result<AccountDto, AppError> {
        let url = format!(
            "https://americas.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}?api_key={}",
            game_name, tag_line, self.config.api_key
        );

        let body = self.execute_request(&url)?;
        serde_json::from_str(&body).map_err(|_| {
            AppError::PlayerNotFound(format!("{}#{}", game_name, tag_line))
        })
    }

    pub fn get_summoner(&self, puuid: &str) -> Result<SummonerDto, AppError> {
        let url = format!(
            "https://{}.api.riotgames.com/lol/summoner/v4/summoners/by-puuid/{}?api_key={}",
            self.config.region, puuid, self.config.api_key
        );

        let body = self.execute_request(&url)?;
        serde_json::from_str(&body).map_err(|e| {
            AppError::JsonError(e.to_string())
        })
    }

    pub fn get_league_entry(&self, summoner_id: &str) -> Result<LeagueEntryDto, AppError> {
        let url = format!(
            "https://{}.api.riotgames.com/lol/league/v4/entries/by-summoner/{}?api_key={}",
            self.config.region, summoner_id, self.config.api_key
        );

        let body = self.execute_request(&url)?;
        serde_json::from_str(&body).map_err(|e| {
            AppError::JsonError(e.to_string())
        })
    }

    pub fn get_match_ids(&self, puuid: &str, count: usize) -> Result<Vec<String>, AppError> {
        let regional_routing = self.get_regional_routing();
        let url = format!(
            "https://{}.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?type=ranked&count={}&api_key={}",
            regional_routing, puuid, count, self.config.api_key
        );

        let body = self.execute_request(&url)?;
        serde_json::from_str(&body).map_err(|e| {
            AppError::JsonError(e.to_string())
        })
    }

    pub fn get_match(&self, match_id: &str) -> Result<MatchDto, AppError> {
        let regional_routing = self.get_regional_routing();
        let url = format!(
            "https://{}.api.riotgames.com/lol/match/v5/matches/{}?api_key={}",
            regional_routing, match_id, self.config.api_key
        );

        let body = self.execute_request(&url)?;
        serde_json::from_str(&body).map_err(|e| {
            AppError::JsonError(e.to_string())
        })
    }

    #[allow(dead_code)]
    pub fn get_champion_data(&self) -> Result<DataDragonChampions, AppError> {
        let url = "https://ddragon.leagueoflegends.com/cdn/14.25.1/data/en_US/champion.json";

        let body = ureq::get(url)
            .set("User-Agent", "league_detect/0.1.0")
            .call()
            .map_err(|e| AppError::HttpError(e.to_string()))?
            .into_string()
            .map_err(|e| AppError::HttpError(e.to_string()))?;

        serde_json::from_str(&body).map_err(|e| {
            AppError::JsonError(e.to_string())
        })
    }
}
