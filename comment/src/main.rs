use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use tower_http::trace::TraceLayer;

mod api;
mod domain;
mod repo;

#[tokio::main]
async fn main() {
    // if std::env::var_os("RUST_LOG").is_none() {
    //     std::env::set_var("RUST_LOG", "tower_http=debug,middleware=debug");
    // }
    util::log_init::init::init();

    repo::init().await;
    util::pb::client::init(
        "http://127.0.0.1:8089".to_string(),
        "".to_string(),
        "".to_string(),
    )
    .await;

    util::pb::init_url_auth(&[
        ("/comment_to", "normal"),
        ("/change_comment", "normal"),
        ("/delete_comment", "normal"),
    ])
    .await;

    let app = Router::new()
        .route("/comment_to", post(api::comment::comment_to))
        .route("/change_comment", post(api::comment::change_comment))
        .route("/delete_comment", delete(api::comment::delete_comment))
        .layer(middleware::from_fn(util::axum::auth::auth))
        .route("/comment_of", get(api::comment::comments_of))
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&"0.0.0.0:8083".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
