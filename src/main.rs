mod config;
mod discord;

// My Modules
use crate::discord::oauth::exchange_code;
use config::{load_config, Data};

//Actix
use actix_web::{web::{Redirect, Query, scope}, cookie::{Key}, App, get, HttpResponse, HttpServer, Responder, Error, middleware::Logger};
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};

use serde::{Deserialize};
use env_logger::Env;
use once_cell::sync::Lazy;
use log::{info, error};
use reqwest::StatusCode;
use crate::discord::user::{DiscordUser, get_user};

static CONFIG: Lazy<Data> = Lazy::new(|| {
    info!("Loading configuration...");
    match load_config("conf.toml") {
        Ok(config) => config,
        Err(err) => {
            error!("Failed to load config: {}", err);
            panic!("Failed to load config");
        }
    }
});

#[derive(Deserialize)]
struct CallbackQuery {
    code: String
}

#[derive(Deserialize, Debug)]
struct CookieData {
    access_token: String,
    token_type: String,
    expires_in: i64,
    refresh_token: String,
    scope: String
}


#[get("/")]
async fn index(session: Session) -> Result<HttpResponse, Error> {
    if let Some(user) = session.get::<CookieData>("access_token")? {
        let user: DiscordUser = get_user(user.access_token).await.unwrap();
        Ok(HttpResponse::Ok().body(format!("Hello {}", user.username)))
    } else {
        Ok(HttpResponse::Ok().body("Hello, please <a href=\"/auth/login\">Login</a>"))
    }
}

#[get("/login")]
async fn login() -> impl Responder {
    Redirect::to(&CONFIG.discord.discord_auth_url).temporary()
}

#[get("/callback")]
async fn callback(query_params: Query<CallbackQuery>, session: Session) -> impl Responder {
    let code: String = String::from(&query_params.code);
    let token = exchange_code(code).await.unwrap();

    session.insert("access_token", &token).expect("Failed to insert access_token into session data");
    session.insert("expiration_time", &token.expires_in).expect("Failed to insert expiration_time into session data");
    Redirect::to("/").using_status_code(StatusCode::MOVED_PERMANENTLY)
}

async fn init_server(config: &'static Data) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .cookie_name(String::from("user"))
                    .build()
            )
            .wrap(Logger::default())
            .app_data(config)
            .service(index)
            .service(scope("/auth")
                .service(login)
                .service(callback)
            )
    })
        .bind(("localhost", 8080))?
        .run()
        .await
}


#[actix_web::main]
async fn main() {

    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_module_path(false)
        .init();

    info!("Starting the application...");

    if let Err(err) = init_server(&CONFIG).await {
        error!("Failed to start the server: {}", err);
    }
}