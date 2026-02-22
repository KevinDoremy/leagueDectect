mod analysis;
mod api;
mod cache;
mod config;
mod display;
mod error;
mod rate_limit;

use analysis::champion_stats::ChampionStatsTracker;
use analysis::recommender::BanRecommender;
use api::client::RiotApiClient;
use clap::Parser;
use config::Config;
use display::output::{display_ban_recommendations, display_error, display_info, display_success, display_match_history, display_ally_analysis};
use error::AppError;
use indicatif::ProgressBar;

#[derive(Debug, Clone)]
struct MatchResult {
    match_number: usize,
    player_champion: String,
    won: bool,
    enemy_champions: Vec<String>,
}

#[derive(Parser, Debug)]
#[command(name = "League Detect")]
#[command(about = "Analyze ranked games and get ban recommendations", long_about = None)]
struct Args {
    /// Riot Game Name
    game_name: String,

    /// Riot Tag (tag line)
    tag_line: String,

    /// Region (default: na1)
    #[arg(short, long)]
    region: Option<String>,

    /// Number of top bans to display (default: 5)
    #[arg(short, long, default_value = "5")]
    top_n: usize,

    /// Number of matches to analyze (default: 20, max: 200)
    #[arg(short, long, default_value = "20")]
    matches: usize,

    /// Skip first N matches (offset from most recent)
    /// Example: --offset 50 --matches 50 analyzes matches 50-100 in the past
    #[arg(long, default_value = "0")]
    offset: usize,

    /// Force refresh from Riot API (ignore cache)
    #[arg(long)]
    refresh: bool,
}

fn main() {
    let args = Args::parse();

    if let Err(e) = run(args) {
        display_error(&e.to_string());
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<(), AppError> {
    // Load configuration
    let mut config = Config::from_env()?;
    if let Some(region) = args.region {
        config.region = region;
    }

    let player_key = format!("{}#{}", args.game_name, args.tag_line);

    // Load rate limit tracker
    let mut rate_limiter = rate_limit::RequestLog::load(&player_key)?;

    // Check if we can make requests
    if !args.refresh && rate_limiter.can_make_request() {
        // OK - we have requests remaining
    } else if args.refresh && !rate_limiter.can_make_request() {
        rate_limiter.display_status();
        return Err(AppError::ApiError(
            format!("⛔ API limit reached (50 req/day, 20 req/hour). Try again in ~1 hour or tomorrow."),
        ));
    }

    display_info(&format!(
        "Fetching data for {} in region {}",
        player_key, config.region
    ));

    let client = RiotApiClient::new(config.clone());

    // Step 1: Get account info (PUUID)
    display_info("Step 1: Getting account info...");
    let account = client.get_account(&args.game_name, &args.tag_line)?;
    display_success(&format!("Found PUUID: {}", &account.puuid[0..8]));

    // Step 2: Get summoner info
    display_info("Step 2: Getting summoner info...");
    let summoner = client.get_summoner(&account.puuid)?;
    display_success(&format!("Summoner Level: {}", summoner.summoner_level));

    // Step 3: Get rank info (optional - for context)
    display_info("Step 3: Getting rank info...");
    display_success(&format!(
        "Summoner Level: {}",
        summoner.summoner_level
    ));

    // Step 4: Get match IDs (with caching)
    let player_key = format!("{}#{}", args.game_name, args.tag_line);
    let region = config.region.clone();
    let mut match_cache = cache::MatchCache::load(&player_key).ok();

    let has_cache = match_cache.as_ref().map(|c| !c.matches.is_empty()).unwrap_or(false);

    let mut all_match_ids = if has_cache && !args.refresh {
        // Smart cache: check online for new matches (IDs only - fast!)
        display_info("Step 4: Checking for new matches online...");
        let matches_count = std::cmp::min(args.matches, 100);
        let total_needed = std::cmp::min(matches_count + args.offset, 100);

        // Fetch just the match IDs from API (fast - 1 request)
        let api_match_ids = client.get_match_ids(&account.puuid, total_needed)?;

        // Record API request
        rate_limiter.record_request();
        rate_limiter.save().ok();

        if api_match_ids.is_empty() {
            return Err(AppError::NoRankedGames);
        }

        // Compare with cache
        let cached_ids: std::collections::HashSet<_> = match_cache
            .as_ref()
            .unwrap()
            .matches
            .iter()
            .map(|m| m.id.clone())
            .collect();

        let new_ids: Vec<String> = api_match_ids
            .iter()
            .filter(|id| !cached_ids.contains(*id))
            .cloned()
            .collect();

        if new_ids.is_empty() {
            // Cache is up-to-date, use it directly
            display_success("⚡ Cache is up-to-date (no new matches)");
            match_cache
                .as_ref()
                .unwrap()
                .matches
                .iter()
                .map(|m| m.id.clone())
                .collect::<Vec<_>>()
        } else {
            // Found new matches - fetch only the new ones
            display_success(&format!("✨ Found {} new matches, fetching details...", new_ids.len()));

            // Merge: new IDs + cached IDs
            let mut merged = new_ids.clone();
            merged.extend(
                match_cache
                    .as_ref()
                    .unwrap()
                    .matches
                    .iter()
                    .map(|m| m.id.clone())
            );
            merged
        }
    } else if args.refresh {
        // Force refresh from API
        display_info("Step 4: Refreshing data from Riot API (--refresh)...");
        let matches_count = std::cmp::min(args.matches, 100);
        let total_needed = std::cmp::min(matches_count + args.offset, 100);

        let ids = client.get_match_ids(&account.puuid, total_needed)?;

        // Record API request
        rate_limiter.record_request();
        rate_limiter.save().ok();

        if ids.is_empty() {
            return Err(AppError::NoRankedGames);
        }

        ids
    } else {
        // No cache, fetch from API
        display_info("Step 4: Fetching match IDs from Riot API...");
        let matches_count = std::cmp::min(args.matches, 100);
        let total_needed = std::cmp::min(matches_count + args.offset, 100);

        let ids = client.get_match_ids(&account.puuid, total_needed)?;

        // Record API request
        rate_limiter.record_request();
        rate_limiter.save().ok();

        if ids.is_empty() {
            return Err(AppError::NoRankedGames);
        }

        ids
    };

    // Extract the slice we want to analyze
    let match_ids: Vec<String> = all_match_ids
        .iter()
        .skip(args.offset)
        .take(std::cmp::min(args.matches, 100))
        .cloned()
        .collect();

    if match_ids.is_empty() {
        return Err(AppError::NoRankedGames);
    }

    display_success(&format!("Found {} matches to analyze", match_ids.len()));

    // Step 5: Fetch match details with progress bar
    let pb = ProgressBar::new(match_ids.len() as u64);
    pb.set_message("Fetching match details");
    let mut tracker = ChampionStatsTracker::new();
    let mut match_history = Vec::new();

    for (idx, match_id) in match_ids.iter().enumerate() {
        let match_data = client.get_match(match_id)?;

        // Record API request for match details
        rate_limiter.record_request();
        rate_limiter.save().ok();

        pb.inc(1);

        // Find our player in the match
        let our_player = match_data
            .info
            .participants
            .iter()
            .find(|p| p.puuid == account.puuid);

        let our_team_id = our_player.map(|p| p.team_id).unwrap_or(100);
        let won = our_player.map(|p| p.win).unwrap_or(false);
        let player_champion = our_player
            .map(|p| p.champion_name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        // Collect enemy champions and track allies
        let mut enemy_champions = Vec::new();
        let recency_weight = 1.0 - (idx as f64 / match_ids.len() as f64);

        // Track enemy champions and allies
        for participant in &match_data.info.participants {
            if participant.team_id != our_team_id {
                // Enemy champion
                enemy_champions.push(participant.champion_name.clone());
                tracker.add_champion_encounter(
                    participant.champion_name.clone(),
                    won,
                    recency_weight,
                );
            } else if participant.puuid != account.puuid {
                // Ally champion (same team but not us)
                tracker.add_ally_encounter(
                    participant.champion_name.clone(),
                    won,
                    recency_weight,
                );
            }
        }

        match_history.push(MatchResult {
            match_number: idx + 1,
            player_champion,
            won,
            enemy_champions,
        });
    }

    pb.finish_with_message("✓ Match data fetched");

    // Update cache with new matches
    if match_cache.is_none() {
        match_cache = Some(cache::MatchCache::new(&player_key, &config.region));
    }

    if let Some(ref mut cache_mut) = match_cache {
        cache_mut.region = region.clone();
        let cached_matches: Vec<cache::CachedMatch> = match_history
            .iter()
            .map(|m| cache::CachedMatch {
                id: match_ids[m.match_number - 1].clone(),
                champion: m.player_champion.clone(),
                won: m.won,
                enemies: m.enemy_champions.clone(),
                timestamp: chrono::Utc::now(),
            })
            .collect();

        cache_mut.add_matches(cached_matches);
        let _ = cache_mut.save(); // Save to disk silently
    }

    // Step 6: Generate recommendations (use actual analyzed matches, not total)
    let stats = tracker.get_stats();
    let total_games_analyzed = match_ids.len();
    let recommendations =
        BanRecommender::get_recommendations(stats, total_games_analyzed, args.top_n);

    // Display results
    let history_data: Vec<_> = match_history
        .iter()
        .map(|m| {
            (
                m.match_number,
                m.player_champion.clone(),
                m.won,
                m.enemy_champions.clone(),
            )
        })
        .collect();

    display_match_history(history_data);
    display_ban_recommendations(recommendations, &summoner.name);

    // Analyze and display ally performance
    let ally_stats = tracker.get_ally_stats();
    let ally_analysis = BanRecommender::analyze_allies(ally_stats, 1); // Show allies with 1+ games
    display_ally_analysis(ally_analysis);

    // Display API usage stats
    rate_limiter.display_status();

    Ok(())
}
