use reqwest;
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;
use chrono::{DateTime, FixedOffset, Utc};

#[derive(Debug, Clone, Deserialize)]
pub struct FullGameDetails {
    pub home_team_abbrev: String,
    pub home_team_score: u8,
    pub away_team_abbrev: String,
    pub away_team_score: u8,
    pub game_state: String,
    pub start_time_utc: String,
    pub readable_start_time: String,
}

fn convert_to_eastern_time(start_time_utc: &str, est_offset: &str) -> String {
    // Parse the UTC datetime
    let utc_time = match DateTime::parse_from_rfc3339(start_time_utc) {
        Ok(dt) => dt.with_timezone(&Utc),
        Err(_) => return "Invalid UTC time".to_string(),
    };

    // Parse the EST offset (e.g., "-0500" or "-05:00")
    let offset_secs = {
        let cleaned = est_offset.replace(":", "");
        let sign = if cleaned.starts_with('-') { -1 } else { 1 };
        let hours: i32 = cleaned[1..3].parse().unwrap_or(0);
        let mins: i32 = cleaned[3..5].parse().unwrap_or(0);
        sign * (hours * 3600 + mins * 60)
    };
    let eastern_offset = match FixedOffset::east_opt(offset_secs) {
        Some(offset) => offset,
        None => return "Invalid offset".to_string(),
    };

    // Apply the offset and format time without leading zero
    let eastern_time = utc_time.with_timezone(&eastern_offset);
    format!(
        "{}:{} {}",
        eastern_time.format("%-I"), // %-I removes leading zero
        eastern_time.format("%M"),
        eastern_time.format("%p")
    )
}


fn parse_all_games_details_from_data(
    data: serde_json::Value,
) -> Result<Vec<FullGameDetails>, Box<dyn Error>> {
    let focused_date_str = data
        .get("focusedDate")
        .and_then(Value::as_str)
        .ok_or("Response missing 'focusedDate' field or it's not a string")?;

    let games_by_date_array = data
        .get("gamesByDate")
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
        let game_state = game_val["gameState"].as_str().unwrap_or("").to_string();
        let start_time_utc = game_val["startTimeUTC"].as_str().unwrap_or("").to_string();
        let est_offset = game_val["easternUTCOffset"].as_str().unwrap_or("-0500").to_string();
        let readable_start_time = convert_to_eastern_time(&start_time_utc, &est_offset);
        let home_team_abbrev = game_val["homeTeam"]["abbrev"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let home_team_score = game_val["homeTeam"]["score"].as_u64().unwrap_or(0) as u8;
        let away_team_abbrev = game_val["awayTeam"]["abbrev"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let away_team_score = game_val["awayTeam"]["score"].as_u64().unwrap_or(0) as u8;

        games.push(FullGameDetails {
            home_team_abbrev,
            home_team_score,
            away_team_abbrev,
            away_team_score,
            game_state,
            start_time_utc,
            readable_start_time,
        });
    }
    Ok(games)
}

pub fn fetch_all_games_details(url: &str) -> Result<Vec<FullGameDetails>, Box<dyn Error>> {
    let raw_data = reqwest::blocking::get(url)?.json::<serde_json::Value>()?;
    parse_all_games_details_from_data(raw_data)
}

pub fn extract_future_game_start_timestamps(
    all_games: &[FullGameDetails],
) -> Result<Vec<u64>, Box<dyn Error>> {
    let mut timestamps = Vec::new();
    for game in all_games {
        if game.game_state == "FUT" && !game.start_time_utc.is_empty() {
            match DateTime::parse_from_rfc3339(&game.start_time_utc) {
                Ok(dt) => timestamps.push(dt.timestamp() as u64),
                Err(e) => eprintln!(
                    "Warning: Could not parse timestamp '{}': {}",
                    game.start_time_utc, e
                ),
            }
        }
    }
    timestamps.sort_unstable();
    timestamps.dedup();
    Ok(timestamps)
}

pub fn filter_favourite_teams_details<'a>(
    games: &'a [FullGameDetails],
    favourite_teams: &[&str],
) -> Vec<&'a FullGameDetails> {
    games
        .iter()
        .filter(|game| {
            favourite_teams.contains(&game.home_team_abbrev.as_str())
                || favourite_teams.contains(&game.away_team_abbrev.as_str())
        })
        .collect()
}

pub fn print_game_scores_details(games: &[&FullGameDetails]) {
    if games.is_empty() {
        return;
    }
    for game in games {
        println!(
            "{}: {} - {}: {} (State: {})",
            game.home_team_abbrev,
            game.home_team_score,
            game.away_team_abbrev,
            game.away_team_score,
            game.game_state
        );
    }
}

pub fn parse_start_time_to_timestamp(start_time_utc_str: &str) -> Result<u64, Box<dyn Error>> {
    if start_time_utc_str.is_empty() {
        return Err("Empty start time string".into());
    }
    Ok(DateTime::parse_from_rfc3339(start_time_utc_str)?.timestamp() as u64)
}
