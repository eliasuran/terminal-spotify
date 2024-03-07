use dotenv::dotenv;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth};
use terminal_spotify::{get_env, user_input, CurrentlyPlayingData};

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

// used to check if certain functions are allowed
// ex: get_currently_playing() is not allowed unless something is playing
async fn get_player_status(spotify: &AuthCodeSpotify) -> bool {
    let currently_playing = spotify.current_user_playing_item().await;
    match currently_playing {
        Ok(_) => {
            if currently_playing.unwrap().unwrap().is_playing {
                return true;
            }
            return false;
        }
        Err(_) => return false,
    }
}

async fn get_currently_playing(
    spotify: AuthCodeSpotify,
) -> Result<CurrentlyPlayingData, Box<dyn std::error::Error>> {
    //fetch
    let currently_playing = spotify.current_user_playing_item().await?.unwrap();

    if !currently_playing.is_playing {
        println!("Not listening to anything");
    }

    let currently_playing_data = CurrentlyPlayingData {
        is_playing: currently_playing.is_playing,
        item: currently_playing.item,
        progress_ms: currently_playing.progress,
        timestamp: currently_playing.timestamp,
    };

    Ok(currently_playing_data)
}

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

    loop {
        let status = get_player_status(&spotify).await;

        println!("{:?}", status);

        let user_input = user_input();

        println!("You wrote {}", user_input);

        if user_input.trim() == "exit" {
            break;
        }
    }

    println!("Exiting..");

    Ok(())
}
