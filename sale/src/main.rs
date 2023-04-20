use axum::routing::{get, post};
use axum::{middleware, Router};
use tower_http::trace::TraceLayer;

mod api;
mod domain;
mod repo;
// TODO: discard&consult
#[tokio::main]
async fn main() {
    // if std::env::var_os("RUST_LOG").is_none() {
    //     std::env::set_var("RUST_LOG", "tower_http=debug,middleware=debug");
    // }
    util::log_init::init::init();

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
        .route("/add_to_cart", post(api::item::add_to_cart))
        .route("/get_records", get(api::item::get_record))
        .route("/change_home", post(api::item::change_home))
        .route("/pay", post(api::item::pay))
        .route("/send", post(api::item::send))
        .route("/sign_to_record", post(api::item::sign_to_record))
        .route("/cancel", post(api::item::cancel_record))
        .route("/consult", post(api::item::consult))
        .route("/show_consult", get(api::item::get_consult))
        .layer(middleware::from_fn(util::axum::auth::auth))
        .route("/show_items", get(api::item::show_items))
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
    check_url_auth("/show_items", "normal").await;
    check_url_auth("/add_to_cart", "normal").await;
    check_url_auth("/get_records", "normal").await;
    check_url_auth("/change_home", "normal").await;
    check_url_auth("/pay", "normal").await;
    check_url_auth("/send", "worker").await;
    check_url_auth("/sign_to_record", "normal").await;
    check_url_auth("/cancel", "normal").await;
    check_url_auth("/consult", "normal").await;
    check_url_auth("/show_consult", "worker").await;
}

async fn check_url_auth(url: &str, auth: &str) {
    if !util::pb::client::add_url_auth(url.to_string(), auth.to_string())
        .await
        .expect(&format!("start init url failed,{},{}", url, auth))
    {
        panic!("init url failed,{},{}", url, auth)
    }
}
