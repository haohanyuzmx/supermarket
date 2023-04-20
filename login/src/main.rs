use axum::{middleware, routing::post, Router};
use tower_http::trace::TraceLayer;

mod api;
mod cache;
mod domain;
mod pb;
mod repo;

#[tokio::main]
async fn main() {
    // if std::env::var_os("RUST_LOG").is_none() {
    //     std::env::set_var("RUST_LOG", "tower_http=debug,middleware=debug");
    // }

    util::log_init::init::init();

    repo::init().await;

    tokio::spawn(async move { pb::server::grpc_server("0.0.0.0:8089").await });

    let app = Router::new()
        .route("/add_auth", post(api::auth::add_auth))
        .layer(middleware::from_fn(api::validate::auth))
        .route("/login", post(api::login::login))
        .route("/sign", post(api::login::sign))
        .route("/refresh_token", post(api::login::refresh))
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
