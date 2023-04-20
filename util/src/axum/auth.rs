use crate::pb::client::{get_all_auth, get_all_home, operate_wallet, validate, WalletIndex};
use crate::pb::home::HomeAddress;
use crate::pb::validate::UserInfo;
use crate::pb::wallet::OperateResponse;
use anyhow::Result;
use axum::response::IntoResponse;
use axum::{
    body::Body,
    http,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::info;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct UserToken {
    pub user_name: String,
    pub user_id: u64,
    pub data: HashMap<String, String>,
    pub time_out: i64,
}

macro_rules! token_get_fn {
    ($name:ident->$result_ty:ty=>$fn:ident) => {
        paste::paste! {
            #[allow(dead_code)]
            pub async fn [<get_$name>](&self)->$result_ty{
                match self.data.get(stringify!($name)){
                    None=>Some($fn(self.user_id).await.ok()?),
                    Some(value)=>Some(serde_json::from_str(value).ok()?),
                }
            }
        }
    };
}

impl UserToken {
    token_get_fn!(homes -> Option<Vec<Home>> => get_all_home);
    token_get_fn!(auths -> Option<Vec<String>> => get_all_auth);
    token_get_fn!(wallet -> Option<Wallet> => get_wallet);
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct Home {
    pub home_id: u64,
    pub user_id: u64,
    pub home_address: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct Wallet {
    pub balance_id: u64,
    pub user_id: u64,
    pub num: u64,
}

async fn get_wallet(user_id: u64) -> Result<Wallet> {
    operate_wallet(WalletIndex::UserID(user_id), 0, false).await
}

impl From<OperateResponse> for Wallet {
    fn from(value: OperateResponse) -> Self {
        Self {
            balance_id: value.balance_id,
            user_id: value.user_id,
            num: value.num,
        }
    }
}

impl From<HomeAddress> for Home {
    fn from(value: HomeAddress) -> Self {
        Self {
            home_id: value.address_id,
            user_id: value.user_id,
            home_address: value.address,
        }
    }
}

impl From<UserInfo> for UserToken {
    fn from(value: UserInfo) -> Self {
        Self {
            user_id: value.user_id,
            user_name: value.user_name,
            data: value.data,
            time_out: value.time_out,
        }
    }
}

#[derive(Clone)]
pub struct AuthLayer {
    pub validate: bool,
    pub auth: bool,
    pub home: bool,
    pub wallet: bool,
    // TODO:选择到底有哪些额外的组件
}

impl AuthLayer {
    #[allow(dead_code)]
    pub fn new(validate: bool, auth: bool, home: bool, wallet: bool) -> Self {
        Self {
            validate,
            auth,
            home,
            wallet,
        }
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthServer<S>;

    fn layer(&self, inner: S) -> Self::Service {
        let &AuthLayer {
            validate,
            auth,
            home,
            wallet,
        } = self;
        AuthServer {
            validate,
            auth,
            home,
            wallet,
            inner,
        }
    }
}

#[derive(Clone)]
pub struct AuthServer<S> {
    validate: bool,
    auth: bool,
    home: bool,
    wallet: bool,
    inner: S,
}

impl<S> Service<Request<Body>> for AuthServer<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static + Clone,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        let mut clone = self.clone();
        Box::pin(async move {
            let mut token = match auth_fn(&request, clone.validate).await {
                Ok(token) => token,
                Err(e) => return Ok(e.into_response()),
            };
            if clone.auth {
                let auths = token.get_auths().await.unwrap_or_default();
                token
                    .data
                    .insert("auth".to_string(), serde_json::to_string(&auths).unwrap());
            }
            if clone.home {
                let home = token.get_homes().await.unwrap_or_default();
                token
                    .data
                    .insert("home".to_string(), serde_json::to_string(&home).unwrap());
            }
            if clone.wallet {
                let wallet = token.get_wallet().await.unwrap_or_default();
                token.data.insert(
                    "wallet".to_string(),
                    serde_json::to_string(&wallet).unwrap(),
                );
            }
            request.extensions_mut().insert(token);
            let response: Response = clone.inner.call(request).await?;
            Ok(response)
        })
    }
}

pub async fn auth_fn(req: &Request<Body>, auth: bool) -> Result<UserToken, StatusCode> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());
    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    match validate(auth_header.to_string(), req.uri().path().to_string()).await {
        Ok((current_user, ok)) => {
            //可选
            if !auth || ok {
                return Ok(current_user);
            }
        }
        Err(e) => info!("{}", e),
    }
    Err(StatusCode::UNAUTHORIZED)
}

// TODO:返回函数？或者实现layer特征，自定义需要获取的额外参数，是否需要鉴权等
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

    if let Ok((current_user, ok)) =
        validate(auth_header.to_string(), req.uri().path().to_string()).await
    {
        if ok {
            req.extensions_mut().insert(current_user);
            return Ok(next.run(req).await);
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::get;
    use axum::body::BoxBody;
    use axum::Extension;
    use std::result::Result;
    use tower::{BoxError, ServiceBuilder, ServiceExt};
    use tower_test::mock;

    #[tokio::test]
    async fn layer_test() {
        let str_token = "eyJ1c2VyX25hbWUiOiIzMzMiLCJ1c2VyX2lkIjoxLCJkYXRhIjp7fSwidGltZV9vdXQiOjE2ODE1NDcyNDZ9LmVmOTllNzMwYTczZjU3MTZlMjk5YjllNDgwNmEwMmFmM2JjYzU3MTUyMDNmY2M3OTBlZWI4Nzc3ODZlNDM4Mzc=".to_string();
        crate::pb::client::init(
            "http://127.0.0.1:8089".to_string(),
            "http://127.0.0.1:8090".to_string(),
            "http://127.0.0.1:8090".to_string(),
        )
        .await;

        let mut service = ServiceBuilder::new()
            .layer(AuthLayer::new(false, true, true, true))
            .service_fn(echo);

        let mut request = Request::get("/")
            .header(http::header::ACCEPT, "application/json")
            .header(http::header::AUTHORIZATION, str_token)
            .body(Body::empty())
            .unwrap();

        let res = <AuthServer<_> as ServiceExt<_>>::ready(&mut service)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        dbg!(res);
        //assert_eq!(res.status(), StatusCode::OK);
    }
    async fn echo(req: Request<Body>) -> Result<Response, BoxError> {
        let token: &UserToken = req.extensions().get().unwrap();
        dbg!(token);
        dbg!(req);
        Ok(Response::new(BoxBody::default()))
    }
    // async fn echo(Extension(user): Extension<UserToken>) -> crate::axum::Response<UserToken> {
    //     dbg!(&user);
    //     crate::axum::Response::ok(user)
    // }
}
