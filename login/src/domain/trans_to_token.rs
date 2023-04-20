use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Add;
use std::string::FromUtf8Error;

use base64::engine::general_purpose::STANDARD as base64;
use base64::{DecodeError, Engine};
use chrono::prelude::*;
use chrono::Duration;
use hex::FromHexError;
use hmac::digest::MacError;
use hmac::{Hmac, Mac};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;

use crate::api::login::{LoginRequest, SignRequest};
use crate::cache::redis::{get_user_by_rt, set_rt_with_ttl};
use crate::repo::user::{create_user, get_user_by_info, User};

lazy_static! {
    static ref HMAC_SHA256: Hmac<Sha256> =
        Hmac::<Sha256>::new_from_slice(b"magic secret key").unwrap();
}

type TokenRT = (String, String);

pub async fn trans(request: LoginRequest) -> anyhow::Result<TokenRT> {
    let expire_time = Duration::days(1);
    let user = get_user_by_info(&request.user_name, Some(&request.pass_word)).await?;
    let user_info = serde_json::to_string(&UserToken::new(&user, expire_time)).unwrap();
    Ok(get_set_token(user_info, expire_time * 7).await)
}

pub async fn refresh_token(rt: String) -> anyhow::Result<TokenRT> {
    let expire_time = Duration::days(1);
    let user_str = get_user_by_rt(&rt).await?;
    let user: UserToken = serde_json::from_str(&user_str)?;
    let user_str = serde_json::to_string(&user.refresh(expire_time)).unwrap();
    Ok(get_set_token(user_str, expire_time * 7).await)
}

pub async fn sign_by_req(request: SignRequest) -> anyhow::Result<TokenRT> {
    let expire_time = Duration::days(1);
    let user = create_user(request.user_name, request.pass_word).await?;
    let user_info = serde_json::to_string(&UserToken::new(&user, expire_time)).unwrap();
    Ok(get_set_token(user_info, expire_time * 7).await)
}

async fn get_set_token(user_info: String, expire: Duration) -> TokenRT {
    let (token, rt) = user_to_token(&user_info);
    if let Err(e) = set_rt_with_ttl(&rt, &user_info, expire).await {
        fast_log::print(format!("set rt err {}", e)).unwrap_or(())
    };
    (token, rt)
}

fn user_to_token(user_info: &str) -> TokenRT {
    let mut hmac_sha256 = HMAC_SHA256.clone();
    hmac_sha256.update(user_info.as_ref());
    let hmac_result = hex::encode(hmac_sha256.finalize().into_bytes().to_vec());
    let token = base64.encode(format!("{}.{}", user_info, hmac_result));
    let rt = hex::encode(md5::compute(&token).to_vec());
    (token, rt)
}

#[derive(Debug, Error)]
pub enum ValidateErr {
    DecodeBase64Err(#[from] DecodeError),
    TransUTF8Err(#[from] FromUtf8Error),
    LenErr,
    ValidateHmacErr(#[from] MacError),
    ValidateHexErr(#[from] FromHexError),
    JSONErr(#[from] serde_json::Error),
    Expire,
}

impl Display for ValidateErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "validate err {:?}", self)
    }
}

pub fn validate(token: &str) -> Result<UserToken, ValidateErr> {
    let decode_token = String::from_utf8(base64.decode(token)?)?;
    let str_hmac: Vec<&str> = decode_token.split(".").collect();
    if str_hmac.len() != 2 {
        return Err(ValidateErr::LenErr);
    }
    let str_user = str_hmac[0];
    let hmac = str_hmac[1];

    let mut hmac_sha256 = HMAC_SHA256.clone();
    hmac_sha256.update(str_user.as_ref());
    hmac_sha256.verify_slice(hex::decode(hmac.as_bytes())?.as_ref())?;
    let user: UserToken = serde_json::from_str(str_user)?;
    if !user.valid() {
        return Err(ValidateErr::Expire);
    }
    Ok(user)
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UserToken {
    pub user_name: String,
    pub user_id: u64,
    pub data: HashMap<String, String>,
    pub time_out: i64,
}

impl UserToken {
    fn new(user_info: &User, expire_time: Duration) -> Self {
        Self {
            user_name: user_info.user_name.clone(),
            user_id: user_info.id.unwrap_or(0),
            data: Default::default(),
            time_out: Local::now().add(expire_time).timestamp(),
        }
    }
    fn valid(&self) -> bool {
        return self.time_out > Local::now().timestamp();
    }
    fn refresh(mut self, expire_time: Duration) -> Self {
        self.time_out = Local::now().add(expire_time).timestamp();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token() {
        let user_token = UserToken::new(
            &User {
                id: Some(3),
                user_name: "worker".to_string(),
                pass_word: "worker".to_string(),
            },
            Duration::weeks(7),
        );
        let user_info = serde_json::to_string(&user_token).unwrap();
        let (token, _) = user_to_token(&user_info);
        dbg!(&token);
        let check_user = validate(&token).unwrap();
        assert_eq!(user_token, check_user);
    }
    #[test]
    fn test_validate() {}
}
