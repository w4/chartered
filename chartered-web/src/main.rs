#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod endpoints;
mod middleware;

use axum::{
    handler::{delete, get, put},
    AddExtensionLayer, Router,
};
use tower::ServiceBuilder;

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
    )
    .layer(AddExtensionLayer::new(pool));

    let middleware_stack = ServiceBuilder::new()
        .layer_fn(middleware::logging::LoggingMiddleware)
        .into_inner();

    let app = Router::new()
        .route("/", get(hello_world))
        .nest("/a/:key/api/v1", api_authenticated)
        .layer(middleware_stack);

    axum::Server::bind(&"0.0.0.0:8888".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr, _>())
        .await
        .unwrap();
}
