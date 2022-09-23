use actix_files as fs;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key,
    get,
    web::{self, Data},
    App, HttpResponse, HttpServer, Result as ActixResult,
};
use anyhow::Result;
use log::info;
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    reqwest::async_http_client as http_client,
    url::Url,
    AuthenticationFlow, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, RedirectUrl, Scope,
};
use std::env;
use uuid::Uuid;

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();
    start_webserver().await?;
    Ok(())
}

pub struct AppState {
    pub dummy: String,
}

pub fn start_webserver() -> actix_web::dev::Server {
    let secret = Key::generate();

    info!("Starting webserver");

    let server = HttpServer::new(move || {
        let app_state = Data::new(AppState {
            dummy: "abcd".to_string(),
        });
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret.clone())
                    .cookie_secure(true)
                    .cookie_http_only(true)
                    .cookie_same_site(actix_web::cookie::SameSite::Strict)
                    .build(),
            )
            .app_data(app_state)
            .service(hello)
            .service(login)
            .service(fs::Files::new("/", "./dist").index_file("index.html"))
        // .default_service(web::get().to(index))
    });

    server.bind(("localhost", 8080)).unwrap().run()
}

#[get("/hello")]
async fn hello(_app_state: web::Data<AppState>) -> String {
    "hello there".to_string()
}

/// if anonymous user that is not logged in, we generate a session with key "anonuser" with a uuid to track him.
/// if he is already logged in?
#[get("/api/login")]
async fn login(_app_state: web::Data<AppState>, session: Session) -> ActixResult<HttpResponse> {
    let anonuser = "anonuser";
    if let Some(anonuserid) = session.get::<String>(anonuser)? {
        info!("Anonymous user already has a session_id: {}", anonuserid)
    } else {
        info!("Anonymous user does NOT have a session_id. Generating one for him");
        session.insert(anonuser, Uuid::new_v4().to_string())?
    }
    let (url, _, _) = get_oidc_login().await;
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header(("Location", url.to_string()))
        .body("Redirecting to login"))
}

// async fn index(req: HttpRequest) -> ActixResult<fs::NamedFile> {
//     Ok(fs::NamedFile::open("./dist/index.html")?)
// }

async fn get_oidc_login() -> (Url, CsrfToken, Nonce) {
    let google_client_id = ClientId::new(
        env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let google_client_secret = ClientSecret::new(
        env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );
    let issuer_url =
        IssuerUrl::new("https://accounts.google.com".to_string()).expect("Invalid issuer URL");
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, http_client)
        .await
        .unwrap();

    // Set up the config for the Google OAuth2 process.
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        google_client_id,
        Some(google_client_secret),
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8080/hello".to_string()).expect("Invalid redirect URL"),
    );

    let (authorize_url, csrf_state, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    (authorize_url, csrf_state, nonce)
}
