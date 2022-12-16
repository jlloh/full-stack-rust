use crate::{
    auth::{
        get_oidc_login, get_user_from_session_cookie, is_authorised, token_exchange_internal,
        Callback,
    },
    AppState,
};
use actix_session::Session;
use actix_web::error::ErrorInternalServerError;
use actix_web::{get, Result as ActixResult};
use actix_web::{web, HttpResponse};
use log::info;
use uuid::Uuid;

#[get("/api/hello")]
async fn hello(session: Session) -> ActixResult<String> {
    let user = is_authorised(session)?;
    Ok(format!("hello there {}", user))
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

#[get("/api/token_exchange")]
async fn token_exchange(
    req_body: web::Query<Callback>,
    app_state: web::Data<AppState>,
    session: Session,
) -> ActixResult<HttpResponse> {
    // info!("api/token_exchange has: {:#?}", session.entries());
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
