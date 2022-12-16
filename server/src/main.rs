use actix_files as fs;
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::{cookie::Key, web::Data, App, HttpServer};
use anyhow::Result;
use log::info;
use openidconnect::{CsrfToken, Nonce};
use std::collections::HashMap;
use tokio::sync::Mutex;
mod auth;
use std::env;
mod handlers;

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();
    start_webserver().await?;
    Ok(())
}

/// Store a mutex of hashmap to persist csrftoken and nonce
pub struct AppState {
    pub session_oidc_state: Mutex<HashMap<String, (CsrfToken, Nonce)>>,
    pub client_id: String,
    pub client_secret: String,
}

const GOOGLE_CLIENT_ID_KEY: &str = "GOOGLE_CLIENT_ID";
const GOOGLE_CLIENT_SECRET_KEY: &str = "GOOGLE_CLIENT_SECRET";
const SERVER_SECRET_KEY: &str = "SERVER_SECRET_KEY";

pub fn start_webserver() -> actix_web::dev::Server {
    let client_id =
        env::var(GOOGLE_CLIENT_ID_KEY).expect("Missing the GOOGLE_CLIENT_ID environment variable.");
    let client_secret = env::var(GOOGLE_CLIENT_SECRET_KEY)
        .expect("Missing the GOOGLE_CLIENT_SECRET environment variable.");
    let secret_key = env::var(SERVER_SECRET_KEY).expect("Expected SERVER_SECRET_KEY to be present");
    assert!(
        secret_key.len() > 64,
        "Expected {} secret key to have length > 64, it had length {}",
        SERVER_SECRET_KEY,
        secret_key.len()
    );
    let secret = Key::from(secret_key.as_bytes());

    info!("Starting webserver");

    let app_state = Data::new(AppState {
        session_oidc_state: Mutex::new(HashMap::<String, (CsrfToken, Nonce)>::new()),
        client_id,
        client_secret,
    });

    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret.clone())
                    .cookie_secure(true)
                    .cookie_http_only(true)
                    .cookie_same_site(actix_web::cookie::SameSite::Lax)
                    .cookie_content_security(actix_session::config::CookieContentSecurity::Private)
                    .build(),
            )
            .app_data(app_state.clone())
            .service(handlers::hello)
            .service(handlers::login)
            .service(handlers::token_exchange)
            .service(handlers::get_user_info)
            .service(handlers::logout)
            .service(fs::Files::new("/", "./dist").index_file("index.html"))
    });

    server.bind(("localhost", 8080)).unwrap().run()
}
