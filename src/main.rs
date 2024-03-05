use dotenv::dotenv;

use terminal_spotify::env_check;

fn main() {
    dotenv().ok();

    let client_id = env_check("CLIENT_ID");
    let client_secret = env_check("CLIENT_SECRET");

    println!(
        "CLIENT_ID: {:?}\nCLIENT_SECRET: {:?}",
        client_id, client_secret
    )
}
