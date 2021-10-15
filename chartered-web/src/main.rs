#![deny(clippy::pedantic)]
#![deny(rust_2018_idioms)]
#![allow(clippy::module_name_repetitions)]

mod config;
mod endpoints;
mod middleware;

use axum::{
    handler::get,
    http::{header, Method},
    AddExtensionLayer, Router,
};
use clap::Clap;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clap)]
#[clap(version = clap::crate_version!(), author = clap::crate_authors!())]
#[clap(setting = clap::AppSettings::ColoredHelp)]
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

// there's some sort of issue with monomorphization of axum routes
// which causes compile times to increase exponentially with every
// new route, the workaround is to box the router down to a
// dynamically dispatched version with every new route.
macro_rules! axum_box_after_every_route {
    (Router::new()
        $(.nest($nest_path:expr, $nest_svc:expr$(,)?))*
        $(.route($route_path:expr, $route_svc:expr$(,)?))*
    ) => {
        Router::new()
            $(
                .nest($nest_path, $nest_svc)
                .boxed()
            )*
            $(
                .route($route_path, $route_svc)
                .boxed()
            )*
    };
}

pub(crate) use axum_box_after_every_route;

#[tokio::main]
#[allow(clippy::semicolon_if_nothing_returned)] // lint breaks with tokio::main
async fn main() {
    let opts: Opts = Opts::parse();
    let config: config::Config = toml::from_slice(&std::fs::read(&opts.config).unwrap()).unwrap();

    tracing_subscriber::fmt::init();

    let pool = chartered_db::init().unwrap();

    let middleware_stack = ServiceBuilder::new()
        .layer_fn(middleware::logging::LoggingMiddleware)
        .into_inner();

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
        // TODO!!!
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
                .allow_origin(Any)
                .allow_credentials(false),
        )
        .layer(AddExtensionLayer::new(pool))
        .layer(AddExtensionLayer::new(Arc::new(
            config.create_oidc_clients().await.unwrap(),
        )))
        .layer(AddExtensionLayer::new(Arc::new(config)));

    axum::Server::bind(&"0.0.0.0:8888".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr, _>())
        .await
        .unwrap();
}
