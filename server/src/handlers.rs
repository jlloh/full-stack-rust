use crate::{
    auth::{get_oidc_login, is_authorised, token_exchange_internal, Callback},
    AppState,
};
use crate::{database, OidcMetadata};
use actix_session::Session;
use actix_web::error::ErrorInternalServerError;
use actix_web::{get, post, Responder, Result as ActixResult};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_lab::sse;
use anyhow::Result as AnyhowResult;
use common::{ServerSentData, UserInfo};
use log::{error, info};
use std::time::Duration;
use uuid::Uuid;

//https://github.com/rishadbaniya/Actix-web-SPA-react-js-example/blob/master/src/main.rs
pub async fn spa_index() -> ActixResult<actix_files::NamedFile> {
    match actix_files::NamedFile::open("./dist/index.html") {
        Ok(response) => Ok(response),
        Err(e) => Err(ErrorInternalServerError(e.to_string())),
    }
}

#[get("/api/hello")]
async fn hello(
    app_state: web::Data<AppState>,
    session: Session,
    request: HttpRequest,
) -> ActixResult<String> {
    let user = is_authorised(&session, &app_state.authz_enforcer, request)?;
    Ok(format!("hello there {}", user.email))
}

/// Endpoint to get user info, e.g. his username, etc.
#[get("/public/get_user_info2")]
async fn get_user_info2(
    app_state: web::Data<AppState>,
    session: Session,
    request: HttpRequest,
) -> ActixResult<web::Json<UserInfo>> {
    let mut user = is_authorised(&session, &app_state.authz_enforcer, request)?;
    info!("{:#?}", user);

    // Get user assigned queue number if it exists
    // TODO: Should we store assigned queue number in the cookie as well? Probably not
    // let db_connection = &mut app_state.db_connection_pool.get().unwrap();
    // let user_assigned_number = wrap_internal_server_error(database::get_user_assigned_queue(
    //     db_connection,
    //     &user.email,
    // ))?;
    // user.assigned_number = user_assigned_number;

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
#[get("/public/{subapp}/trigger_login")]
async fn login(
    app_state: web::Data<AppState>,
    session: Session,
    info: web::Path<(String,)>,
    request: HttpRequest,
) -> ActixResult<HttpResponse> {
    let subapp = info.into_inner().0;
    let base_url = format!("/{}", subapp);
    // authorisation of public endpoints unnecessary? but good hygiene I guess
    is_authorised(&session, &app_state.authz_enforcer, request)?;
    // if user already logged in, we skip this flow
    if let Some(email) = session.get::<String>("user")? {
        if !email.is_empty() {
            info!("User already logged in");
            return Ok(HttpResponse::TemporaryRedirect()
                .insert_header(("Location", base_url))
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
    let oidc_metadata = OidcMetadata {
        csrf_token,
        nonce,
        subapp,
    };
    (*session_oidc_state).insert(anonuserid, oidc_metadata);
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header(("Location", url.to_string()))
        .body("Redirecting to login"))
}

#[get("/api/{subapp}/trigger_logout")]
async fn logout(
    app_state: web::Data<AppState>,
    session: Session,
    info: web::Path<(String,)>,
    request: HttpRequest,
) -> ActixResult<HttpResponse> {
    let subapp = info.into_inner().0;
    let base_url = format!("/{}", subapp);
    is_authorised(&session, &app_state.authz_enforcer, request)?;
    // if user already logged in, we clear his session token
    let user_key = "user";
    if (session.get::<String>(user_key)?).is_some() {
        // session.remove(user_key);
        session.clear();
    }
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header(("Location", base_url))
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

#[get("/public/get_selected_number")]
async fn get_selected_number(
    app_state: web::Data<AppState>,
    session: Session,
    request: HttpRequest,
) -> ActixResult<web::Json<ServerSentData>> {
    let user = is_authorised(&session, &app_state.authz_enforcer, request)?;
    let db_connection = &mut app_state.db_connection_pool.get().unwrap();
    let selected_number = wrap_internal_server_error(database::get_selected_queue(db_connection))?;
    let assigned_number = wrap_internal_server_error(database::get_user_assigned_queue(
        db_connection,
        &user.email,
    ))?;
    Ok(web::Json(ServerSentData {
        selected_number,
        assigned_number,
    }))
    // match selected_number {
    //     Some(number) => Ok(number.to_string()),
    //     None => Ok("None".to_string()),
    // }
}

/// API to add new queue
#[post("/api/get_new_number")]
async fn get_new_number(
    app_state: web::Data<AppState>,
    session: Session,
    request: HttpRequest,
) -> ActixResult<web::Json<UserInfo>> {
    let user = is_authorised(&session, &app_state.authz_enforcer, request)?;
    let db_connection = &mut app_state.db_connection_pool.get().unwrap();
    let user_info = wrap_internal_server_error(database::get_or_insert(db_connection, user))?;
    Ok(web::Json(user_info))
}

/// API to get assigned number if it exists
#[get("/api/get_assigned_number")]
async fn get_assigned_number(
    app_state: web::Data<AppState>,
    session: Session,
    request: HttpRequest,
) -> ActixResult<String> {
    let user = is_authorised(&session, &app_state.authz_enforcer, request)?;
    let db_connection = &mut app_state.db_connection_pool.get().unwrap();
    let _assigned_number = wrap_internal_server_error(database::get_user_assigned_queue(
        db_connection,
        &user.email,
    ))?;
    Ok("Ok".to_string())
}

/// API to abandon number
#[post("/api/abandon_assigned_number")]
async fn abandon_assigned_number(
    app_state: web::Data<AppState>,
    session: Session,
    request: HttpRequest,
) -> ActixResult<String> {
    let user = is_authorised(&session, &app_state.authz_enforcer, request)?;
    let db_connection = &mut app_state.db_connection_pool.get().unwrap();
    wrap_internal_server_error(database::set_to_abandoned(db_connection, user))?;
    Ok("Ok".to_string())
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

// utils
fn wrap_internal_server_error<T>(result: AnyhowResult<T>) -> ActixResult<T> {
    match result {
        Ok(v) => Ok(v),
        Err(e) => {
            let error = format!("Internal server error with error: {:#}", e);
            error!("{}", error);
            Err(ErrorInternalServerError(error))
        }
    }
}
