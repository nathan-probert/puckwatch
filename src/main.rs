mod game_processing;
mod status_manager;

use game_processing::*;
use status_manager::*;
use std::error::Error;

const NHL_API_URL: &str = "https://api-web.nhle.com/v1/scoreboard/now";

fn main() -> Result<(), Box<dyn Error>> {
    // Define your favourite teams here
    let favourite_teams = ["TOR"];

    let mut previous_status =
        read_status_file().unwrap_or_else(|_| TrackerStatus::default_status());

    let current_timestamp = get_current_timestamp()?;
    let is_first_run_today = previous_status.last_run_timestamp == 0
        || (current_timestamp / (24 * 60 * 60))
            > (previous_status.last_run_timestamp / (24 * 60 * 60));

    if is_first_run_today {
        // For first run of the day, fetch all start times for favourite teams
        match fetch_all_games_details(NHL_API_URL) {
            Ok(all_games) => {
                let favourite_games =
                    filter_favourite_teams_details(&all_games, &favourite_teams);
                let favourite_games_cloned: Vec<FullGameDetails> = favourite_games.iter().map(|&game| game.clone()).collect();
                let future_start_timestamps = extract_future_game_start_timestamps(&favourite_games_cloned)?;
                previous_status.game_start_timestamps = future_start_timestamps;

                // Print the games with future start times once
                if !favourite_games.is_empty() {
                    for game in favourite_games {
                        println!(
                            "{} vs {} - {}",
                            game.home_team_abbrev, game.away_team_abbrev, game.readable_start_time
                        );
                    }
                }

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

    // If no relevant start times have passed and there are no games in the status, write the status file
    if !relevant_start_times_passed && !status.game_start_timestamps.is_empty() {
        write_status_file(&TrackerStatus::new(
            current_timestamp,
            status.current_status.clone(),
            status.game_start_timestamps.clone(),
        ))?;
        return Ok(());
    }

    // Handle live games
    match fetch_all_games_details(NHL_API_URL) {
        Ok(all_games_details) => {
            let favourite_games_details =
                filter_favourite_teams_details(&all_games_details, favourite_teams);

            print_game_scores_details(&favourite_games_details);

            // Determine if there's any active game
            let mut active_game_found = false;
            for game_detail in &favourite_games_details {
                if game_detail.game_state != "OFF" && game_detail.game_state != "FUT" {
                    active_game_found = true;
                    break; 
                }
            }

            // A timestamp is removed if all games associated with it are "OFF", or if no games are found for it.
            let mut new_updated_start_times = Vec::new();
            for &timestamp_in_status in &status.game_start_timestamps {
                let relevant_games_for_timestamp: Vec<&FullGameDetails> = favourite_games_details
                    .iter()
                    .filter(|&game_detail| {
                        match parse_start_time_to_timestamp(&game_detail.start_time_utc) {
                            Ok(game_ts) => game_ts == timestamp_in_status,
                            Err(_) => false,
                        }
                    })
                    .cloned() 
                    .collect();

                if relevant_games_for_timestamp.is_empty() {
                    // No games for this timestamp in current API results; assume they are over.
                } else {
                    // Games found for this timestamp. Check if all are "OFF".
                    let all_found_games_are_off = relevant_games_for_timestamp
                        .iter()
                        .all(|game| game.game_state == "OFF");

                    if !all_found_games_are_off {
                        new_updated_start_times.push(timestamp_in_status);
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
                new_updated_start_times, // Use the newly computed list
            ))?;
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(())
}
