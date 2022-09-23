use actix_files as fs;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::error::{ErrorForbidden, ErrorInternalServerError};
use actix_web::{
    cookie::Key,
    get,
    web::{self, Data},
    App, HttpResponse, HttpServer, Result as ActixResult,
};
use anyhow::{anyhow, Result as AnyhowResult};
use anyhow::{Context, Result};
use log::info;
use openidconnect::AuthorizationCode;
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    reqwest::async_http_client as http_client,
    url::Url,
    AuthenticationFlow, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, RedirectUrl, Scope,
};
use serde::Deserialize;
use std::sync::Mutex;
use std::{collections::HashMap, env};
use uuid::Uuid;

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();
    start_webserver().await?;
    Ok(())
}

/// Store a mutex of hashmap to persist csrftoken and nonce
pub struct AppState {
    pub session_oidc_state: Mutex<HashMap<String, (CsrfToken, Nonce)>>,
}

pub fn start_webserver() -> actix_web::dev::Server {
    // TODO: What to do if secret is regenerated and all cookies are invalid?
    // Dummy token. To be replaced in a real setting.
    let secret = Key::from(
        "mysupersecretrandomkeythatisverylonglonglongmysupersecretrandomkeythatisverylonglonglong"
            .as_bytes(),
    );

    info!("Starting webserver");

    let app_state = Data::new(AppState {
        session_oidc_state: Mutex::new(HashMap::<String, (CsrfToken, Nonce)>::new()),
    });

    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret.clone())
                    .cookie_secure(true)
                    .cookie_http_only(true)
                    .cookie_same_site(actix_web::cookie::SameSite::Lax)
                    .build(),
            )
            .app_data(app_state.clone())
            .service(hello)
            .service(login)
            .service(token_exchange)
            .service(get_user_info)
            .service(logout)
            .service(fs::Files::new("/", "./dist").index_file("index.html"))
    });

    server.bind(("localhost", 8080)).unwrap().run()
}

#[get("/api/hello")]
async fn hello(session: Session) -> ActixResult<String> {
    if !is_authorised(session)? {
        return Err(ErrorForbidden("Unauthorised"));
    }
    Ok("hello there".to_string())
}

#[get("/api/get_user_info")]
async fn get_user_info(session: Session) -> ActixResult<String> {
    let user = get_user_from_session_cookie(session);
    if let Ok(Some(email)) = user {
        Ok(email)
    } else {
        Ok("anonymous".to_string())
    }
}

fn get_user_from_session_cookie(session: Session) -> AnyhowResult<Option<String>> {
    if let Some(email) = session.get::<String>("user")? {
        if !email.is_empty() {
            Ok(Some(email))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn is_authorised(session: Session) -> ActixResult<bool> {
    let user = get_user_from_session_cookie(session);
    let result = match user {
        Ok(inside) => inside,
        Err(e) => return Err(ErrorInternalServerError(e.to_string())),
    };
    if let Some(_user) = result {
        // check if user in a whitelist?
        Ok(true)
    } else {
        Ok(false)
    }
}

/// if anonymous user that is not logged in, we generate a session with key "anonuser" with a uuid to track him.
/// if he is already logged in?
#[get("/api/trigger_login")]
async fn login(app_state: web::Data<AppState>, session: Session) -> ActixResult<HttpResponse> {
    // if user already logged in, we skip this flow
    if let Some(email) = session.get::<String>("user")? {
        if !email.is_empty() {
            return Ok(HttpResponse::TemporaryRedirect()
                .insert_header(("Location", "/"))
                .body("Already logged in. Redirecting"));
        }
    }

    let anonuser = "anonuser";
    let anonuserid = if let Some(anonuserid) = session.get::<String>(anonuser)? {
        // info!("Anonymous user already has a session_id: {}", anonuserid);
        anonuserid
    } else {
        info!("Anonymous user does NOT have a session_id. Generating one for him");
        let uuid = Uuid::new_v4().to_string();
        session.insert(anonuser, uuid.clone())?;
        uuid
    };
    let (url, csrf_token, nonce) = get_oidc_login().await;
    let mut session_oidc_state = app_state
        .session_oidc_state
        .lock()
        .expect("Expected to be able to lock mutex");
    (*session_oidc_state).insert(anonuserid, (csrf_token, nonce));
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header(("Location", url.to_string()))
        .body("Redirecting to login"))
}

#[derive(Deserialize)]
pub struct Callback {
    pub code: String,
    pub state: String,
}

#[get("/api/token_exchange")]
async fn token_exchange(
    req_body: web::Query<Callback>,
    app_state: web::Data<AppState>,
    session: Session,
) -> ActixResult<HttpResponse> {
    // info!("api/token_exchange has: {:#?}", session.entries());
    match token_exchange_internal(req_body, app_state, session).await {
        Ok(value) => Ok(value),
        Err(e) => Err(ErrorInternalServerError(e.to_string())),
    }
}

async fn token_exchange_internal(
    req_body: web::Query<Callback>,
    app_state: web::Data<AppState>,
    session: Session,
) -> AnyhowResult<HttpResponse> {
    // info!("Session has: {:#?}", session.entries());
    let anonuser = "anonuser";
    let anonuserid = if let Some(uuid) = session.get::<String>(anonuser)? {
        Ok(uuid)
    } else {
        Err(anyhow!("No anonuser uuid in token. Not a valid flow",))
    }?;
    let returned_code = AuthorizationCode::new(req_body.code.to_owned());
    let returned_state = CsrfToken::new(req_body.state.to_owned());
    // Retrieved stored nonce and csrf token
    let mut session_oidc_state = app_state
        .session_oidc_state
        .lock()
        .expect("Expected to be able to lock mutex");
    let (stored_csrf, stored_nonce) =
        if let Some((csrf, nonce)) = session_oidc_state.get(&anonuserid) {
            Ok((csrf, nonce))
        } else {
            Err(anyhow!("State store did not have necessary info",))
        }?;
    // Verify csrf_state
    if returned_state.secret() != stored_csrf.secret() {
        return Err(anyhow!("Failed to verify csrf"));
    }
    // Exchange for a token
    // Need another client here. Copy pasta for now
    let oidc_metadata = get_oidc_metadata().await;
    let provider_metadata =
        CoreProviderMetadata::discover_async(oidc_metadata.issuer_url, http_client)
            .await
            .unwrap();
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        oidc_metadata.client_id,
        Some(oidc_metadata.client_secret),
    )
    .set_redirect_uri(RedirectUrl::new(oidc_metadata.redirect_url).expect("Invalid redirect URL"));
    // Do token exchange
    let token_response = client
        .exchange_code(returned_code)
        .request_async(http_client)
        .await
        .context("Failed to exchange token")?;
    let id_token_verifier = client.id_token_verifier();
    let id_token = if let Some(id_token) = token_response.extra_fields().id_token() {
        Ok(id_token)
    } else {
        Err(anyhow!("Empty id token"))
    }?;

    let id_token_claims = id_token
        .claims(&id_token_verifier, stored_nonce)
        .context("Failed to verify id token claims")?;
    let email = if let Some(email) = id_token_claims.email() {
        Ok(email)
    } else {
        Err(anyhow!("No email found in claims"))
    }?;

    // clean up both cookie and internal state
    (*session_oidc_state).remove(&anonuserid);
    session.insert("user", email)?;
    session.remove(anonuser);

    // redirect
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header(("Location", "/"))
        .body("Redirecting to login"))
}

struct OidcMetadata {
    client_id: ClientId,
    client_secret: ClientSecret,
    issuer_url: IssuerUrl,
    redirect_url: String,
}

async fn get_oidc_metadata() -> OidcMetadata {
    let client_id = ClientId::new(
        env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let client_secret = ClientSecret::new(
        env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );
    let issuer_url =
        IssuerUrl::new("https://accounts.google.com".to_string()).expect("Invalid issuer URL");
    let redirect_url = "http://localhost:8080/api/token_exchange".to_string();

    OidcMetadata {
        client_id,
        client_secret,
        issuer_url,
        redirect_url,
    }
}

async fn get_oidc_login() -> (Url, CsrfToken, Nonce) {
    let oidc_metadata = get_oidc_metadata().await;
    // Set up the config for the Google OAuth2 process.
    let provider_metadata =
        CoreProviderMetadata::discover_async(oidc_metadata.issuer_url, http_client)
            .await
            .unwrap();

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        oidc_metadata.client_id,
        Some(oidc_metadata.client_secret),
    )
    .set_redirect_uri(RedirectUrl::new(oidc_metadata.redirect_url).expect("Invalid redirect URL"));

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

#[get("/api/trigger_logout")]
async fn logout(session: Session) -> ActixResult<HttpResponse> {
    // if user already logged in, we clear his session token
    let user_key = "user";
    if (session.get::<String>(user_key)?).is_some() {
        session.remove(user_key);
    }
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header(("Location", "/"))
        .body("Logged out. Redirecting"))
}
