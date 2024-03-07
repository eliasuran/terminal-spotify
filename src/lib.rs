use chrono::{DateTime, Duration, Utc};
use rspotify::model::{ArtistId, PlayableItem};
use std::{env, io};

///// FUNCTIONS /////

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

pub fn user_input() -> String {
    fn read_input() -> Result<String, io::Error> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input)
    }

    let input = read_input();

    match input {
        Ok(input) => input,
        Err(_) => String::new(),
    }
}

///// STRUCTS /////
#[derive(Debug)]
pub struct Artists {
    pub id: Option<ArtistId<'static>>,
    pub name: String,
}

#[derive(Debug)]
pub struct Track {
    pub artists: Vec<Artists>,
}

#[derive(Debug)]
pub struct Episode {}

#[derive(Debug)]
pub enum PlayableItems {
    Track(Track),
    Episode(Episode),
}

#[derive(Debug)]
pub struct CurrentlyPlayingData {
    pub item: Option<PlayableItem>,
    pub progress_ms: Option<Duration>,
    pub is_playing: bool,
    pub timestamp: DateTime<Utc>,
}
