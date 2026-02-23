use serde::Deserialize;

// Account V1 response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct AccountDto {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
}

// Summoner V4 response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct SummonerDto {
    #[serde(default)]
    pub id: String,
    pub puuid: String,
    #[serde(default)]
    pub name: String,
    pub summoner_level: i32,
    #[serde(default)]
    pub profile_icon_id: i32,
    #[serde(default)]
    pub revision_date: i64,
}

// League V4 response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct LeagueEntryDto {
    pub summoner_id: String,
    pub rank: String,
    pub tier: String,
    pub league_points: i32,
    pub wins: i32,
    pub losses: i32,
}

// Match V5 response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MatchDto {
    pub metadata: MatchMetadata,
    pub info: MatchInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct MatchMetadata {
    pub match_id: String,
    pub participants: Vec<String>,
    #[serde(default)]
    pub data_version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct MatchInfo {
    pub game_duration: i64,
    pub participants: Vec<ParticipantDto>,
    #[serde(default)]
    pub game_id: i64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ParticipantDto {
    pub puuid: String,
    pub champion_id: i32,
    pub champion_name: String,
    pub team_id: i32,
    pub win: bool,
    #[serde(default)]
    pub lane: String,  // TOP, JUNGLE, MIDDLE, BOTTOM, UTILITY
    #[serde(default)]
    pub role: String,  // TOP, JUNGLE, MID, ADC, SUPPORT
}

// Data Dragon Champion response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DataDragonChampions {
    pub data: std::collections::HashMap<String, ChampionInfo>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ChampionInfo {
    pub id: String,
    pub name: String,
    pub key: String,
}
