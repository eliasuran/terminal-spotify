use chrono::{Duration, TimeDelta};
use dotenv::dotenv;
use rspotify::{model::PlayableItem, prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth};
use std::io::Write;
use terminal_spotify::{get_env, user_input};

// has to be &str can't call String::from outside fn ?
const REDIRECT_URI: &str = "http://localhost:8888/callback";

async fn authorize_user(
    client_id: &str,
    client_secret: &str,
) -> Result<AuthCodeSpotify, Box<dyn std::error::Error>> {
    let creds = Credentials::new(client_id, client_secret);
    let oauth = OAuth {
        redirect_uri: REDIRECT_URI.to_string(),
        scopes: scopes!(
            "user-read-playback-state",
            "user-read-currently-playing",
            "user-modify-playback-state"
        ),
        ..Default::default()
    };

    let spotify = AuthCodeSpotify::new(creds, oauth);

    let url = spotify.get_authorize_url(false)?;
    spotify.prompt_for_token(&url).await?;

    Ok(spotify)
}

// gets the state of the player (is a device active or not)
// TODO: get playback state to check currently running device

// get all available devices
// ex: get_currently_playing() is not allowed unless something is playing
async fn get_available_devices(spotify: &AuthCodeSpotify) -> Vec<(Option<String>, String, bool)> {
    let devices = spotify.device().await.unwrap();
    devices
        .iter()
        .map(|device| (device.id.clone(), device.name.clone(), device.is_active))
        .collect()
}

#[derive(Debug)]
struct CurrentlyPlaying {
    is_playing: bool,
    progress: Option<TimeDelta>,
    song_name: String,
    artists: Vec<String>,
}
async fn get_currently_playing(
    spotify: &AuthCodeSpotify,
) -> Result<CurrentlyPlaying, Box<dyn std::error::Error>> {
    //fetch
    let currently_playing = spotify.current_user_playing_item().await?.unwrap();

    let currently_playing_data = CurrentlyPlaying {
        is_playing: currently_playing.is_playing,
        progress: currently_playing.progress,
        song_name: match currently_playing.clone().item.unwrap() {
            PlayableItem::Track(track) => track.name,
            PlayableItem::Episode(episode) => episode.name,
        },
        artists: match currently_playing.item.unwrap() {
            PlayableItem::Track(track) => track
                .artists
                .iter()
                .map(|artist| (artist.name.clone()))
                .collect(),
            PlayableItem::Episode(..) => vec![String::from("")],
        },
    };

    Ok(currently_playing_data)
}

// TODO: add function to activate device?

// TODO: add function to start / resume playback

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let client_id = get_env("RSPOTIFY_CLIENT_ID");
    let client_secret = get_env("RSPOTIFY_CLIENT_SECRET");

    if client_id == "" || client_secret == "" {
        return Err("One or more of the necessary env variables were not found".into());
    }

    // user authentication
    let spotify = authorize_user(&client_id, &client_secret).await?;

    let devices = get_available_devices(&spotify).await;

    // TODO: make this store a vec of the active device, not just a string of the id
    let mut active_device_id = String::from("");

    if devices.len() == 0 {
        println!("No devices available currently")
    } else {
        println!("Available devices:");
        for (id, name, is_active) in devices {
            println!("Device name: {}, Active: {}", name, is_active);
            if is_active && id != Some("".to_string()) {
                active_device_id = id.as_deref().unwrap_or("").to_string();
            }
        }
    }

    loop {
        // force print out the > to make it appear before user_input
        print!("> ");
        std::io::stdout().flush().unwrap();

        let user_input = user_input();

        if user_input.trim().is_empty() {
            continue;
        }

        match user_input.trim() {
            "play" => {
                // TODO: find a way to better check for active_device_id where it is needed
                if active_device_id == "" {
                    println!("Can't resume playback because there is no active device");
                    continue;
                }
                match spotify
                    .resume_playback(Some(&active_device_id), Duration::zero().into())
                    .await
                {
                    Ok(_) => println!("Resumed playback"),
                    Err(err) => println!("Could not resume playback: {}", err),
                }
            }
            "pause" => {
                if active_device_id == "" {
                    println!("Can't pause playback because there is no active device");
                    continue;
                }
                match spotify.pause_playback(Some(&active_device_id)).await {
                    Ok(_) => println!("Paused playback"),
                    Err(err) => println!("Could not pause playback: {}", err),
                }
            }
            "status" => {
                if active_device_id == "" {
                    println!("Can't pause playback because there is no active device");
                    continue;
                }
                let currently_playing = get_currently_playing(&spotify).await?;

                if !currently_playing.is_playing {
                    println!("You are not listening to anything at the moment");
                    continue;
                }

                if currently_playing.is_playing {
                    println!(
                        "You are listening to {} by {}. Progress: {}",
                        currently_playing.song_name,
                        currently_playing.artists.join(", "),
                        // TODO: make this display with this format minutes:seconds, ex: 16:47
                        match currently_playing.progress {
                            Some(time) => time.num_seconds(),
                            None => 0,
                        }
                    );
                }
            }
            "exit" => break,
            _ => println!("Command not found: {}", user_input),
        }
    }

    println!("Exiting..");

    Ok(())
}
