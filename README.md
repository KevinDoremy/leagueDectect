# League Detect - Ban Analyzer

A Rust CLI tool that analyzes your last 20 ranked League of Legends games and recommends which champions to ban.

## Features

- Fetches your last 20 ranked matches from Riot API
- Analyzes enemy champion frequencies and win rates
- Generates ban recommendations using a weighted scoring algorithm
- Colorized CLI output with detailed statistics
- Rate-limited API requests (respects Riot API limits)

## Installation

1. Clone the repository
2. Install Rust (if not already installed): https://rustup.rs/
3. Copy `.env.example` to `.env` and add your Riot API key:
   ```bash
   cp .env.example .env
   ```

## Setup

1. Get a Riot API key from https://developer.riotgames.com/
2. Add your API key to `.env`:
   ```
   RIOT_API_KEY=RGAPI-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
   RIOT_REGION=na1
   ```

3. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

```bash
cargo run -- "YourGameName" "TAG"
```

Or with a specific region:
```bash
cargo run -- "YourGameName" "TAG" --region euw1
```

### Examples

```bash
# NA region (default)
cargo run -- "PlayerName" "NA1"

# EUW region
cargo run -- "PlayerName" "EUW" --region euw1

# Get top 10 bans instead of default 5
cargo run -- "PlayerName" "NA1" --top-n 10
```

## Output

The tool displays:
- Your summoner level and rank
- Your last 20 ranked matches
- Top 5 ban recommendations with:
  - **Frequency**: How often the champion appeared
  - **Win Rate**: Your win rate against this champion
  - **Score**: Combined metric for ban priority

## Ban Scoring Algorithm

The scoring algorithm considers three factors:

```
score = (0.4 × frequency) + (0.5 × (1 - win_rate)) + (0.1 × recency)

where:
- frequency = (times faced / 20) × 100
- win_rate = wins against / times faced
- recency = weighted by match position (recent games weighted higher)
```

Higher score = higher priority to ban

## API Limits

The Riot API has rate limits:
- 20 requests/second (developer key)
- 100 requests/2 minutes (developer key)

This tool respects these limits and will wait if necessary.

## Error Handling

Common errors and solutions:

| Error | Solution |
|-------|----------|
| `Player not found` | Check spelling and tag format: `Name#TAG` |
| `No ranked games found` | Play ranked games first |
| `API key not found` | Add `RIOT_API_KEY` to `.env` |
| `Rate limit exceeded` | Wait a few seconds and try again |

## Project Structure

```
src/
├── main.rs              # CLI entry and orchestration
├── config.rs            # Configuration (API key, region)
├── error.rs             # Error types
├── api/
│   ├── client.rs        # HTTP client with rate limiting
│   ├── endpoints.rs     # API endpoint constants
│   └── models.rs        # API response structs
├── analysis/
│   ├── champion_stats.rs # Champion statistics tracking
│   └── recommender.rs    # Ban scoring algorithm
└── display/
    └── output.rs        # CLI formatting and output
```

## Dependencies

- **ureq**: Lightweight HTTP client
- **serde/serde_json**: JSON handling
- **clap**: CLI argument parsing
- **governor**: Rate limiting
- **colored**: Terminal colors
- **tabled**: Table formatting
- **indicatif**: Progress bars
- **anyhow/thiserror**: Error handling

## License

MIT
