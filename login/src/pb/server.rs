use tonic::{transport::Server, Request, Response, Status};
use tower_http::trace::TraceLayer;

use crate::domain::validate_auth::set_job_auth;
use crate::domain::{
    trans_to_token::validate as token_validate, validate_auth::validate as auth_validate,
};

use crate::repo::auth::Role;
use util::pb::validate::{
    validate_server::{Validate, ValidateServer},
    *,
};

#[derive(Debug, Default)]
pub struct ValidateImpl {}

#[tonic::async_trait]
impl Validate for ValidateImpl {
    async fn validate_token(
        &self,
        request: Request<TokenRequest>,
    ) -> Result<Response<UserInfo>, Status> {
        let user = token_validate(&request.into_inner().token)
            .map_err(|e| Status::unknown(e.to_string()))?;
        let info = UserInfo {
            user_name: user.user_name,
            user_id: user.user_id,
            data: user.data,
            time_out: user.time_out,
        };
        Ok(Response::new(info))
    }

    async fn validate_auth(
        &self,
        request: Request<AuthRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let auth_request = request.into_inner();
        Ok(Response::new(AuthResponse {
            ok: auth_validate(auth_request.user_id, &auth_request.url).await,
        }))
    }

    async fn validate(
        &self,
        request: Request<ValidateRequest>,
    ) -> Result<Response<ValidateResponse>, Status> {
        let validate_request = request.into_inner();
        let user = self
            .validate_token(Request::new(TokenRequest {
                token: validate_request.token,
            }))
            .await?
            .into_inner();
        let auth = self
            .validate_auth(Request::new(AuthRequest {
                user_id: user.user_id,
                url: validate_request.url,
            }))
            .await?
            .into_inner();
        Ok(Response::new(ValidateResponse {
            user: Some(user),
            auth: Some(auth),
        }))
    }

    async fn add_url_auth(
        &self,
        request: Request<AddUrlAuthRequest>,
    ) -> Result<Response<AddUrlAuthResponse>, Status> {
        let auth_request = request.into_inner();
        set_job_auth(auth_request.url, &auth_request.auth)
            .await
            .map_err(|e| Status::unknown(e.to_string()))?;
        Ok(Response::new(AddUrlAuthResponse { ok: true }))
    }

    async fn get_all_auth(
        &self,
        request: Request<GetAllAuthRequest>,
    ) -> Result<Response<GetAllAuthResponse>, Status> {
        let roles = Role::get_by_user(request.into_inner().user_id).await;
        Ok(Response::new(GetAllAuthResponse {
            auth_infos: Vec::from_iter(roles),
        }))
    }
}

pub async fn grpc_server(addr: &str) {
    let validate = ValidateImpl::default();
    Server::builder()
        .layer(TraceLayer::new_for_grpc())
        .add_service(ValidateServer::new(validate))
        .serve(addr.parse().unwrap())
        .await
        .unwrap();
}
