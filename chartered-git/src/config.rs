use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bind_address: SocketAddr,
    pub database_uri: String,
}
