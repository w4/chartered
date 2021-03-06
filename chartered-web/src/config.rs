use chacha20poly1305::Key as ChaCha20Poly1305Key;
use chartered_fs::FileSystem;
use openid::DiscoveredClient;
use serde::{de::Error as SerdeDeError, Deserialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error discovering OpenID provider: {0}")]
    OpenId(#[from] openid::error::Error),
    #[error("Failed to create file system handle: {0}")]
    Fs(#[from] Box<chartered_fs::Error>),
    #[error("Failed to build URL: {0}")]
    Parse(#[from] url::ParseError),
}

pub type OidcClients = HashMap<String, DiscoveredClient>;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bind_address: SocketAddr,
    pub database_uri: String,
    pub storage_uri: String,
    pub frontend_base_uri: Url,
    pub auth: AuthConfig,
    #[serde(deserialize_with = "deserialize_encryption_key")]
    pub encryption_key: ChaCha20Poly1305Key,
}

impl Config {
    pub async fn get_file_system(&self) -> Result<FileSystem, Error> {
        Ok(FileSystem::from_str(&self.storage_uri)
            .await
            .map_err(Box::new)?)
    }

    pub async fn create_oidc_clients(&self) -> Result<OidcClients, Error> {
        Ok(futures::future::try_join_all(
            self.auth
                .oauth
                .iter()
                .filter(|(_, config)| config.enabled)
                .map(|(name, config)| async move {
                    let redirect = self.frontend_base_uri.join("auth/login/oauth")?;

                    Ok::<_, Error>((
                        name.to_string(),
                        DiscoveredClient::discover(
                            config.client_id.to_string(),
                            config.client_secret.to_string(),
                            Some(redirect.to_string()),
                            config.discovery_uri.clone(),
                        )
                        .await?,
                    ))
                }),
        )
        .await?
        .into_iter()
        .collect())
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct AuthConfig {
    pub password: PasswordAuthConfig,
    #[serde(flatten)]
    pub oauth: HashMap<String, OAuthConfig>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct PasswordAuthConfig {
    pub enabled: bool,
}

#[derive(Deserialize, Debug)]
pub struct OAuthConfig {
    pub enabled: bool,
    pub discovery_uri: Url,
    pub client_id: String,
    pub client_secret: String,
}

fn deserialize_encryption_key<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<ChaCha20Poly1305Key, D::Error> {
    let key = String::deserialize(deserializer)?;

    if key.as_bytes().len() != 32 {
        return Err(D::Error::custom("encryption_key must be 32 bytes"));
    }

    Ok(ChaCha20Poly1305Key::clone_from_slice(key.as_bytes()))
}
