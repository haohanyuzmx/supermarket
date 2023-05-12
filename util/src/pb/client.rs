use super::validate::validate_client::ValidateClient;
use crate::axum::auth::{Home, UserToken, Wallet};
use crate::pb::home::home_client::HomeClient;
use crate::pb::home::{GetAllHomeRequest, HomeId};
use crate::pb::validate::{
    AddUrlAuthRequest, AuthRequest, GetAllAuthRequest, TokenRequest, ValidateRequest,
};
use crate::pb::wallet::wallet_client::WalletClient;
use crate::pb::wallet::{operate_request, OperateRequest};
use tonic::transport::Channel;
use tonic::Request;

static mut V_CLIENT: String = String::new();
static mut H_CLIENT: String = String::new();
static mut W_CLIENT: String = String::new();

type Result<T> = anyhow::Result<T>;

async fn validate_client() -> ValidateClient<Channel> {
    ValidateClient::connect(validate).await.ok().unwrap()
}

async fn home_client() -> HomeClient<Channel> {
    HomeClient::connect(home).await.ok().unwrap()
}

async fn wallet_client() -> WalletClient<Channel> {
    WalletClient::connect(wallet).await.ok().unwrap()
}

pub async fn init(validate: String, home: String, wallet: String) {
    unsafe {
        V_CLIENT = validate;
        H_CLIENT = home;
        W_CLIENT = wallet;
    }
}

pub async fn validate_token(token: String) -> Result<UserToken> {
    let mut pb = validate_client();
    Ok(pb
        .validate_token(Request::new(TokenRequest { token }))
        .await?
        .into_inner()
        .into())
}

pub async fn validate_auth(user_id: u64, url: String) -> Result<bool> {
    let mut pb = validate_client();
    Ok(pb
        .validate_auth(Request::new(AuthRequest { user_id, url }))
        .await?
        .into_inner()
        .ok)
}

pub async fn validate(token: String, url: String) -> Result<(UserToken, bool)> {
    let mut pb = validate_client();
    let resp = pb
        .validate(Request::new(ValidateRequest { token, url }))
        .await?
        .into_inner();
    Ok((resp.user.unwrap().into(), resp.auth.unwrap().ok))
}

pub async fn add_url_auth(url: String, auth: String) -> Result<bool> {
    let mut pb = validate_client();
    Ok(pb
        .add_url_auth(Request::new(AddUrlAuthRequest { url, auth }))
        .await?
        .into_inner()
        .ok)
}

pub async fn get_all_auth(user_id: u64) -> Result<Vec<String>> {
    let mut pb = validate_client();
    Ok(pb
        .get_all_auth(Request::new(GetAllAuthRequest { user_id }))
        .await?
        .into_inner()
        .auth_infos)
}

pub async fn get_all_home(user_id: u64) -> Result<Vec<Home>> {
    let mut pb = home_client();
    Ok(pb
        .get_all_home(Request::new(GetAllHomeRequest { user_id }))
        .await?
        .into_inner()
        .home_addresses
        .into_iter()
        .map(|home| Home::from(home))
        .collect())
}

pub async fn get_home_by_id(home_id: u64) -> Result<Home> {
    let mut pb = home_client();
    Ok(pb
        .get_home_by_id(Request::new(HomeId { home_id }))
        .await?
        .into_inner()
        .into())
}

pub enum WalletIndex {
    BalanceID(u64),
    UserID(u64),
}

impl Into<OperateRequest> for WalletIndex {
    fn into(self) -> OperateRequest {
        let (typ, id) = match self {
            WalletIndex::BalanceID(id) => (operate_request::Type::BalanceId.into(), id),
            WalletIndex::UserID(id) => (operate_request::Type::UserId.into(), id),
        };
        OperateRequest {
            typ,
            id,
            ..Default::default()
        }
    }
}

pub async fn operate_wallet(wallet: WalletIndex, num: i64, force: bool) -> Result<Wallet> {
    let mut pb = wallet_client();
    let mut request: OperateRequest = wallet.into();
    request.num = num;
    request.force = force;
    Ok(pb.operate(Request::new(request)).await?.into_inner().into())
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::join;

    #[test]
    fn validate_token_info() {
        tokio_test::block_on(async {
            init(
                "http://127.0.0.1:8089".to_string(),
                "http://127.0.0.1:8090".to_string(),
                "".to_string(),
            )
            .await;
            let token = "eyJ1c2VyX25hbWUiOiIzMzMiLCJ1c2VyX2lkIjoxLCJkYXRhIjp7fSwidGltZV9vdXQiOjE2NzY3ODU1OTd9LjY1ZjVmYTBkMzk0N2YxMzZmNzgwMGM3YmE3YTA4YjBiZWY2YmM3MjYwNGVlMTBkNzE5MzI4MWUwODc2NjAyMzQ=".to_string();

            let user = join!(
                tokio::spawn(validate_token(token.clone())),
                tokio::spawn(validate_token(token.clone()))
            );
            dbg!(user);
        })
    }
}
