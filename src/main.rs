mod config;
mod discord;

// My Modules
use crate::discord::oauth::exchange_code;
use config::{load_config, Data};

//Actix
use actix_web::{web::{Redirect, Query, scope}, App, get, HttpResponse, HttpServer, Responder, Error};
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;

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


#[get("/")]
async fn index(session: Session) -> Result<HttpResponse, Error> {
    if let Some(access_token) = session.get::<String>("access_token")? {
        let user: DiscordUser = get_user(access_token).await.unwrap();
        Ok(HttpResponse::Ok().body("Hello, ".to_owned() + &user.username))
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
    info!("Here is the callback code from your auth request: {}", code);

    let token = exchange_code(code).await.unwrap();

    session.insert("access_token", &token).expect("Failed to insert access_token into session data");

    info!("Here is the auth token from your auth request: {}", token.access_token);
    Redirect::to("/").using_status_code(StatusCode::MOVED_PERMANENTLY)
}

async fn init_server(config: &'static Data) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .build()
            )
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

    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format_module_path(false)
        .write_style(env_logger::WriteStyle::Always)
        .init();

    info!("Starting the application...");


    // Lazy::force(&CONFIG);

    if let Err(err) = init_server(&CONFIG).await {
        error!("Failed to start the server: {}", err);
    }
}