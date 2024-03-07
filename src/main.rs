use std::io;

use dotenv::dotenv;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth};
use terminal_spotify::{get_env, CurrentlyPlayingData};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let client_id = get_env("RSPOTIFY_CLIENT_ID");
    let client_secret = get_env("RSPOTIFY_CLIENT_SECRET");

    // user authentication
    let spotify = authorize_user(&client_id, &client_secret).await?;

    // get currenlty playing track
    let currently_playing = spotify.current_user_playing_item().await?.unwrap();

    if !currently_playing.is_playing {
        println!("Not listening to anything");
        return Ok(());
    }

    let currently_playing_data = CurrentlyPlayingData {
        is_playing: currently_playing.is_playing,
        item: currently_playing.item,
        progress_ms: currently_playing.progress,
        timestamp: currently_playing.timestamp,
    };

    println!("Currently playing: {:?}", currently_playing_data);

    loop {
        let user_input = user_input();

        println!("You wrote {}", user_input);

        if user_input.trim() == "exit" {
            println!("exiting");
            break;
        }
    }

    println!("Exiting..");

    Ok(())
}

fn user_input() -> String {
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
