use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const STATUS_FILE_PATH: &str = "/tmp/nhl_game_tracker_status.json";
pub const STATUS_WATCHING_LIVE: &str = "WATCHING_LIVE";
pub const STATUS_NO_GAMES_LIVE: &str = "NO_GAMES_LIVE";

#[derive(Serialize, Deserialize, Debug)]
pub struct TrackerStatus {
    pub last_run_timestamp: u64,
    pub current_status: String,
    pub game_start_timestamps: Vec<u64>, // Renamed from game_start_times
}

impl TrackerStatus {
    pub fn new(last_run_timestamp: u64, current_status: String, game_start_timestamps: Vec<u64>) -> Self {
        TrackerStatus {
            last_run_timestamp,
            current_status,
            game_start_timestamps, // Updated field name
        }
    }

    pub fn default_status() -> Self {
        TrackerStatus {
            last_run_timestamp: 0,
            current_status: STATUS_NO_GAMES_LIVE.to_string(),
            game_start_timestamps: Vec::new(), // Updated field name
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
    // Ensure the directory exists, though /tmp/ should always exist
    if let Some(parent) = Path::new(STATUS_FILE_PATH).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json_string = serde_json::to_string_pretty(status)?;
    writeln!(file, "{}", json_string)?;
    Ok(())
}

pub fn get_current_timestamp() -> Result<u64, Box<dyn Error>> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
