mod status_manager;
mod game_processing;

use std::error::Error;
use status_manager::*;
use game_processing::*;

const NHL_API_URL: &str = "https://api-web.nhle.com/v1/scoreboard/now";

fn get_relevant_games(favourite_teams: &[&str]) -> Result<Vec<GameInfo>, Box<dyn Error>> {
    let all_games = fetch_games_info(NHL_API_URL)?;
    let favourite_ongoing_games = filter_favourite_teams(&all_games, favourite_teams);
    Ok(favourite_ongoing_games)
}

fn main() -> Result<(), Box<dyn Error>> {
    let favourite_teams = ["TOR", "DAL", "MTL"]; // Example favourite teams

    let previous_status = read_status_file().unwrap_or_else(|_| TrackerStatus::default_status());

    let current_timestamp = get_current_timestamp()?;
    let time_elapsed = current_timestamp.saturating_sub(previous_status.last_run_timestamp);

    let should_run = match previous_status.current_status.as_str() {
        _ if previous_status.last_run_timestamp == 0 => {
            true
        }
        STATUS_WATCHING_LIVE if time_elapsed >= TEN_SECONDS => true,
        STATUS_NO_GAMES_LIVE if time_elapsed >= TEN_MINUTES_IN_SECONDS => true,
        _ => false,
    };

    if should_run {
        match get_relevant_games(&favourite_teams) {
            Ok(games) => {
                print_game_scores(&games);
                let new_status_str = if games.is_empty() {
                    STATUS_NO_GAMES_LIVE.to_string()
                } else {
                    STATUS_WATCHING_LIVE.to_string()
                };
                write_status_file(&TrackerStatus::new(current_timestamp, new_status_str))?;
            }
            Err(e) => {
                write_status_file(&TrackerStatus::new(
                    current_timestamp,
                    previous_status.current_status,
                ))?;
                return Err(e);
            }
        }
    } else {
        if previous_status.current_status == STATUS_WATCHING_LIVE {
            TEN_SECONDS.saturating_sub(time_elapsed)
        } else {
            TEN_MINUTES_IN_SECONDS.saturating_sub(time_elapsed)
        };
    }

    Ok(())
}
