use crate::domain::item::{
    add_item_num as domain_add_item_num, add_to_cart as domain_add_to_cart, change_record_home,
    create_item, get_all_item, get_all_record_by_user, get_consult_record, root_consult,
    send_out_record, set_item_num, set_item_price, sign_record, user_consult, wallet_record,
    Target,
};

use crate::repo::item::{Item, Record as repo_record};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use util::axum::auth::UserToken;
use util::axum::Response;
use util::pb::client::get_home_by_id;

macro_rules! response {
    ($item:expr) => {{
        match $item {
            Ok(item) => Response::ok(item),
            Err(e) => Response::err(300, e.to_string()),
        }
    }};
    ($item:expr,$typ:ty) => {{
        match $item {
            Ok(item) => Response::ok(<$typ>::from(item)),
            Err(e) => Response::err(300, e.to_string()),
        }
    }};
}

#[derive(Deserialize, Serialize)]
pub struct AddItemRequest {
    pub name: String,
    pub kind: String,
    pub price: u64,
    pub remain: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ItemResponse {
    pub id: u64,
    pub name: String,
    pub kind: String,
    pub remain: u64,
}

impl From<Item> for ItemResponse {
    fn from(value: Item) -> Self {
        Self {
            id: value.id.unwrap(),
            name: value.name,
            kind: value.kind,
            remain: value.remain,
        }
    }
}

pub async fn add_item(Json(request): Json<AddItemRequest>) -> Json<Response<ItemResponse>> {
    let response = response!(create_item(request).await, ItemResponse);
    Json(response)
}

#[derive(Deserialize, Serialize)]
pub enum ItemID {
    #[serde(rename = "item_id")]
    ItemID(u64),
    #[serde(rename = "item_name")]
    ItemName(String),
}

impl Into<Item> for ItemID {
    fn into(self) -> Item {
        match self {
            ItemID::ItemID(id) => Item {
                id: Some(id),
                ..Default::default()
            },
            ItemID::ItemName(name) => Item {
                name,
                ..Default::default()
            },
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ItemOperateRequest {
    pub item: ItemID,
    pub num: i64,
}

pub async fn add_item_num(Json(req): Json<ItemOperateRequest>) -> Json<Response<ItemResponse>> {
    let resp = response!(domain_add_item_num(req).await, ItemResponse);
    Json(resp)
}

pub async fn change_item_num(Json(req): Json<ItemOperateRequest>) -> Json<Response<ItemResponse>> {
    let resp = response!(set_item_num(req).await, ItemResponse);
    Json(resp)
}

pub async fn change_item_price(Json(req): Json<ItemOperateRequest>) -> Response<ItemResponse> {
    response!(set_item_price(req).await, ItemResponse)
}

#[derive(Deserialize, Serialize)]
pub struct Record {
    pub id: u64,
    pub item_name: String,
    pub num: u64,
    pub status: String,
    pub home: String,
}

impl Record {
    pub async fn new(value: repo_record, item_name: Option<String>, home: Option<String>) -> Self {
        let item_name = match item_name {
            None => {
                let item_name = Item::select_by_id(value.item_id).await.unwrap_or_default();
                item_name.name
            }
            Some(name) => name,
        };
        let home = match home {
            None => {
                let home = get_home_by_id(value.home_id).await.unwrap_or_default();
                home.home_address
            }
            Some(home) => home,
        };
        Self {
            id: value.id.unwrap(),
            item_name,
            num: value.num,
            status: value.status,
            home,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct AddToCartRequest {
    item: ItemOperateRequest,
    home_id: Option<u64>,
}

pub async fn add_to_cart(
    Extension(user): Extension<UserToken>,
    Json(req): Json<AddToCartRequest>,
) -> Json<Response<Record>> {
    let resp = response!(
        domain_add_to_cart(
            req.item,
            user.user_id,
            req.home_id.unwrap_or(
                user.get_homes()
                    .await
                    .unwrap_or_default()
                    .pop()
                    .unwrap_or_default()
                    .home_id
            )
        )
        .await
    );
    Json(resp)
}

pub async fn get_record(Extension(user): Extension<UserToken>) -> Json<Response<Vec<Record>>> {
    let resp = response!(get_all_record_by_user(user.user_id).await);
    Json(resp)
}

pub async fn show_items() -> Json<Response<Vec<ItemResponse>>> {
    let resp = response!(get_all_item().await);
    Json(resp)
}

#[derive(Deserialize, Serialize)]
pub enum RecordIndex {
    #[serde(rename = "record_id")]
    RecordID(u64),
    // #[serde(rename = "record_info")]
    // RecordInfo { item_id: u64, home_id: u64 },
}

impl Into<repo_record> for RecordIndex {
    fn into(self) -> repo_record {
        match self {
            RecordIndex::RecordID(id) => repo_record {
                id: Some(id),
                ..Default::default()
            },
            // RecordIndex::RecordInfo { item_id, home_id } => repo_record {
            //     item_id,
            //     home_id,
            //     ..Default::default()
            // },
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ChangeHomeRequest {
    pub record: RecordIndex,
    pub home_id: u64,
}

#[axum::debug_handler]
pub async fn change_home(
    Extension(user): Extension<UserToken>,
    Json(req): Json<ChangeHomeRequest>,
) -> Response<Record> {
    let mut record: repo_record = req.record.into();
    record.user_id = user.user_id;
    response!(change_record_home(record, req.home_id).await)
}

pub async fn pay(
    Extension(user): Extension<UserToken>,
    Json(req): Json<RecordIndex>,
) -> Response<Record> {
    let mut record: repo_record = req.into();
    record.user_id = user.user_id;
    response!(wallet_record(record, user, Target::Pay).await)
}

pub async fn send(
    Extension(user): Extension<UserToken>,
    Json(req): Json<RecordIndex>,
) -> Response<Record> {
    let mut record: repo_record = req.into();
    record.user_id = user.user_id;
    response!(send_out_record(record).await)
}

pub async fn sign_to_record(
    Extension(user): Extension<UserToken>,
    Json(req): Json<RecordIndex>,
) -> Response<Record> {
    let mut record: repo_record = req.into();
    record.user_id = user.user_id;
    response!(sign_record(record, user).await)
}

pub async fn cancel_record(
    Extension(user): Extension<UserToken>,
    Json(req): Json<RecordIndex>,
) -> Response<Record> {
    let mut record: repo_record = req.into();
    record.user_id = user.user_id;
    response!(wallet_record(record, user, Target::Cancel).await)
}

pub async fn consult(
    Extension(user): Extension<UserToken>,
    Json(req): Json<RecordIndex>,
) -> Response<Record> {
    let mut record: repo_record = req.into();
    record.user_id = user.user_id;
    let response = if user
        .get_auths()
        .await
        .unwrap_or_default()
        .iter()
        .find(|auth| auth == "worker")
        .is_some()
    {
        user_consult(record, user.user_id).await
    } else {
        root_consult(record).await
    };
    response!(response)
}

pub async fn get_consult() -> Response<Vec<Record>> {
    response!(get_consult_record().await)
}
