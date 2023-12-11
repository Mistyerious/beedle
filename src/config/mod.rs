use log::{info};
use std::{env, fs};
use serde::Deserialize;
use toml;


#[derive(Deserialize, Debug)]
pub struct Data {
    pub discord: DiscordData,
    pub actix: ActixData,
}

#[derive(Deserialize, Debug)]
pub struct DiscordData {
    pub client_secret: String,
    pub client_id: String,
    pub redirect_url: String,
    pub discord_api_url: String,
    pub discord_auth_url: String,
    pub discord_auth_url_email: String
}

#[derive(Deserialize, Debug)]
pub struct ActixData {
    pub secret: String
}

pub fn load_config(filename: &str) -> Result<Data, Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let file_path = current_dir.join(filename);

    if !file_path.exists() {
        return Err(format!("Config file does not exist at: {}", file_path.display()).into())
    }

    let contents = fs::read_to_string(&file_path)?;

    let data: Data = toml::from_str(&contents)?;

    info!("Config loaded successfully from `{}`", filename);
    Ok(data)
}