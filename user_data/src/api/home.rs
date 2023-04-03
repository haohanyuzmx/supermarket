use crate::domain::home::{
    add_home_addr, change_home_addr, delete_home_addr, get_all as root_get_all, get_all_by_user,
    Home,
};
use crate::repo::home::Home as repo_home;

use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use util::axum::auth::UserToken;
use util::axum::Response;

#[derive(Deserialize, Serialize)]
pub enum AddressIndex {
    #[serde(rename = "id")]
    ID(u64),
    #[serde(rename = "address")]
    Address(String),
}

impl Into<repo_home> for AddressIndex {
    fn into(self) -> repo_home {
        match self {
            AddressIndex::ID(id) => repo_home {
                id: Some(id),
                ..Default::default()
            },
            AddressIndex::Address(addr) => repo_home {
                home_address: addr,
                ..Default::default()
            },
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Address {
    pub id: u64,
    pub user_id: u64,
    pub user_name: String,
    pub home_address: String,
}

impl Address {
    fn new(home: Home, user_name: String) -> Self {
        Self {
            id: home.id,
            user_id: home.user_id,
            home_address: home.home_address,
            user_name,
        }
    }
}

#[axum::debug_handler]
pub async fn add_home_address(
    Extension(user): Extension<UserToken>,
    Json(request): Json<AddressIndex>,
) -> Response<Address> {
    let mut home: repo_home = request.into();
    home.user_id = user.user_id;
    match add_home_addr(home).await {
        Ok(home) => Response::ok(Address::new(home, user.user_name)),
        Err(e) => Response::err(300, e.to_string()),
    }
}

#[derive(Deserialize, Serialize)]
pub struct ChangeAddrRequest {
    pub old: AddressIndex,
    pub new: AddressIndex,
}

pub async fn change_home_address(
    Extension(user): Extension<UserToken>,
    Json(request): Json<ChangeAddrRequest>,
) -> Response<Address> {
    let mut old: repo_home = request.old.into();
    old.user_id = user.user_id;
    let mut new: repo_home = request.new.into();
    new.user_id = user.user_id;
    match change_home_addr(old, new).await {
        Ok(home) => Response::ok(Address::new(home, user.user_name)),
        Err(e) => Response::err(300, e.to_string()),
    }
}

pub async fn delete_home_address(
    Extension(user): Extension<UserToken>,
    Json(request): Json<AddressIndex>,
) -> Response<Address> {
    let mut home: repo_home = request.into();
    home.user_id = user.user_id;
    match delete_home_addr(home).await {
        Ok(home) => Response::ok(Address::new(home, user.user_name)),
        Err(e) => Response::err(300, e.to_string()),
    }
}

pub async fn get_all_address(Extension(user): Extension<UserToken>) -> Response<Vec<Address>> {
    let addresses = match user.get_auths().await {
        None => {
            return Response::err(300, "no auth".to_string());
        }
        Some(addresses) => addresses,
    };

    let homes = match addresses.iter().find(|role| *role == "root") {
        None => get_all_by_user(user.user_id).await,
        Some(_) => root_get_all().await,
    };
    match homes {
        Ok(addresses) => Response::ok(
            addresses
                .into_iter()
                .map(|home| Address::new(home, user.user_name.to_string()))
                .collect(),
        ),
        Err(e) => Response::err(300, e.to_string()),
    }
}
