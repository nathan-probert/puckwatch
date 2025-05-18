use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const STATUS_FILE_PATH: &str = "/tmp/nhl_game_tracker_status.json";
pub const STATUS_WATCHING_LIVE: &str = "WATCHING_LIVE";
pub const STATUS_NO_GAMES_LIVE: &str = "NO_GAMES_LIVE";
pub const TEN_SECONDS: u64 = 10;
pub const TEN_MINUTES_IN_SECONDS: u64 = 10 * 60;

#[derive(Serialize, Deserialize, Debug)]
pub struct TrackerStatus {
    pub last_run_timestamp: u64,
    pub current_status: String,
}

impl TrackerStatus {
    pub fn new(last_run_timestamp: u64, current_status: String) -> Self {
        TrackerStatus {
            last_run_timestamp,
            current_status,
        }
    }

    pub fn default_status() -> Self {
        TrackerStatus {
            last_run_timestamp: 0,
            current_status: STATUS_NO_GAMES_LIVE.to_string(),
        }
    }
}

pub fn read_status_file() -> Result<TrackerStatus, Box<dyn Error>> {
    if !Path::new(STATUS_FILE_PATH).exists() {
        return Ok(TrackerStatus::default_status());
    }
    let mut file = File::open(STATUS_FILE_PATH)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    if contents.trim().is_empty() {
        return Ok(TrackerStatus::default_status());
    }
    let status: TrackerStatus = serde_json::from_str(&contents)?;
    Ok(status)
}

pub fn write_status_file(status: &TrackerStatus) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(STATUS_FILE_PATH)?;
    let json_string = serde_json::to_string_pretty(status)?;
    writeln!(file, "{}", json_string)?;
    Ok(())
}

pub fn get_current_timestamp() -> Result<u64, Box<dyn Error>> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
