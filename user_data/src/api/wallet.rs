use crate::domain::wallet::{cash_out_from_wallet, recharge_to_wallet, root_operate as operate};
use crate::repo::wallet::Balance as repo_balance;
use axum::{Extension, Form, Json};
use serde::{Deserialize, Serialize};
use util::axum::auth::UserToken;
use util::axum::Response;
use util::response;

#[derive(Deserialize, Serialize)]
pub struct Balance {
    balance_id: u64,
    user_id: u64,
    num: u64,
}

impl From<repo_balance> for Balance {
    fn from(value: repo_balance) -> Self {
        Self {
            balance_id: value.id.unwrap_or_default(),
            user_id: value.user_id,
            num: value.num,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Num {
    num: u64,
}

pub async fn recharge(
    Extension(user): Extension<UserToken>,
    Form(num): Form<Num>,
) -> Response<Balance> {
    let balance = repo_balance::from_user(user.user_id);
    response!(recharge_to_wallet(balance, num.num).await, Balance)
}

pub async fn cash_out(
    Extension(user): Extension<UserToken>,
    Form(num): Form<Num>,
) -> Response<Balance> {
    let balance = repo_balance::from_user(user.user_id);
    response!(cash_out_from_wallet(balance, num.num).await, Balance)
}

#[derive(Deserialize, Serialize)]
pub struct RootOperateRequest {
    pub balance_id: u64,
    pub num: i64,
    pub force: bool,
}

pub async fn root_operate(Json(req): Json<RootOperateRequest>) -> Response<Balance> {
    let balance = repo_balance::from_id(req.balance_id);
    response!(operate(balance, req.num, req.force).await, Balance)
}
