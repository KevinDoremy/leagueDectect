// API endpoint definitions and URL builders
// Endpoints are handled directly in client.rs for this implementation

#[allow(dead_code)]
pub const ACCOUNT_ENDPOINT: &str = "https://americas.api.riotgames.com/riot/account/v1/accounts/by-riot-id";
#[allow(dead_code)]
pub const SUMMONER_ENDPOINT: &str = "https://{region}.api.riotgames.com/lol/summoner/v4/summoners/by-puuid";
#[allow(dead_code)]
pub const LEAGUE_ENDPOINT: &str = "https://{region}.api.riotgames.com/lol/league/v4/entries/by-summoner";
#[allow(dead_code)]
pub const MATCH_IDS_ENDPOINT: &str = "https://{region}.api.riotgames.com/lol/match/v5/matches/by-puuid";
#[allow(dead_code)]
pub const MATCH_ENDPOINT: &str = "https://{region}.api.riotgames.com/lol/match/v5/matches";
#[allow(dead_code)]
pub const DATA_DRAGON_ENDPOINT: &str = "https://ddragon.leagueoflegends.com/cdn/{version}/data/en_US/champion.json";
