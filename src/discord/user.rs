use log::error;
use serde::{Deserialize, Serialize};
use reqwest::{Client};
use crate::CONFIG;

#[derive(Deserialize, Serialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub global_name: String,
    pub avatar: String,
    pub email: Option<String>
}


pub async fn get_user(access_token: String) -> Result<DiscordUser, Box<dyn std::error::Error>> {
    let client = Client::new();

    let response = client.get(CONFIG.discord.discord_api_url.to_owned() + "/users/@me")
        .bearer_auth(access_token)
        .send()
        .await?;



    if response.status().is_success() {
        let result: Result<DiscordUser, reqwest::Error> = response.json().await;
        let user: DiscordUser = match result {
            Ok(user) => user,
            Err(err) => {
                error!("Error decoding JSON: {:?}", err);
                return Err(err.into())
            }
        };

        Ok(user)
    } else {

        let response_text = response.text().await?;

        error!("Error: {}", response_text);

        Err(response_text.into())
    }
}