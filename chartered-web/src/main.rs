#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod endpoints;
mod middleware;

use axum::{
    body::Body,
    handler::{delete, get, post, put},
    http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
    http::Method,
    AddExtensionLayer, Router,
};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;

#[allow(clippy::unused_async)]
async fn hello_world() -> &'static str {
    "hello, world!"
}

// there's some sort of issue with monomorphization of axum routes
// which causes compile times to increase exponentially with every
// new route, the workaround is to box the router down to a
// dynamically dispatched version with every new route.
macro_rules! axum_box_after_every_route {
    (Router::new()$(.route($path:expr, $svc:expr$(,)?))*) => {
        Router::new()
            $(
                .route($path, $svc)
                .boxed()
            )*
    };
}

#[tokio::main]
#[allow(clippy::semicolon_if_nothing_returned)] // lint breaks with tokio::main
async fn main() {
    env_logger::init();

    let pool = chartered_db::init().unwrap();

    let api_authenticated = axum_box_after_every_route!(Router::new()
        .route("/crates/new", put(endpoints::cargo_api::publish))
        .route("/crates/search", get(hello_world))
        .route(
            "/crates/:crate/owners",
            get(endpoints::cargo_api::get_owners)
        )
        .route("/crates/:crate/owners", put(hello_world))
        .route("/crates/:crate/owners", delete(hello_world))
        .route(
            "/crates/:crate/:version/yank",
            delete(endpoints::cargo_api::yank)
        )
        .route(
            "/crates/:crate/:version/unyank",
            put(endpoints::cargo_api::unyank)
        )
        .route(
            "/crates/:crate/:version/download",
            get(endpoints::cargo_api::download)
        ))
    .layer(
        ServiceBuilder::new()
            .layer_fn(middleware::auth::AuthMiddleware)
            .into_inner(),
    );

    let web_unauthenticated =
        axum_box_after_every_route!(Router::new().route("/login", post(endpoints::web_api::login)));

    let web_authenticated = axum_box_after_every_route!(
        Router::new().route("/crates/:crate", get(endpoints::web_api::crate_info))
    )
    .layer(
        ServiceBuilder::new()
            .layer_fn(middleware::auth::AuthMiddleware)
            .into_inner(),
    );

    let middleware_stack = ServiceBuilder::new()
        .layer_fn(middleware::logging::LoggingMiddleware)
        .into_inner();

    let app = Router::new()
        .route("/", get(hello_world))
        .nest("/a/:key/web/v1", web_authenticated)
        .nest("/a/-/web/v1", web_unauthenticated)
        .nest("/a/:key/api/v1", api_authenticated)
        .layer(middleware_stack)
        // TODO!!!
        .layer(
            CorsLayer::new()
                .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
                .allow_origin(Any)
                .allow_credentials(false),
        )
        .layer(AddExtensionLayer::new(pool));

    axum::Server::bind(&"0.0.0.0:8888".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr, _>())
        .await
        .unwrap();
}
