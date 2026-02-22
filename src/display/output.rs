use crate::analysis::recommender::{BanRecommendation, AllyAnalysis};
use colored::*;
use tabled::{settings::Style, Table, Tabled};

#[derive(Tabled)]
struct MatchRow {
    #[tabled(rename = "#")]
    number: String,
    champion: String,
    result: String,
    enemies: String,
}

#[derive(Tabled)]
struct BanRow {
    rank: String,
    champion: String,
    frequency: String,
    win_rate: String,
    score: String,
}

#[derive(Tabled)]
struct AllyRow {
    rank: String,
    champion: String,
    games: String,
    win_rate: String,
}

pub fn display_ban_recommendations(
    recommendations: Vec<BanRecommendation>,
    player_name: &str,
) {
    println!(
        "\n{}",
        format!("🎮 Ban Recommendations for {} ", player_name)
            .bold()
            .cyan()
    );
    println!("{}\n", "=".repeat(60).cyan());

    if recommendations.is_empty() {
        println!(
            "{}",
            "No ban recommendations available (not enough data)".yellow()
        );
        return;
    }

    let mut rows = vec![];
    for (idx, rec) in recommendations.iter().enumerate() {
        let rank = format!("#{}", idx + 1);
        let champion = rec.champion_name.clone();
        let frequency = format!("{:.1}%", rec.frequency);
        let win_rate = format!("{:.1}%", rec.win_rate * 100.0);
        let score = format!("{:.2}", rec.score);

        rows.push(BanRow {
            rank,
            champion,
            frequency,
            win_rate,
            score,
        });
    }

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", table);

    println!("\n{}", "Interpretation".bold().yellow());
    println!(
        "{}",
        "• Frequency: How often this champion appeared in your last 20 games"
    );
    println!("• Win Rate: Your win rate when facing this champion");
    println!("• Score: Combined metric (higher = more dangerous to your rank)\n");

    // Detailed reasoning for top 1 ban
    if let Some(top_ban) = recommendations.first() {
        println!("{}", "Top Priority Ban".bold().red());
        println!(
            "  {} faced {}/20 games ({:.1}%) with {:.1}% win rate",
            top_ban.champion_name, top_ban.times_faced, top_ban.frequency, top_ban.win_rate * 100.0
        );
        if top_ban.win_rate < 0.33 {
            println!(
                "  {} High threat - very low win rate",
                "⚠️".red()
            );
        } else if top_ban.frequency > 25.0 {
            println!(
                "  {} Very common in your elo",
                "🔥".red()
            );
        }
    }

    println!();
}

pub fn display_error(error: &str) {
    eprintln!("{} {}", "❌ Error:".red().bold(), error);
}

pub fn display_info(message: &str) {
    println!("{} {}", "ℹ️".cyan(), message);
}

pub fn display_success(message: &str) {
    println!("{} {}", "✓".green(), message);
}

pub fn display_match_history(matches: Vec<(usize, String, bool, Vec<String>)>) {
    let total_matches = matches.len();
    let wins = matches.iter().filter(|(_, _, won, _)| *won).count();
    let losses = total_matches - wins;
    let win_rate = (wins as f64 / total_matches as f64) * 100.0;

    println!("\n{}", format!("📊 MATCH HISTORY (Last {} Games)", total_matches).bold().cyan());
    println!("{}\n", "=".repeat(80).cyan());
    println!("{} {} W / {} L ({:.1}% WR)\n",
        "📈 Overall:".bold(),
        wins.to_string().green(),
        losses.to_string().red(),
        win_rate);

    let mut rows = vec![];
    for (number, champion, won, enemies) in matches {
        let result = if won {
            "WIN".green().to_string()
        } else {
            "LOSS".red().to_string()
        };

        let enemies_str = enemies.join(", ");

        rows.push(MatchRow {
            number: format!("{}", number),
            champion,
            result,
            enemies: enemies_str,
        });
    }

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}\n", table);
}

pub fn display_ally_analysis(allies: Vec<AllyAnalysis>) {
    if allies.is_empty() {
        return;
    }

    println!("\n{}", "👥 ALLY PERFORMANCE ANALYSIS".bold().cyan());
    println!("{}\n", "=".repeat(60).cyan());

    let mut rows = vec![];
    for (idx, ally) in allies.iter().enumerate() {
        let rank = format!("#{}", idx + 1);
        let champion = ally.champion_name.clone();
        let games = format!("{}", ally.times_played_together);
        let win_rate = format!("{:.1}%", ally.win_rate * 100.0);

        rows.push(AllyRow {
            rank,
            champion,
            games,
            win_rate,
        });
    }

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", table);

    println!("\n{}", "Analysis".bold().yellow());
    println!("• Lists allies with lowest win rate when playing together");
    println!("• Consider playing differently with these champions or adjusting team composition\n");

    if let Some(worst_ally) = allies.first() {
        println!("{}", "Worst Ally Match".bold().red());
        println!(
            "  {} with {:.1}% win rate ({}/{} games)",
            worst_ally.champion_name, worst_ally.win_rate * 100.0, worst_ally.wins_together, worst_ally.times_played_together
        );
        if worst_ally.win_rate < 0.25 {
            println!(
                "  {} Very poor synergy - may need team adjustments",
                "⚠️".red()
            );
        } else if worst_ally.win_rate < 0.4 {
            println!(
                "  {} Below average synergy with this ally",
                "⚠️".yellow()
            );
        }
    }

    println!();
}
