use crate::AppState;
use actix_session::Session;
use actix_web::error::{ErrorForbidden, ErrorInternalServerError};
use actix_web::{web, HttpResponse, Result as ActixResult};
use anyhow::{anyhow, Context, Result as AnyhowResult};
use openidconnect::AuthorizationCode;
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    reqwest::async_http_client as http_client,
    url::Url,
    AuthenticationFlow, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, RedirectUrl, Scope,
};
use serde::Deserialize;

pub fn get_user_from_session_cookie(session: Session) -> AnyhowResult<Option<String>> {
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

pub fn is_authorised(session: Session) -> ActixResult<String> {
    let user = get_user_from_session_cookie(session);
    let result = match user {
        Ok(inside) => inside,
        Err(e) => return Err(ErrorInternalServerError(e.to_string())),
    };
    if let Some(user) = result {
        // check if user in a whitelist?
        Ok(user)
    } else {
        Err(ErrorForbidden("Unauthorised"))
    }
}

#[derive(Deserialize)]
pub struct Callback {
    pub code: String,
    pub state: String,
}

pub async fn token_exchange_internal(
    client_id: String,
    client_secret: String,
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
    let mut session_oidc_state = app_state.session_oidc_state.lock().await;
    // .expect("Expected to be able to lock mutex");
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
    let oidc_metadata = get_oidc_metadata(client_id, client_secret).await;
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

async fn get_oidc_metadata(client_id: String, client_secret: String) -> OidcMetadata {
    let client_id = ClientId::new(client_id);
    let client_secret = ClientSecret::new(client_secret);
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

pub async fn get_oidc_login(client_id: String, client_secret: String) -> (Url, CsrfToken, Nonce) {
    let oidc_metadata = get_oidc_metadata(client_id, client_secret).await;
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
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    (authorize_url, csrf_state, nonce)
}
