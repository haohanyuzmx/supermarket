use crate::domain::wallet::root_operate;
use crate::repo::wallet::Balance;
use tonic::{Request, Response, Status};
use util::pb::wallet::operate_request::Type;
use util::pb::wallet::{wallet_server::Wallet, *};

#[derive(Debug, Default)]
pub struct WalletImpl {}

#[tonic::async_trait]
impl Wallet for WalletImpl {
    async fn operate(
        &self,
        request: Request<OperateRequest>,
    ) -> Result<Response<OperateResponse>, Status> {
        let req = request.into_inner();
        let balance = match Type::from_i32(req.typ) {
            None => return Err(Status::unknown("invalid type")),
            Some(Type::UserId) => {
                let b = Balance::from_user(req.id);
                root_operate(b, req.num, false)
                    .await
                    .map_err(|e| Status::unknown(format!("operate err:{e}")))?
            }
            Some(Type::BalanceId) => {
                let b = Balance::from_id(req.id);
                root_operate(b, req.num, true)
                    .await
                    .map_err(|e| Status::unknown(format!("operate err:{e}")))?
            }
        };
        Ok(Response::new(OperateResponse {
            balance_id: balance.id.unwrap_or_default(),
            user_id: balance.user_id,
            num: balance.num,
        }))
    }
}
