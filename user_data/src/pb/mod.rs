use home::HomeImpl;
use tonic::transport::Server;
use tower_http::trace::TraceLayer;
use util::pb::home::home_server::HomeServer;
use util::pb::wallet::wallet_server::WalletServer;
use wallet::WalletImpl;

pub mod home;
pub mod wallet;

pub async fn grpc_server(addr: &str) {
    let home = HomeImpl::default();
    let wallet = WalletImpl::default();
    Server::builder()
        .layer(TraceLayer::new_for_grpc())
        .add_service(HomeServer::new(home))
        .add_service(WalletServer::new(wallet))
        .serve(addr.parse().unwrap())
        .await
        .unwrap();
}
