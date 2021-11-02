#![deny(clippy::pedantic)]
#![deny(rust_2018_idioms)]
#![allow(clippy::module_name_repetitions)]

mod config;
mod endpoints;
mod middleware;

use axum::{
    http::{header, Method},
    routing::get,
    AddExtensionLayer, Router,
};
use clap::Parser;
use std::{fmt::Formatter, path::PathBuf, sync::Arc};
use thiserror::Error;
use tower::ServiceBuilder;
use tower_http::cors::{CorsLayer, Origin};
use tracing::info;
use url::Url;

#[derive(Parser)]
#[clap(version = clap::crate_version!(), author = clap::crate_authors!())]
pub struct Opts {
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    #[clap(short, long)]
    config: PathBuf,
}

#[allow(clippy::unused_async)]
async fn hello_world() -> &'static str {
    "hello, world!"
}

#[tokio::main]
#[allow(clippy::semicolon_if_nothing_returned)] // lint breaks with tokio::main
async fn main() -> Result<(), InitError> {
    // parse CLI arguments
    let opts: Opts = Opts::parse();

    // overrides the RUST_LOG variable to our own value based on the
    // amount of `-v`s that were passed when calling the service
    std::env::set_var(
        "RUST_LOG",
        match opts.verbose {
            1 => "debug",
            2 => "trace",
            _ => "info",
        },
    );

    let config: config::Config = toml::from_slice(&std::fs::read(&opts.config)?)?;

    // initialise logging/tracing
    tracing_subscriber::fmt::init();

    let bind_address = config.bind_address;
    let pool = chartered_db::init(&config.database_uri)?;

    // the base stack of middleware that is applied to _all_ routes
    let middleware_stack = ServiceBuilder::new()
        .layer_fn(middleware::logging::LoggingMiddleware)
        .into_inner();

    let config = Arc::new(config);

    let app = Router::new()
        .route("/", get(hello_world))
        .nest(
            "/a/:key/web/v1",
            endpoints::web_api::authenticated_routes().layer(
                ServiceBuilder::new()
                    .layer_fn(crate::middleware::auth::AuthMiddleware)
                    .into_inner(),
            ),
        )
        .nest("/a/-/web/v1", endpoints::web_api::unauthenticated_routes())
        .nest(
            "/a/:key/o/:organisation/api/v1",
            endpoints::cargo_api::routes().layer(
                ServiceBuilder::new()
                    .layer_fn(crate::middleware::auth::AuthMiddleware)
                    .into_inner(),
            ),
        )
        .layer(middleware_stack)
        .layer(
            CorsLayer::new()
                .allow_methods(vec![
                    Method::GET,
                    Method::POST,
                    Method::PATCH,
                    Method::DELETE,
                    Method::PUT,
                    Method::OPTIONS,
                ])
                .allow_headers(vec![header::CONTENT_TYPE, header::USER_AGENT])
                .allow_origin(Origin::predicate({
                    let config = config.clone();
                    move |url, _| {
                        url.to_str()
                            .ok()
                            .and_then(|url| Url::parse(url).ok())
                            .map(|url| url.host_str() == config.frontend_base_uri.host_str())
                            .unwrap_or_default()
                    }
                }))
                .allow_credentials(false),
        )
        .layer(AddExtensionLayer::new(pool))
        .layer(AddExtensionLayer::new(Arc::new(
            config.create_oidc_clients().await?,
        )))
        .layer(AddExtensionLayer::new(Arc::new(
            config.get_file_system().await?,
        )))
        .layer(AddExtensionLayer::new(config.clone()));

    info!("HTTP server listening on {}", bind_address);

    axum::Server::bind(&bind_address)
        .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr, _>())
        .await
        .map_err(|e| InitError::ServerSpawn(Box::new(e)))?;

    Ok(())
}

#[derive(Error)]
pub enum InitError {
    #[error("Failed to read configuration: {0}")]
    ConfigRead(#[from] std::io::Error),
    #[error("Failed to parse configuration: {0}")]
    ConfigParse(#[from] toml::de::Error),
    #[error("Configuration error: {0}")]
    Config(#[from] config::Error),
    #[error("Database error: {0}")]
    Database(#[from] chartered_db::Error),
    #[error("Failed to spawn HTTP server: {0}")]
    ServerSpawn(Box<dyn std::error::Error>),
    #[error("Failed to build CORS header: {0}")]
    Cors(axum::http::header::InvalidHeaderValue),
}

impl std::fmt::Debug for InitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
