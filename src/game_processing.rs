use serde::Deserialize;
use serde_json::Value;
use reqwest;
use std::error::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct GameInfo {
    pub home_team_abbrev: String,
    pub home_team_score: u8,
    pub away_team_abbrev: String,
    pub away_team_score: u8,
}

fn parse_games_from_data(data: serde_json::Value) -> Result<Vec<GameInfo>, Box<dyn Error>> {
    let focused_date_str = data.get("focusedDate")
        .and_then(Value::as_str)
        .ok_or("Response missing 'focusedDate' field or it's not a string")?;

    let games_by_date_array = data.get("gamesByDate")
        .and_then(Value::as_array)
        .ok_or("Response missing 'gamesByDate' field or it's not an array")?;

    let games_list = games_by_date_array
        .iter()
        .find(|entry| entry.get("date").and_then(Value::as_str) == Some(focused_date_str))
        .and_then(|entry| entry.get("games"))
        .and_then(Value::as_array)
        .ok_or_else(|| format!("No 'games' array found for date: {}", focused_date_str))?;

    let mut games = Vec::new();
    for game_val in games_list {
        if let Some(state) = game_val.get("gameState").and_then(Value::as_str) {
            if state == "OFF" || state == "FUT" {
                continue; // skip games that are not in progress
            }
        }
        let home_team_abbrev = game_val["homeTeam"]["abbrev"].as_str().unwrap_or("").to_string();
        let home_team_score = game_val["homeTeam"]["score"].as_u64().unwrap_or(0) as u8;
        let away_team_abbrev = game_val["awayTeam"]["abbrev"].as_str().unwrap_or("").to_string();
        let away_team_score = game_val["awayTeam"]["score"].as_u64().unwrap_or(0) as u8;

        games.push(GameInfo {
            home_team_abbrev,
            home_team_score,
            away_team_abbrev,
            away_team_score,
        });
    }
    Ok(games)
}

pub fn fetch_games_info(url: &str) -> Result<Vec<GameInfo>, Box<dyn Error>> {
    let raw_data = reqwest::blocking::get(url)?.json::<serde_json::Value>()?;
    parse_games_from_data(raw_data)
}


pub fn filter_favourite_teams(games: &[GameInfo], favourite_teams: &[&str]) -> Vec<GameInfo> {
    games
        .iter()
        .filter(|game| {
            favourite_teams.contains(&game.home_team_abbrev.as_str())
                || favourite_teams.contains(&game.away_team_abbrev.as_str())
        })
        .cloned()
        .collect()
}


pub fn print_game_scores(games: &[GameInfo]) {
    if games.is_empty() {
        return;
    }
    for game in games {
        println!(
            "{}: {} - {}: {}",
            game.home_team_abbrev, game.home_team_score, game.away_team_abbrev, game.away_team_score
        );
    }
}
