use axum::Json;
use serde::{Deserialize, Serialize};

use crate::api::Response;
use crate::domain::trans_to_token::{refresh_token, sign_by_req, trans};

#[derive(Deserialize, Serialize)]
pub struct LoginResponse {
    token: String,
    refresh_token: String,
}

impl LoginResponse {
    fn new(token: String, rt: String) -> Self {
        Self {
            token,
            refresh_token: rt,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub user_name: String,
    pub pass_word: String,
}

fn get_resp<F>(
    result: anyhow::Result<(String, String)>,
    err_code: i32,
    err_msg: F,
) -> Response<LoginResponse>
where
    F: FnOnce(anyhow::Error) -> String,
{
    match result {
        Ok((token, rt)) => Response::ok(LoginResponse::new(token, rt)),
        Err(e) => Response::err(err_code, err_msg(e)),
    }
}

pub async fn login(Json(user): Json<LoginRequest>) -> Json<Response<LoginResponse>> {
    let resp = get_resp(trans(user).await, 301, |e| format!("err in login,{}", e));
    Json(resp)
}

#[derive(Deserialize, Serialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub async fn refresh(Json(request): Json<RefreshRequest>) -> Json<Response<LoginResponse>> {
    let resp = get_resp(refresh_token(request.refresh_token).await, 301, |e| {
        format!("err in refresh,{}", e)
    });
    Json(resp)
}

#[derive(Deserialize, Serialize)]
pub struct SignRequest {
    pub user_name: String,
    pub pass_word: String,
}

pub async fn sign(Json(request): Json<SignRequest>) -> Json<Response<LoginResponse>> {
    let resp = get_resp(sign_by_req(request).await, 301, |e| {
        format!("err in sign,{}", e)
    });
    Json(resp)
}
