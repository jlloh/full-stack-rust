use actix_files as fs;
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::{cookie::Key, web::Data, App, HttpServer};
use actix_web_lab::sse;
use anyhow::Result;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    SqliteConnection,
};
use futures::{executor::block_on, future::join_all};
use log::info;
use openidconnect::{CsrfToken, Nonce};
use std::{
    collections::HashMap,
    sync::Arc,
    thread,
    time::{Duration, SystemTime},
};
use tokio::sync::Mutex;
mod auth;
pub mod database;
pub mod schema;
use casbin::prelude::*;
use std::env;

use crate::database::establish_connection_pool;
mod handlers;

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();

    // sse sender in another tokio task
    let sender_list = Arc::new(Mutex::new(vec![]));
    let sender_list_copy = sender_list.clone();
    let _sender_task = thread::spawn(move || {
        info!("Starting server side event sender in background thread..");
        block_on(sse_sender(sender_list_copy));
    });

    // webserver
    start_webserver(sender_list).await?.await?;
    Ok(())
}

/// Store a mutex of hashmap to persist csrftoken and nonce
pub struct AppState {
    pub session_oidc_state: Mutex<HashMap<String, (CsrfToken, Nonce)>>,
    pub client_id: String,
    pub client_secret: String,
    pub sse_senders: Arc<Mutex<Vec<sse::Sender>>>,
    pub authz_enforcer: Enforcer,
    pub db_connection_pool: Pool<ConnectionManager<SqliteConnection>>, // pub db_connection: Arc<SqliteConnection>,
}

const GOOGLE_CLIENT_ID_KEY: &str = "GOOGLE_CLIENT_ID";
const GOOGLE_CLIENT_SECRET_KEY: &str = "GOOGLE_CLIENT_SECRET";
const SERVER_SECRET_KEY: &str = "SERVER_SECRET_KEY";
const DATABASE_URL_KEY: &str = "DATABASE_URL";
const ANONYMOUS: &str = "anonymous";

pub async fn start_webserver(
    sse_senders: Arc<Mutex<Vec<sse::Sender>>>,
) -> Result<actix_web::dev::Server> {
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

    // Casbin for authZ stuff. Create an enforcer
    let authz_enforcer = Enforcer::new("authz/abac_model.conf", "authz/abac_policy.csv").await?;

    info!("Establishing database connection");
    let db_connection_pool = establish_connection_pool();

    info!("Starting webserver in main thread");

    let app_state = Data::new(AppState {
        session_oidc_state: Mutex::new(HashMap::<String, (CsrfToken, Nonce)>::new()),
        client_id,
        client_secret,
        sse_senders,
        authz_enforcer,
        db_connection_pool,
    });

    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret.clone())
                    .cookie_secure(true)
                    .cookie_http_only(true)
                    .cookie_same_site(actix_web::cookie::SameSite::Lax)
                    .cookie_content_security(actix_session::config::CookieContentSecurity::Private)
                    // session lifecycle defaults to browser and 1 day
                    // .session_lifecycle()
                    .build(),
            )
            .app_data(app_state.clone())
            .service(handlers::hello)
            .service(handlers::login)
            .service(handlers::token_exchange)
            .service(handlers::get_user_info2)
            .service(handlers::logout)
            .service(handlers::subscribe)
            .service(handlers::admin_test)
            .service(handlers::get_new_number)
            .service(handlers::abandon_assigned_number)
            .service(fs::Files::new("/", "./dist").index_file("index.html"))
    });

    Ok(server.bind(("localhost", 8080)).unwrap().run())
}

async fn sse_sender(sse_senders: Arc<Mutex<Vec<sse::Sender>>>) {
    let now = SystemTime::now();
    loop {
        thread::sleep(Duration::from_secs(1));
        // use an Arc and Mutex. But does this mean I'm blocking new subscribers when I'm sending events
        let mut senders = sse_senders.lock().await;
        let elapsed = now.elapsed().expect("expected valid elapsed").as_secs();
        let futures = senders
            .clone()
            .into_iter()
            .map(|sender| send_message(elapsed.to_string(), sender));
        // channels that are able to have stuff sent to them are still alive
        // we overwrite the original list of senders with list of new senders
        let open_channels: Vec<sse::Sender> =
            join_all(futures).await.into_iter().flatten().collect();
        (*senders) = open_channels;
    }
}

async fn send_message(message: String, sender: sse::Sender) -> Option<sse::Sender> {
    let mut data = sse::Data::new(message);
    data.set_event("data");
    match sender.send(data).await {
        Ok(_) => Some(sender),
        Err(_) => None,
    }
}
