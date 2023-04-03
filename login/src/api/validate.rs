use axum::{
    http,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::domain::{
    trans_to_token::validate as token_validate, validate_auth::validate as auth_validate,
};

pub async fn auth<B>(mut req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Ok(current_user) = token_validate(auth_header) {
        if auth_validate(current_user.user_id, req.uri().path()).await {
            req.extensions_mut().insert(current_user);
            return Ok(next.run(req).await);
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}
