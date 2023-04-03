use super::Error;
use crate::repo::DB;
use anyhow::anyhow;
use anyhow::Result;
use rbatis::{crud, impl_delete, impl_select};
use serde::{Deserialize, Serialize};
use table_rbs::CreateTable;
use util::rbatis::init::get_tx_set_defer;

// TODO:所有相关接口
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, CreateTable)]
pub struct Home {
    pub id: Option<u64>,
    #[index]
    pub user_id: u64,
    pub home_address: String,
}

crud!(Home {});
impl_select!(Home{select_by_info(user_id:u64,home_address:&str)->Option
    =>"`where  user_id=#{user_id} and home_address = #{home_address}`"});
impl_delete!(Home{delete_by_info(user_id:u64,home_address:&str)
    =>"`where  user_id=#{user_id} and home_address = #{home_address}`"});

impl Home {
    #[allow(dead_code)]
    pub fn new(user_id: u64, home_address: String) -> Self {
        Self {
            id: None,
            user_id,
            home_address,
        }
    }
    util::get!(Home;(home_address->address:&str));
    util::get!(Home;(id->id_value:u64));
    util::get!(pub(DB) Home;(id->id_value:u64));

    pub async fn create_check(&mut self) -> Error {
        let mut tx = get_tx_set_defer(DB.clone()).await?;
        if let Ok(Some(_)) = Home::select_by_info(&mut tx, self.user_id, &self.home_address).await {
            return Err(anyhow!("the item exit"));
        }
        self.id = Home::insert(&mut DB.clone(), self)
            .await?
            .last_insert_id
            .as_u64();
        tx.commit().await?;
        Ok(())
    }

    pub async fn change_addr(&mut self, addr: String) -> Error {
        let mut tx = get_tx_set_defer(DB.clone()).await?;
        *self = match self.id {
            Some(id) => Home::get_by_id(&mut tx, id)
                .await
                .ok_or(anyhow!("can't get id"))?,
            None => Home::select_by_info(&mut tx, self.user_id, &self.home_address)
                .await?
                .ok_or(anyhow!("can't select by info"))?,
        };
        self.home_address = addr;
        Home::update_by_column(&mut tx, self, "id").await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn delete(self) -> Error {
        match self.id {
            Some(id) => Home::delete_by_column(&mut DB.clone(), "id", id).await?,
            None => Home::delete_by_info(&mut DB.clone(), self.user_id, &self.home_address).await?,
        };
        Ok(())
    }

    pub async fn get_all_by_user(user_id: u64) -> Result<Vec<Self>> {
        let mut rb = DB.clone();
        Ok(Home::select_by_column(&mut DB.clone(), "user_id", user_id).await?)
    }

    pub async fn get_all() -> Result<Vec<Self>> {
        Ok(Home::select_all(&mut DB.clone()).await?)
    }
}
