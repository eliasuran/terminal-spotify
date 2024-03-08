use dotenv::dotenv;
use rspotify::{
    model::CurrentlyPlayingContext, prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth,
};
use terminal_spotify::{get_env, user_input};

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
async fn get_available_devices(spotify: &AuthCodeSpotify) -> Vec<(Option<String>, bool)> {
    let devices = spotify.device().await.unwrap();
    devices
        .iter()
        .map(|device| (device.id.clone(), device.is_active))
        .collect()
}

// activates a device based on id
// async fn activate_device(spotify: &AuthCodeSpotify, id: &str) {}

async fn get_currently_playing(
    spotify: &AuthCodeSpotify,
) -> Result<CurrentlyPlayingContext, Box<dyn std::error::Error>> {
    //fetch
    let currently_playing = spotify.current_user_playing_item().await?.unwrap();

    if !currently_playing.is_playing {
        println!("Not listening to anything");
    }

    Ok(currently_playing)
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

    let mut active_device = String::from("");

    for (device_id, is_active) in devices {
        println!(
            "Device ID: {}, Active: {}",
            device_id.as_deref().unwrap_or(""),
            is_active
        );
        if is_active && device_id != Some(String::from("")) {
            active_device = device_id.as_deref().unwrap_or("").to_string()
        }
    }

    println!("The active device's id: {}", active_device);

    loop {
        let user_input = user_input();

        match user_input.as_str() {
            "play" => println!("boiii"),
            _ => println!("nothing written"),
        }

        if user_input.trim() == "exit" {
            break;
        }
    }

    println!("Exiting..");

    Ok(())
}
