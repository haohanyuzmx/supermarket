use super::Error;
use crate::repo::DB;
use anyhow::anyhow;
use rbatis::crud;
use serde::{Deserialize, Serialize};
use table_rbs::CreateTable;
use util::rbatis::init::get_tx_set_defer;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, CreateTable)]
pub struct Balance {
    pub id: Option<u64>,
    #[index]
    pub user_id: u64,
    pub num: u64,
}

crud!(Balance {});

impl Balance {
    util::get!(Balance;(id->id_value:u64));
    util::get!(Balance;(user_id->id_value:u64));
    pub fn from_user(user_id: u64) -> Self {
        Self {
            user_id,
            ..Default::default()
        }
    }
    pub fn from_id(id: u64) -> Self {
        Self {
            id: Some(id),
            ..Default::default()
        }
    }
    pub async fn operate_num(&mut self, num: i64, force: bool) -> Error {
        let mut tx = get_tx_set_defer(DB.clone()).await?;
        match self.id {
            None => match Balance::get_by_user_id(&mut tx, self.user_id).await {
                // create
                None => {
                    self.id = Balance::insert(&mut tx, self)
                        .await?
                        .last_insert_id
                        .as_u64()
                }
                Some(balance) => {
                    self.id = balance.id;
                    self.num = balance.num
                }
            },
            Some(id) => {
                *self = Balance::get_by_id(&mut tx, id)
                    .await
                    .ok_or(anyhow!("wrong id"))?
            }
        }
        if force {
            self.num = num.try_into()?
        } else {
            self.num = match self.num.checked_add_signed(num) {
                None => return Err(anyhow!("wrong operate num:{num},now num:{:?}", self.num)),
                Some(num) => num,
            };
        }

        Balance::update_by_column(&mut tx, self, "id").await?;
        tx.commit().await?;
        Ok(())
    }
}
