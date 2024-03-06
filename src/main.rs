use dotenv::dotenv;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth};
use std::{thread, time::Duration};
use terminal_spotify::get_env;

struct CurrentlyPlaying {
    name: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client_id = get_env("RSPOTIFY_CLIENT_ID");
    let client_secret = get_env("RSPOTIFY_CLIENT_SECRET");

    // user authentication
    // returns a variable which all functions used to get data can be run on
    let spotify = authorize_user(&client_id, &client_secret).await;

    let currently_playing = spotify.current_user_playing_item().await.unwrap();
    println!("{:?}", currently_playing);

    loop {
        println!("bruh bruh");
        thread::sleep(Duration::new(2, 0))
    }
}

async fn authorize_user(client_id: &str, client_secret: &str) -> AuthCodeSpotify {
    let creds = Credentials::new(client_id, client_secret);
    // localhost:8888 is currently just an express server with one endpoint, callback
    let oauth = OAuth {
        redirect_uri: "http://localhost:8888/callback".to_string(),
        scopes: scopes!(
            "user-read-playback-state",
            "user-read-currently-playing",
            "user-modify-playback-state"
        ),
        ..Default::default()
    };

    let spotify = AuthCodeSpotify::new(creds, oauth);

    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url).await.unwrap();

    // no need to parse response code, just return the AuthCodeSpotify
    return spotify;
}
