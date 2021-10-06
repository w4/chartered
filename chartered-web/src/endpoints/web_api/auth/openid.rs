use crate::config::{Config, OidcClients};
use axum::{extract, Json};
use chacha20poly1305::{
    aead::{Aead, NewAead},
    ChaCha20Poly1305, Nonce as ChaCha20Poly1305Nonce,
};
use chartered_db::{users::User, ConnectionPool};
use openid::{Options, Token};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

pub type Nonce = [u8; 16];

#[derive(Serialize)]
pub struct ListProvidersResponse {
    providers: Vec<String>,
}

pub async fn list_providers(
    extract::Extension(oidc_clients): extract::Extension<Arc<OidcClients>>,
) -> Json<ListProvidersResponse> {
    Json(ListProvidersResponse {
        providers: oidc_clients
            .keys()
            .into_iter()
            .map(|v| v.to_string())
            .collect(),
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    provider: String,
    nonce: Nonce,
}

#[derive(Serialize)]
pub struct BeginResponse {
    redirect_url: String,
}

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
        ..Default::default()
    });

    Ok(Json(BeginResponse {
        redirect_url: auth_url.to_string(),
    }))
}

#[derive(Deserialize)]
pub struct CompleteOidcParams {
    state: String,
    code: String,
    scope: Option<String>,
    prompt: Option<String>,
}

pub async fn complete_oidc(
    extract::Query(params): extract::Query<CompleteOidcParams>,
    extract::Extension(config): extract::Extension<Arc<Config>>,
    extract::Extension(oidc_clients): extract::Extension<Arc<OidcClients>>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    user_agent: Option<extract::TypedHeader<headers::UserAgent>>,
    addr: extract::ConnectInfo<std::net::SocketAddr>,
) -> Result<Json<super::LoginResponse>, Error> {
    let state: State = serde_json::from_slice(&decrypt_url_safe(&params.state, &config)?)?;

    let client = oidc_clients
        .get(&state.provider)
        .ok_or(Error::UnknownOauthProvider)?;

    let mut token: Token = client.request_token(&params.code).await?.into();

    if let Some(mut id_token) = token.id_token.as_mut() {
        client.decode_token(&mut id_token)?;

        let nonce = base64::encode_config(state.nonce, base64::URL_SAFE_NO_PAD);
        client.validate_token(&id_token, Some(nonce.as_str()), None)?;
    } else {
        return Err(Error::MissingToken);
    }

    let userinfo = client.request_userinfo(&token).await?;

    let user = User::find_or_create(
        db.clone(),
        format!("{}:{}", state.provider, userinfo.sub.unwrap()),
        userinfo.name,
        userinfo.nickname,
        userinfo.email,
        userinfo.profile,
        userinfo.picture,
    )
    .await?;

    Ok(Json(super::login(db, user, user_agent, addr).await?))
}

const NONCE_LEN: usize = 12;

fn encrypt_url_safe(input: &[u8], config: &Config) -> Result<String, Error> {
    let cipher = ChaCha20Poly1305::new(&config.encryption_key);

    let nonce = rand::random::<[u8; NONCE_LEN]>();
    let nonce = ChaCha20Poly1305Nonce::from_slice(&nonce);

    let mut ciphertext = cipher.encrypt(nonce, input)?;
    ciphertext.extend_from_slice(&nonce);

    Ok(base64::encode_config(&ciphertext, base64::URL_SAFE_NO_PAD))
}

fn decrypt_url_safe(input: &str, config: &Config) -> Result<Vec<u8>, Error> {
    let cipher = ChaCha20Poly1305::new(&config.encryption_key);

    let mut ciphertext = base64::decode_config(input, base64::URL_SAFE_NO_PAD)?;
    let ciphertext_nonce = ciphertext.split_off(ciphertext.len() - NONCE_LEN);
    let ciphertext_nonce = ChaCha20Poly1305Nonce::from_slice(&ciphertext_nonce);

    cipher
        .decrypt(ciphertext_nonce, ciphertext.as_ref())
        .map_err(Error::from)
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
