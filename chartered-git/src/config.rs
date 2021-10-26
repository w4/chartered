use serde::Deserialize;
use std::net::SocketAddr;
use url::Url;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bind_address: SocketAddr,
    pub database_uri: String,
    pub web_base_uri: Url,
    #[serde(default)]
    pub committer: GitCommitter,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitCommitter {
    pub name: String,
    pub email: String,
    pub message: String,
}

impl Default for GitCommitter {
    fn default() -> Self {
        Self {
            name: "chartered".to_string(),
            email: "noreply@chart.rs".to_string(),
            message: "Update crates".to_string(),
        }
    }
}
