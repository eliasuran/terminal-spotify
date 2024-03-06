use std::env;

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
// struct TrackObject {
//  id: String,
//name: String,
//artists: Box<[String]>,
//}

#[derive(Debug)]
pub struct CurrentlyPlaying {
    pub is_playing: bool,
    //r#type: String,
    //pub name: String,
    pub progress_ms: Option<chrono::TimeDelta>,
    //item: TrackObject,
}
