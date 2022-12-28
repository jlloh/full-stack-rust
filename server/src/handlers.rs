use crate::{
    auth::{get_oidc_login, is_authorised, token_exchange_internal, Callback},
    AppState,
};
use actix_session::Session;
use actix_web::error::ErrorInternalServerError;
use actix_web::{get, Responder, Result as ActixResult};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_lab::sse;
use common::UserInfo;
use log::info;
use std::time::Duration;
use uuid::Uuid;

#[get("/api/hello")]
async fn hello(
    app_state: web::Data<AppState>,
    session: Session,
    request: HttpRequest,
) -> ActixResult<String> {
    let user = is_authorised(&session, &app_state.authz_enforcer, request)?;
    Ok(format!("hello there {}", user.email))
}

#[get("/public/get_user_info2")]
async fn get_user_info2(
    app_state: web::Data<AppState>,
    session: Session,
    request: HttpRequest,
) -> ActixResult<web::Json<UserInfo>> {
    let user = is_authorised(&session, &app_state.authz_enforcer, request)?;
    Ok(web::Json(user))
}

#[get("/public/token_exchange")]
async fn token_exchange(
    req_body: web::Query<Callback>,
    app_state: web::Data<AppState>,
    session: Session,
    request: HttpRequest,
) -> ActixResult<HttpResponse> {
    is_authorised(&session, &app_state.authz_enforcer, request)?;
    match token_exchange_internal(
        app_state.client_id.clone(),
        app_state.client_secret.clone(),
        req_body,
        app_state,
        session,
    )
    .await
    {
        Ok(value) => Ok(value),
        Err(e) => Err(ErrorInternalServerError(e.to_string())),
    }
}

/// if anonymous user that is not logged in, we generate a session with key "anonuser" with a uuid to track him.
/// if he is already logged in?
#[get("/public/trigger_login")]
async fn login(
    app_state: web::Data<AppState>,
    session: Session,
    request: HttpRequest,
) -> ActixResult<HttpResponse> {
    // authorisation of public endpoints unnecessary? but good hygiene I guess
    is_authorised(&session, &app_state.authz_enforcer, request)?;
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
    let (url, csrf_token, nonce) =
        get_oidc_login(app_state.client_id.clone(), app_state.client_secret.clone()).await;
    let mut session_oidc_state = app_state.session_oidc_state.lock().await;
    // .expect("Expected to be able to lock mutex");
    (*session_oidc_state).insert(anonuserid, (csrf_token, nonce));
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header(("Location", url.to_string()))
        .body("Redirecting to login"))
}

#[get("/api/trigger_logout")]
async fn logout(
    app_state: web::Data<AppState>,
    session: Session,
    request: HttpRequest,
) -> ActixResult<HttpResponse> {
    is_authorised(&session, &app_state.authz_enforcer, request)?;
    // if user already logged in, we clear his session token
    let user_key = "user";
    if (session.get::<String>(user_key)?).is_some() {
        // session.remove(user_key);
        session.clear();
    }
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header(("Location", "/"))
        .body("Logged out. Redirecting"))
}

#[get("/public/subscribe")]
async fn subscribe(
    app_state: web::Data<AppState>,
    session: Session,
    request: HttpRequest,
) -> impl Responder {
    match is_authorised(&session, &app_state.authz_enforcer, request) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }
    info!("Subscriber added");
    let (sender, receiver) = sse::channel(10);
    let mut sse_senders = app_state.sse_senders.lock().await;
    (*sse_senders).push(sender);
    Ok(receiver.with_retry_duration(Duration::from_secs(10)))
}

// Admin handlers
#[get("/admin/test")]
async fn admin_test(
    app_state: web::Data<AppState>,
    session: Session,
    request: HttpRequest,
) -> ActixResult<String> {
    is_authorised(&session, &app_state.authz_enforcer, request)?;
    Ok("Admin Endpoint".to_string())
}
