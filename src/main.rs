mod game_processing;
mod status_manager;

use game_processing::*;
use status_manager::*;
use std::error::Error;

const NHL_API_URL: &str = "https://api-web.nhle.com/v1/scoreboard/now";

fn main() -> Result<(), Box<dyn Error>> {
    let favourite_teams = ["TOR", "DAL", "MTL"]; // Example favourite teams

    let mut previous_status =
        read_status_file().unwrap_or_else(|_| TrackerStatus::default_status());

    let current_timestamp = get_current_timestamp()?;

    let is_first_run_today = previous_status.last_run_timestamp == 0
        || (current_timestamp / (24 * 60 * 60))
            > (previous_status.last_run_timestamp / (24 * 60 * 60));

    if is_first_run_today {
        match fetch_all_games_details(NHL_API_URL) {
            Ok(all_games) => {
                let future_start_timestamps = extract_future_game_start_timestamps(&all_games)?;
                previous_status.game_start_timestamps = future_start_timestamps;

                process_games(&favourite_teams, &mut previous_status, current_timestamp)?;
            }
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        process_games(&favourite_teams, &mut previous_status, current_timestamp)?;
    }

    Ok(())
}

fn process_games(
    favourite_teams: &[&str],
    status: &mut TrackerStatus,
    current_timestamp: u64,
) -> Result<(), Box<dyn Error>> {
    let mut relevant_start_times_passed = false;
    status
        .game_start_timestamps
        .retain(|&timestamp| timestamp >= current_timestamp);

    for &start_time in &status.game_start_timestamps {
        if current_timestamp >= start_time {
            relevant_start_times_passed = true;
            break;
        }
    }

    if !relevant_start_times_passed && !status.game_start_timestamps.is_empty() {
        write_status_file(&TrackerStatus::new(
            current_timestamp,
            status.current_status.clone(), 
            status.game_start_timestamps.clone(),
        ))?;
        return Ok(());
    }

    match fetch_all_games_details(NHL_API_URL) {
        Ok(all_games_details) => {
            let favourite_games_details = filter_favourite_teams_details(&all_games_details, favourite_teams);

            print_game_scores_details(&favourite_games_details);

            let mut active_game_found = false;
            let mut updated_start_times = status.game_start_timestamps.clone();

            for game_detail in &favourite_games_details {
                if game_detail.game_state != "OFF" && game_detail.game_state != "FUT" {
                    active_game_found = true;
                }
                // Check if a game corresponding to a stored start time is now OFF
                if game_detail.game_state == "OFF" {
                    if let Ok(game_start_ts) = parse_start_time_to_timestamp(&game_detail.start_time_utc) {
                        updated_start_times.retain(|&ts| ts != game_start_ts);
                    }
                }
            }

            let new_status_str = if active_game_found {
                STATUS_WATCHING_LIVE.to_string()
            } else {
                STATUS_NO_GAMES_LIVE.to_string()
            };

            write_status_file(&TrackerStatus::new(
                current_timestamp,
                new_status_str,
                updated_start_times,
            ))?;
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(())
}
