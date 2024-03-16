use chrono::{Duration, TimeDelta};
use colored::Colorize;
use dialoguer::Select;
use dotenv::dotenv;
use rspotify::{
    model::{PlayableItem, PlaylistId, SearchResult, SearchType, TrackId}, prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth
};
use std::io::Write;
use terminal_spotify::{get_env, print_err, printf_err, user_input, you_can_not_leave};

// has to be &str can't call String::from outside fn ?
const REDIRECT_URI: &str = "http://localhost:8888/callback";

////// AUTH /////
// TODO: IMPROVE AUTH (store refresh token in db or locally?)
async fn authorize_user(
    client_id: &str,
    client_secret: &str,
) -> Result<AuthCodeSpotify, Box<dyn std::error::Error>> {
    let creds = Credentials::new(client_id, client_secret);
    let oauth = OAuth {
        redirect_uri: REDIRECT_URI.to_string(),
        scopes: scopes!(
            "user-read-playback-state", // get status about player
            "user-read-currently-playing", // get status about player
            "user-modify-playback-state", // interact with player
            "playlist-read-private" // to get playlists
        ),
        ..Default::default()
    };

    let spotify = AuthCodeSpotify::new(creds, oauth);

    let url = spotify.get_authorize_url(false)?;
    spotify.prompt_for_token(&url).await?;

    Ok(spotify)
}

///// DEVICES /////
#[derive(Debug)]
struct Device {
    id: String,
    name: String,
    is_active: bool,
}
impl Default for Device {
    fn default() -> Self {
        Device {
            id: "".to_string(),
            name: "".to_string(),
            is_active: false,
        }
    }
}

async fn get_available_devices(spotify: &AuthCodeSpotify) -> Vec<(String, String, bool)> {
    let devices = spotify.device().await.unwrap();
    devices
        .iter()
        .map(|device| {
            (
                device.id.as_deref().unwrap_or("").to_string(),
                device.name.clone(),
                device.is_active,
            )
        })
        .collect()
}

fn print_devices(devices: &Vec<(String, String, bool)>, active_device: &mut Device) {
    if devices.len() == 0 {
        print_err("No devices available currently");
        return;
    }

    println!("Available devices:");

    for (id, name, is_active) in devices {
        println!("Device name: {}, Active: {}", name, is_active);
        if *is_active && id != "" {
            *active_device = Device {
                id: id.clone(),
                name: name.clone(),
                is_active: *is_active,
            }
        }
    }

    if active_device.id == "" {
        *active_device = Device {
            ..Default::default()
        }
    }
}

async fn activate_device(
    spotify: &AuthCodeSpotify,
    devices: &Vec<(String, String, bool)>,
    active_device: &mut Device,
) {
    let device_names: Vec<&str> = devices.iter().map(|device| (device.1.as_str())).collect();

    let selection = Select::new()
        .with_prompt("Choose the device you want to activate")
        .items(&device_names[..])
        .interact()
        .unwrap();

    for device in devices {
        if device_names[selection] == device.1 {
            *active_device = Device {
                id: device.0.clone(),
                name: device.1.clone(),
                is_active: true,
            }
        }
    }

    match spotify
        .transfer_playback(&active_device.id, Some(false))
        .await
    {
        Ok(_) => println!("{} was activated", device_names[selection]),
        Err(err) => printf_err("Could not activate the device", err),
    }
}

///// CURRENTLY PLAYING /////
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


///// SEARCHING / SELECTING /////
#[derive(Debug)]
struct SearchRes<'a> {
    id: TrackId<'a>, 
    song_name: String,
    artists: Vec<String>,
}

async fn search<'a>(spotify: &AuthCodeSpotify, query: &str) -> Vec<SearchRes<'a>> {
    let res = spotify
        .search(query, SearchType::Track, None, None, Some(5), None)
        .await;

    // flat_map cause have to do 2 iterations
    let search_data: Vec<SearchRes<'_>> = res
        .iter()
        .flat_map(|result: &_| {
            match result {
                SearchResult::Tracks(tracks) => tracks
                    .clone()
                    .items
                    .into_iter()
                    .map(|track| SearchRes {
                        id: track.id.unwrap().clone(),
                        song_name: track.name.clone(),
                        artists: track
                            .artists
                            .iter()
                            .map(|artist| artist.name.clone())
                            .collect(),
                    })
                    .collect::<Vec<SearchRes<'_>>>(),
                _ => vec![],
            }
        })
        .collect();

    search_data
}

#[derive(Debug)]
struct Playlist<'a> {
    id: PlaylistId<'a>, 
    name: String,
}

async fn select_playlist(spotify: &AuthCodeSpotify, active_device: &mut Device) {
    let playlists = spotify.current_user_playlists_manual(Some(10), None).await;

    let playlist_data: Vec<Playlist<'_>> = playlists.iter().flat_map(|res: &_| {
        res.clone().items.into_iter().map(|playlist| Playlist {
            id: playlist.id.clone(),
            name: playlist.name,
        }).collect::<Vec<Playlist<'_>>>()
    }).collect();

    let playlist_data_names: Vec<String> = playlist_data
        .iter()
        .map(|playlist| playlist.name.clone())
        .collect();

    let selection = Select::new()
        .with_prompt("Select song: ")
        .items(&playlist_data_names[..])
        .interact()
        .unwrap();

    match spotify.start_context_playback(PlayContextId::from(playlist_data[selection].id.clone()), Some(&active_device.id), None, None).await {
        Ok(_) => println!("Started playing playlist: {}", playlist_data[selection].name),
        Err(err) => printf_err("Could not start playing playlist", err)
    }
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

    let devices = get_available_devices(&spotify).await;

    let mut active_device = Device {
        ..Default::default()
    };

    // prints and sets active_device
    print_devices(&devices, &mut active_device);

    loop {
        // force print out the > to make it appear before user_input
        print!("{} ", "->".bold().bright_green());
        std::io::stdout().flush().unwrap();

        let input = user_input();

        if input.trim().is_empty() {
            continue;
        }

        let mut currently_playing = CurrentlyPlaying {
            is_playing: false,
            progress: Duration::new(0, 0),
            song_name: "".to_string(),
            artists: vec![],
        };

        if active_device.id != "" {
            currently_playing = get_currently_playing(&spotify).await?;
        }

        match input.trim() {
            "help" => println!(
                "Available commands:\n\n{}\nhelp -> get a list of available commands\nexit -> exit\nactivate -> select a device you want to activate\n\n{}\ns/search -> search for and play a song\np -> resumes or pauses track, depending on which one is possible\nplay -> resume playback\npause -> pause playback\nrestart -> restarts track\nnext/prev -> skips to next or previous track\nforward/back -> select amount of seconds to go back or forward\nstatus -> get status of currently selected song",
                "Always available".bold().yellow(),
                "If a device is active:".bold().green()
            ),
            "activate" => activate_device(&spotify, &devices, &mut active_device).await,
            "devices" => {
                let devices = get_available_devices(&spotify).await;
                print_devices(&devices, &mut active_device)
            }
            "search" | "s" => {
                print!("Search for a song: ");
                std::io::stdout().flush().unwrap();

                let q = user_input();
                let search_data = search(&spotify, q.trim()).await;

                let search_data_song_and_artists: Vec<String> = search_data
                    .iter()
                    .map(|track| format!("{} - {}", track.song_name.as_str(), track.artists.join(", ")))
                    .collect();

                let selection = Select::new()
                    .with_prompt("Select song: ")
                    .items(&search_data_song_and_artists[..])
                    .interact()
                    .unwrap();
                
                let selected_song = &search_data[selection];

                match spotify
                    .start_uris_playback(Some(PlayableId::from(selected_song.id.clone())), Some(&active_device.id), None, None)
                    .await
                {
                    Ok(_) => println!("Started playing: {}", selected_song.song_name),
                    Err(err) => printf_err("Could not resume playback", err),
                }
            }
            "playlist" | "playlists" => select_playlist(&spotify, &mut active_device).await,
            "p" => {
                if active_device.id == "" {
                    print_err("Can't resume/pause playback because there is no active device");
                    continue;
                }
                if currently_playing.is_playing {
                    match spotify.pause_playback(Some(&active_device.id)).await {
                        Ok(_) => println!("Paused playback"),
                        Err(err) => printf_err("Could not pause playback", err),
                    }
                    continue
                }
                match spotify
                    .resume_playback(Some(&active_device.id), Duration::zero().into())
                    .await
                {
                    Ok(_) => println!("Resumed playback"),
                    Err(err) => printf_err("Could not resume playback", err),
                }
            }
            "play" => {
                // TODO: find a way to better check for active_device_id where it is needed
                if active_device.id == "" {
                    print_err("Can't resume playback because there is no active device");
                    continue;
                }
                if currently_playing.is_playing {
                    println!("Already playing");
                    continue;
                }
                match spotify
                    .resume_playback(Some(&active_device.id), Duration::zero().into())
                    .await
                {
                    Ok(_) => println!("Resumed playback"),
                    Err(err) => printf_err("Could not resume playback", err),
                }
            }
            "pause" => {
                if active_device.id == "" {
                    print_err("Can't pause playback because there is no active device");
                    continue;
                }
                if !currently_playing.is_playing {
                    println!("Already paused");
                    continue;
                }
                match spotify.pause_playback(Some(&active_device.id)).await {
                    Ok(_) => println!("Paused playback"),
                    Err(err) => printf_err("Could not pause playback", err),
                }
            }
            "restart" | "r" => {
                // use seek to position to set position to 0 ms
                match spotify.seek_track(Duration::zero().into(), Some(&active_device.id)).await {
                    Ok(_) => println!("Restarted track"),
                    Err(err) => printf_err("Could not restart track", err),
                }
            }
            "next" => {
                match spotify.next_track(Some(&active_device.id)).await {
                    Ok(_) => println!("Skipped to next track"),
                    Err(err) => printf_err("Could not skip to next track", err),
                }
            }
            "prev" | "previous" => {
                match spotify.previous_track(Some(&active_device.id)).await {
                    Ok(_) => println!("Skipped to previous track"),
                    Err(err) => printf_err("Could not skip to previous track", err),
                }
            }
            "fwd" | "forward" => {
                let seconds = vec![5, 10, 15, 20, 30, 45, 60];
                let selection = Select::new()
                    .with_prompt("Choose how many seconds forward you want to go")
                    .items(&seconds)
                    .interact()
                    .unwrap();
                let seconds_forward = currently_playing.progress.unwrap_or(Duration::zero()).num_seconds() + seconds[selection];
                match spotify.seek_track(Duration::new(seconds_forward as i64, 0).unwrap(), Some(&active_device.id)).await {
                    Ok(_) => println!("Skipped forward {} seconds", seconds[selection]),
                    Err(err) => printf_err("Could not skip forward", err)
                }
            }
            "back" => {
                let seconds = vec![5, 10, 15, 20, 30, 45, 60];
                let selection = Select::new()
                    .with_prompt("Choose how many seconds back you want to go")
                    .items(&seconds)
                    .interact()
                    .unwrap();
                let mut seconds_back = currently_playing.progress.unwrap_or(Duration::zero()).num_seconds() - seconds[selection];
                if seconds_back < 0 {
                    seconds_back = 0
                }
                match spotify.seek_track(Duration::new(seconds_back, 0).unwrap(), Some(&active_device.id)).await {
                    Ok(_) => println!("Skipped back {} seconds", seconds[selection]),
                    Err(err) => printf_err("Could not skip back", err)
                }
            }
            "status" => {
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
            "exit" => {
                let exit_condition = you_can_not_leave();
                if exit_condition {
                    break
                }
            },
            _ => printf_err("Command not found", input),
        }
    }

    println!("Exiting..");

    Ok(())
}
