use std::env;

use chrono::{DateTime, Duration, Utc};
use rspotify::model::PlayableItem;

pub fn get_env(variable: &str) -> String {
    let env_var = match env::var(variable) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("{} environment variable was not found", variable);
            return String::from("");
        }
    };
    return env_var.to_string();
}

///// STRUCTS /////
//struct Track {}

//struct Episode {}

//enum PlayableItems {
//Track(Track),
//Episode(Episode),
//}

#[derive(Debug)]
pub struct CurrentlyPlayingData {
    pub item: Option<PlayableItem>,
    pub progress_ms: Option<Duration>,
    pub is_playing: bool,
    pub timestamp: DateTime<Utc>,
}
