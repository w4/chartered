//! Methods for `OpenID` Connect authentication, we allow the frontend to list all the available and
//! enabled providers so they can show them to the frontend and provide methods for actually doing
//! the authentication.

use crate::config::{Config, OidcClients};
use axum::{extract, Json};
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit, Nonce as ChaCha20Poly1305Nonce};
use chartered_db::{users::User, ConnectionPool};
use openid::{Options, Token};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

pub type Nonce = [u8; 16];

/// Lists all the available and enabled providers that the user can authenticate with.
#[allow(clippy::unused_async)]
pub async fn list_providers(
    extract::Extension(oidc_clients): extract::Extension<Arc<OidcClients>>,
) -> Json<ListProvidersResponse> {
    Json(ListProvidersResponse {
        providers: oidc_clients
            .keys()
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect(),
    })
}

/// Starts the authentication process, generating an encrypted state so we can validate the
/// request came from us and returning back the URL the frontend should redirect the user for
/// authenticating with the provider.
#[allow(clippy::unused_async)]
pub async fn begin_oidc(
    extract::Path(provider): extract::Path<String>,
    extract::Extension(config): extract::Extension<Arc<Config>>,
    extract::Extension(oidc_clients): extract::Extension<Arc<OidcClients>>,
) -> Result<Json<BeginResponse>, Error> {
    let client = oidc_clients
        .get(&provider)
        .ok_or(Error::UnknownOauthProvider)?;

    let nonce = rand::random::<Nonce>();
    let state = serde_json::to_vec(&State { provider, nonce })?;

    let auth_url = client.auth_url(&Options {
        scope: Some("openid email profile".into()),
        nonce: Some(base64::encode_config(&nonce, base64::URL_SAFE_NO_PAD)),
        state: Some(encrypt_url_safe(&state, &config)?),
        ..Options::default()
    });

    Ok(Json(BeginResponse {
        redirect_url: auth_url.to_string(),
    }))
}

/// Handles the response back from the OIDC provider, checking the state came from us, validating
/// the token with the provider themselves and then finally logging the user in.
pub async fn complete_oidc(
    extract::Query(params): extract::Query<CompleteOidcParams>,
    extract::Extension(config): extract::Extension<Arc<Config>>,
    extract::Extension(oidc_clients): extract::Extension<Arc<OidcClients>>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    user_agent: Option<extract::TypedHeader<headers::UserAgent>>,
    addr: extract::ConnectInfo<std::net::SocketAddr>,
) -> Result<Json<super::LoginResponse>, Error> {
    // decrypt the state that we created in `begin_oidc` and parse it as json
    let state: State = serde_json::from_slice(&decrypt_url_safe(&params.state, &config)?)?;

    // check the state for the provider so we can get the right OIDC client to
    // verify the code and grab an id_token
    let client = oidc_clients
        .get(&state.provider)
        .ok_or(Error::UnknownOauthProvider)?;

    let mut token: Token = client.request_token(&params.code).await?.into();

    if let Some(id_token) = token.id_token.as_mut() {
        // ensure the id_token is valid, checking `exp`, etc.
        client.decode_token(id_token)?;

        // ensure the nonce in the returned id_token is the same as the one we sent out encrypted
        // with the original request
        let nonce = base64::encode_config(state.nonce, base64::URL_SAFE_NO_PAD);
        client.validate_token(id_token, Some(nonce.as_str()), None)?;
    } else {
        // the provider didn't send us back a id_token
        return Err(Error::MissingToken);
    }

    // get some basic info from the provider using the claims we requested in `begin_oidc`
    let userinfo = client.request_userinfo(&token).await?;

    let user = User::find_or_create(
        db.clone(),
        // we're using `provider:uid` as the format for OIDC logins, this is fine to create
        // without a password because (1) password auth rejects blank passwords and (2) password
        // auth also rejects any usernames with a `:` in.
        format!("{}:{}", state.provider, userinfo.sub.unwrap()),
        userinfo.name,
        userinfo.nickname,
        userinfo.email,
        userinfo.profile,
        userinfo.picture,
    )
    .await?;

    // request looks good, log the user in!
    Ok(Json(super::login(db, user, user_agent, addr).await?))
}

const NONCE_LEN: usize = 12;

// Encrypts the given string using ChaCha20Poly1305 and returns a url safe base64 encoded
// version of it
fn encrypt_url_safe(input: &[u8], config: &Config) -> Result<String, Error> {
    let cipher = ChaCha20Poly1305::new(&config.encryption_key);

    let nonce = rand::random::<[u8; NONCE_LEN]>();
    let nonce = ChaCha20Poly1305Nonce::from_slice(&nonce);

    let mut ciphertext = cipher.encrypt(nonce, input)?;
    ciphertext.extend_from_slice(nonce);

    Ok(base64::encode_config(&ciphertext, base64::URL_SAFE_NO_PAD))
}

// Decrypts the given string assuming it's a url safe base64 encoded ChaCha20Poly1305 cipher.
fn decrypt_url_safe(input: &str, config: &Config) -> Result<Vec<u8>, Error> {
    let cipher = ChaCha20Poly1305::new(&config.encryption_key);

    let mut ciphertext = base64::decode_config(input, base64::URL_SAFE_NO_PAD)?;
    let ciphertext_nonce = ciphertext.split_off(ciphertext.len() - NONCE_LEN);
    let ciphertext_nonce = ChaCha20Poly1305Nonce::from_slice(&ciphertext_nonce);

    cipher
        .decrypt(ciphertext_nonce, ciphertext.as_ref())
        .map_err(Error::from)
}

#[derive(Serialize)]
pub struct ListProvidersResponse {
    providers: Vec<String>,
}

#[derive(Serialize)]
pub struct BeginResponse {
    redirect_url: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct CompleteOidcParams {
    state: String,
    code: String,
    scope: Option<String>,
    prompt: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    provider: String,
    nonce: Nonce,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] chartered_db::Error),
    #[error("Error serialising/deserialsing state: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Unknown OAuth provider given")]
    UnknownOauthProvider,
    #[error("{0}")]
    OAuth(#[from] openid::error::Error),
    #[error("{0}")]
    OAuthClient(#[from] openid::error::ClientError),
    #[error("Error during encryption/decryption")]
    Cipher(#[from] chacha20poly1305::aead::Error),
    #[error("Base64 error")]
    Base64(#[from] base64::DecodeError),
    #[error("Missing id_token")]
    MissingToken,
}

impl Error {
    fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(e) => e.status_code(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

define_error_response!(Error);
