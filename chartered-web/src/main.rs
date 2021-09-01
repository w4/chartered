mod endpoints;
mod middleware;

use axum::{
    handler::{delete, get, put},
    Router,
};
use tower::{filter::AsyncFilterLayer, ServiceBuilder};

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
async fn main() {
    let api_authenticated = axum_box_after_every_route!(Router::new()
        .route("/crates/new", put(endpoints::cargo_api::publish))
        .route("/crates/search", get(hello_world))
        .route("/crates/:crate/owners", get(hello_world))
        .route("/crates/:crate/owners", put(hello_world))
        .route("/crates/:crate/owners", delete(hello_world))
        .route("/crates/:crate/:version/yank", delete(hello_world))
        .route("/crates/:crate/:version/unyank", put(hello_world))
        .route(
            "/crates/:crate/:version/download",
            get(endpoints::cargo_api::download),
        ))
    .layer(
        ServiceBuilder::new()
            .layer_fn(middleware::auth::AuthMiddleware)
            .into_inner(),
    );

    let middleware_stack = ServiceBuilder::new()
        .layer(AsyncFilterLayer::new(|req| async {
            eprintln!("{:#?}", req);
            Ok::<_, std::convert::Infallible>(req)
        }))
        .into_inner();

    let pool = chartered_db::init();

    let app = Router::new()
        .route(
            "/",
            get(|| async move {
                format!(
                    "{:#?}",
                    chartered_db::get_crate_versions(pool, "cool-test-crate".to_string()).await
                )
            }),
        )
        .nest("/a/:key/api/v1", api_authenticated)
        .layer(middleware_stack);

    axum::Server::bind(&"0.0.0.0:8888".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
