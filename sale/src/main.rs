use axum::routing::{get, post};
use axum::{middleware, Router};
use tower_http::trace::TraceLayer;

mod api;
mod domain;
mod repo;

#[tokio::main]
async fn main() {
    // if std::env::var_os("RUST_LOG").is_none() {
    //     std::env::set_var("RUST_LOG", "tower_http=debug,middleware=debug");
    // }
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    repo::init().await;
    util::pb::client::init(
        "http://127.0.0.1:8089".to_string(),
        "http://127.0.0.1:8090".to_string(),
        "http://127.0.0.1:8090".to_string(),
    )
    .await;
    init_url_auth().await;

    // TODO: 购物车的状态方程，一些零碎的函数
    // TODO: 删除item，删除record，record-num=0自动删除
    let app = Router::new()
        .route("/add_item", post(api::item::add_item))
        .route("/add_item_num", post(api::item::add_item_num))
        .route("/change_item_num", post(api::item::change_item_num))
        .route("/change_item_price", post(api::item::change_item_price))
        .route("/show_items", get(api::item::show_items))
        .route("/add_to_card", post(api::item::add_to_card))
        .route("/get_records", get(api::item::get_record))
        .route("/change_home", post(api::item::change_home))
        .route("/pay", post(api::item::pay))
        .route("/send", post(api::item::send))
        .route("/sign_to_record", post(api::item::sign_to_record))
        .layer(middleware::from_fn(util::axum::auth::auth))
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&"0.0.0.0:8081".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn init_url_auth() {
    check_url_auth("/add_item", "worker").await;
    check_url_auth("/add_item_num", "worker").await;
    check_url_auth("/change_item_num", "worker").await;
    check_url_auth("/change_item_price", "worker").await;
    check_url_auth("/show_items", "worker").await;
    check_url_auth("/add_to_card", "normal").await;
    check_url_auth("/get_records", "normal").await;
    check_url_auth("/change_home", "normal").await;
    check_url_auth("/pay", "normal").await;
    check_url_auth("/send", "worker").await;
    check_url_auth("/sign_to_record", "normal").await;
}

async fn check_url_auth(url: &str, auth: &str) {
    if !util::pb::client::add_url_auth(url.to_string(), auth.to_string())
        .await
        .expect(&format!("start init url failed,{},{}", url, auth))
    {
        panic!("init url failed,{},{}", url, auth)
    }
}
