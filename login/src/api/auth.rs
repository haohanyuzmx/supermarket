use axum::Json;
use serde::{Deserialize, Serialize};

use crate::api::Response;
use crate::domain::validate_auth::set_role_auth;

#[derive(Deserialize, Serialize)]
pub struct AddAuthRequest {
    pub user_name: String,
    pub role_name: String,
}
pub async fn add_auth(Json(request): Json<AddAuthRequest>) -> Json<Response<String>> {
    let result = match set_role_auth(&request.user_name, &request.role_name).await {
        Ok(_) => "ok".to_string(),
        Err(e) => e.to_string(),
    };
    Json(Response::ok(result))
}
