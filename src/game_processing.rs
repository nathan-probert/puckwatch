use serde::{Deserialize, Deserializer};
use serde_json::Value;
use reqwest;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct GameInfo {
    pub state: String,
    pub home_team_abbrev: String,
    pub home_team_score: u8,
    pub away_team_abbrev: String,
    pub away_team_score: u8,
}

impl<'de> Deserialize<'de> for GameInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Value::deserialize(deserializer)?;

        let state = v["gameState"]
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("missing game state"))?
            .to_string();

        let home_team_abbrev = v["homeTeam"]["abbrev"]
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("missing home abbreviation"))?
            .to_string();

        let home_team_score = v["homeTeam"]["score"]
            .as_u64()
            .ok_or_else(|| serde::de::Error::custom("missing home score"))? as u8;

        let away_team_abbrev = v["awayTeam"]["abbrev"]
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("missing away abbreviation"))?
            .to_string();

        let away_team_score = v["awayTeam"]["score"]
            .as_u64()
            .ok_or_else(|| serde::de::Error::custom("missing away score"))? as u8;

        Ok(GameInfo {
            state,
            home_team_abbrev,
            home_team_score,
            away_team_abbrev,
            away_team_score,
        })
    }
}

fn fetch_raw_games_data(url: &str) -> Result<serde_json::Value, Box<dyn Error>> {
    Ok(reqwest::blocking::get(url)?.json::<serde_json::Value>()?)
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

    games_list
        .iter()
        .map(|game_val| serde_json::from_value(game_val.clone()).map_err(|e| e.into()))
        .collect()
}


pub fn fetch_games_info(url: &str) -> Result<Vec<GameInfo>, Box<dyn Error>> {
    let raw_data = fetch_raw_games_data(url)?;
    parse_games_from_data(raw_data)
}


pub fn filter_ongoing_games(games: &[GameInfo]) -> Vec<GameInfo> {
    games
        .iter()
        .filter(|game| game.state != "OFF")
        .cloned()
        .collect()
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
