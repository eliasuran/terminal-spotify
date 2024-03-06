use dotenv::dotenv;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth};
use terminal_spotify::get_env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client_id = get_env("RSPOTIFY_CLIENT_ID");
    let client_secret = get_env("RSPOTIFY_CLIENT_SECRET");

    // user authentication
    let creds = Credentials::new(&client_id, &client_secret);
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

    let auth_code = spotify.parse_response_code(&url);

    if auth_code == None {
        println!("Could not get auth code");
        return;
    }

    let converted_auth_code = auth_code.as_deref().unwrap();

    let _access_token = spotify.request_token(converted_auth_code);
}
