use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;

mod api;
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
    util::pb::client::init(
        "http://127.0.0.1:8089".to_string(),
        "".to_string(),
        "".to_string(),
    )
    .await;
    util::pb::init_url_auth(&[
        ("/add_home_address", "normal"),
        ("/change_home_address", "normal"),
        ("/delete_home_address", "normal"),
        ("/get_all_address", "normal"),
        ("/recharge_to_balance", "normal"),
        ("/cash_out_from_balance", "normal"),
        ("/root_operate_balance", "root"),
    ])
    .await;

    tokio::spawn(async move { pb::grpc_server("0.0.0.0:8090") }.await);

    // show_balance
    let app = Router::new()
        .route("/add_home_address", post(api::home::add_home_address))
        .route("/change_home_address", post(api::home::change_home_address))
        .route("/delete_home_address", post(api::home::delete_home_address))
        .route("/get_all_address", get(api::home::get_all_address))
        .route("/recharge_to_balance", post(api::wallet::recharge))
        .route("/cash_out_from_balance", post(api::wallet::cash_out))
        .route("/root_operate_balance", post(api::wallet::root_operate))
        .layer(util::axum::auth::AuthLayer::new(true, true, false, false))
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&"0.0.0.0:8082".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
