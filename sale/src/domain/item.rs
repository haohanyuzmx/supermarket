use crate::api::item::{AddItemRequest, ItemOperateRequest, ItemResponse, Record as RespRecord};
use crate::repo::item::{Item, Record};
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedMutexGuard};
use tracing::log::{error, warn};
use util::axum::auth::UserToken;
use util::pb::client::{get_home_by_id, operate_wallet, WalletIndex};

lazy_static! {
    static ref RECORD_OPERATE_LOCK_MAP: Mutex<HashMap<u64, Arc<Mutex<()>>>> =
        Mutex::new(HashMap::new());
}

pub async fn create_item(request: AddItemRequest) -> Result<Item> {
    let mut item = Item::new(request.name, request.kind, request.price, request.remain);
    item.operate_num(None, false).await?;
    Ok(item)
}

pub async fn add_item_num(request: ItemOperateRequest) -> Result<Item> {
    let mut item: Item = request.item.into();
    item.operate_num(Some(request.num), false).await?;
    Ok(item)
}

pub async fn set_item_num(request: ItemOperateRequest) -> Result<Item> {
    let mut item: Item = request.item.into();
    item.operate_num(Some(request.num), true).await?;
    Ok(item)
}

pub async fn set_item_price(request: ItemOperateRequest) -> Result<Item> {
    let mut item: Item = request.item.into();
    item.set_price(request.num as u64).await?;
    Ok(item)
}

pub async fn add_to_cart(
    item_index: ItemOperateRequest,
    user_id: u64,
    home_id: u64,
) -> Result<RespRecord> {
    let mut item: Item = item_index.item.into();
    let record = item
        .operate_cart_num(user_id, home_id, item_index.num)
        .await?;
    Ok(RespRecord::new(record, Some(item.name), None).await)
}

pub async fn get_all_record_by_user(user_id: u64) -> Result<Vec<RespRecord>> {
    let mut vec_record = vec![];
    for repo_record in Record::get_by_user_id(user_id).await?.into_iter() {
        vec_record.push(RespRecord::new(repo_record, None, None).await)
    }
    Ok(vec_record)
}

pub async fn get_consult_record() -> Result<Vec<RespRecord>> {
    let mut vec_record = vec![];
    for repo_record in Record::get_status_consult().await?.into_iter() {
        vec_record.push(RespRecord::new(repo_record, None, None).await)
    }
    Ok(vec_record)
}

pub async fn get_all_item() -> Result<Vec<ItemResponse>> {
    Ok(Item::get_all()
        .await?
        .into_iter()
        .map(|item| item.into())
        .collect())
}

pub async fn change_record_home(mut record: Record, home_id: u64) -> Result<RespRecord> {
    record.get_self(None).await?;
    let home = get_home_by_id(home_id).await?;
    if home.user_id != record.user_id {
        return Err(anyhow!("not your home"));
    }
    record.change_home(home_id).await?;
    Ok(RespRecord::new(record, None, None).await)
}

async fn get_record_operate_lock(record_id: u64) -> OwnedMutexGuard<()> {
    let mut record_operate_lock_map = RECORD_OPERATE_LOCK_MAP.lock().await;
    let lock = record_operate_lock_map
        .entry(record_id)
        .or_insert(Arc::new(Mutex::new(())))
        .clone()
        .lock_owned()
        .await;
    drop(record_operate_lock_map);
    lock
}

pub enum Target {
    Pay,
    Cancel,
}

macro_rules! hashset {
    ($($status:expr),+) => {
        [$($status.to_string()),+].into_iter().collect::<HashSet<String>>()
    };
}

pub async fn wallet_record(
    mut record: Record,
    user: UserToken,
    target: Target,
) -> Result<RespRecord> {
    record.get_self(Some(hashset!("cart", "pay"))).await?;
    let operate = get_record_operate_lock(record.id.unwrap()).await;
    if record.user_id != user.user_id {
        return Err(anyhow!("only allow self operate"));
    }
    let item = Item::select_by_id(record.item_id)
        .await
        .ok_or(anyhow!("get item err"))?;

    let now_num = record.num;

    let (num, last_status) = match target {
        Target::Pay => {
            record.pay().await?;
            (-((item.price * record.num) as i64), "cart".to_string())
        }
        Target::Cancel => {
            record.cancel().await?;
            ((item.price * record.num) as i64, "pay".to_string())
        }
    };

    match operate_wallet(WalletIndex::UserID(user.user_id), num, false).await {
        Err(wallet_err) => {
            warn!("pay err! {},num is {}", wallet_err, num);
            match record.force_change_status(last_status).await {
                Err(record_err) => {
                    error!("{:?} change to cart wrong {}", record, &record_err);
                    return Err(anyhow!(
                        "wallet err {},record err,{}",
                        wallet_err,
                        record_err
                    ));
                }
                _ => {}
            };
            return Err(wallet_err);
        }
        _ => {}
    };
    Ok(RespRecord::new(record, Some(item.name), None).await)
}

pub async fn send_out_record(mut record: Record) -> Result<RespRecord> {
    record.get_self(Some(hashset!("pay"))).await?;
    let operate = get_record_operate_lock(record.id.unwrap()).await;
    record.send().await?;
    Ok(RespRecord::new(record, None, None).await)
}

pub async fn sign_record(mut record: Record, user: UserToken) -> Result<RespRecord> {
    record.get_self(Some(hashset!("sending"))).await?;
    let operate = get_record_operate_lock(record.id.unwrap()).await;
    if record.user_id != user.user_id {
        if user
            .get_auths()
            .await
            .unwrap_or_default()
            .iter()
            .find(|auth| auth.as_str() == "root" || auth.as_str() == "worker")
            .is_none()
        {
            return Err(anyhow!("un auth"));
        }
    }
    record.sign().await?;
    Ok(RespRecord::new(record, None, None).await)
}

pub async fn user_consult(mut record: Record, user_id: u64) -> Result<RespRecord> {
    record.get_self(Some(hashset!("sign", "sending"))).await?;
    let operate = get_record_operate_lock(record.id.unwrap()).await;
    if record.user_id != user_id {
        return Err(anyhow!("user_id error"));
    }
    record.consult().await?;
    Ok(RespRecord::new(record, None, None).await)
}

pub async fn root_consult(mut record: Record) -> Result<RespRecord> {
    record.get_self(Some(hashset!("discard"))).await?;
    let operate = get_record_operate_lock(record.id.unwrap()).await;
    record.discard().await?;
    Ok(RespRecord::new(record, None, None).await)
}
