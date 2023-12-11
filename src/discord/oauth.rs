use log::{error, info};
use reqwest::Client;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use crate::CONFIG;

#[derive(Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: String
}

pub async fn exchange_code(code: String) -> Result<Token, Box<dyn std::error::Error>> {

    info!("Redirect URL: {}", CONFIG.discord.redirect_url.to_owned());

    let client = Client::new();

    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/x-www-form-urlencoded").unwrap());

    let response = client.post(CONFIG.discord.discord_api_url.to_owned() + "/oauth2/token")
        .headers(headers)
        .basic_auth(CONFIG.discord.client_id.to_owned(), Some(CONFIG.discord.client_secret.to_owned()))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &CONFIG.discord.redirect_url.to_owned())
        ])
        .send()
        .await?;

    if response.status().is_success() {
        let result: Result<Token, reqwest::Error> = response.json().await;
        let token: Token = match result {
            Ok(token) => token,
            Err(err) => {
                error!("Error decoding JSON: {:?}", err);
                return Err(err.into())
            }
        };

        Ok(token)
    } else {

        let response_text = response.text().await?;

        error!("Error: {}", response_text);

        Err(response_text.into())
    }
}