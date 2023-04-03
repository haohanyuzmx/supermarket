use crate::repo::DB;
use anyhow::{anyhow, Result};
use rbatis;
use rbatis::executor::Executor;
use serde::{Deserialize, Serialize};
use table_rbs::CreateTable;
use util::rbatis::init::get_tx_set_defer;

type Error = anyhow::Result<()>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, CreateTable)]
pub struct Item {
    pub id: Option<u64>,
    #[index]
    pub name: String,
    #[index]
    pub kind: String,
    pub price: u64,
    pub remain: u64,
}

rbatis::crud!(Item {});
rbatis::impl_select!(Item{select_left()->Vec
    =>"`where remain > 0`"});

impl Item {
    pub fn new(name: String, kind: String, price: u64, remain: u64) -> Self {
        Self {
            id: None,
            name,
            kind,
            price,
            remain,
        }
    }

    util::get!(Item;(name->name_value:&str));
    util::get!(Item;(id->id_value:u64));
    util::get!(pub(DB) Item;(id->id_value:u64));

    pub async fn get_all() -> Result<Vec<Self>> {
        Ok(Item::select_all(&mut DB.clone()).await?)
    }

    async fn insert_check(&mut self) -> Error {
        let mut tx = get_tx_set_defer(DB.clone()).await?;
        if let Some(_) = Item::get_by_name(&mut tx, &self.name).await {
            return Err(anyhow!("the item exit"));
        }
        self.id = Item::insert(&mut tx, &self).await?.last_insert_id.as_u64();
        tx.commit().await?;
        Ok(())
    }

    pub async fn operate_num(&mut self, num: Option<i64>, force: bool) -> Error {
        if num.is_none() {
            return self.insert_check().await;
        }
        let mut tx = get_tx_set_defer(DB.clone()).await?;
        self.change_num(&mut tx, num.unwrap(), force).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn set_price(&mut self, price: u64) -> Error {
        let mut rb = DB.clone();
        let option_item = match self.id {
            None => Item::get_by_name(&mut rb, &self.name).await,
            Some(id) => Item::get_by_id(&mut rb, id).await,
        };
        *self = option_item.ok_or(anyhow!("can't get item"))?;
        self.price = price;
        Item::update_by_column(&mut rb, &self, "id").await?;
        Ok(())
    }

    async fn change_num(&mut self, rb: &mut dyn Executor, num: i64, force: bool) -> Error {
        let option_item = match self.id {
            None => Item::get_by_name(rb, &self.name).await,
            Some(id) => Item::get_by_id(rb, id).await,
        };
        *self = option_item.ok_or(anyhow!("can't get item"))?;
        if force {
            self.remain = num as u64
        } else {
            self.remain = match self.remain.checked_add_signed(num) {
                None => {
                    return Err(anyhow!("not enough item"));
                }
                Some(value) => value,
            };
        }
        Item::update_by_column(rb, &self, "id").await?;
        Ok(())
    }

    pub async fn operate_cart_num(
        &mut self,
        user_id: u64,
        home_id: u64,
        num: i64,
    ) -> Result<Record> {
        let mut tx = get_tx_set_defer(DB.clone()).await?;
        self.change_num(&mut tx, -num, false).await?;
        let item_id = self.id.unwrap();
        let record = match Record::select_by_info(&mut tx, item_id, user_id, home_id).await? {
            None => {
                if num < 0 {
                    return Err(anyhow!("create shouldn't negative"));
                }
                let mut record = Record::new(item_id, user_id, home_id, num as u64);
                record.id = Record::insert(&mut tx, &record)
                    .await?
                    .last_insert_id
                    .as_u64();
                record
            }
            Some(mut record) => {
                record.num = match record.num.checked_add_signed(num) {
                    None => {
                        return Err(anyhow!("not enough item"));
                    }
                    Some(value) => value,
                };
                Record::update_by_column(&mut tx, &record, "id").await?;
                record
            }
        };
        tx.commit().await?;
        Ok(record)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, CreateTable)]
pub struct Record {
    pub id: Option<u64>,
    #[index]
    pub item_id: u64,
    #[index]
    pub user_id: u64,
    #[index]
    pub home_id: u64,
    // cart->pay->sending->sign
    pub status: String,
    pub num: u64,
}

rbatis::crud!(Record {});
rbatis::impl_select!(Record{select_by_info(item_id:u64,user_id:u64,home_id:u64)->Option
    =>"`where item_id = #{item_id} and user_id=#{user_id} and home_id = #{home_id}`"});

impl Record {
    fn new(item_id: u64, user_id: u64, home_id: u64, num: u64) -> Self {
        Self {
            id: None,
            item_id,
            user_id,
            home_id,
            status: "cart".to_string(),
            num,
        }
    }
    util::get!(Record;(id->id_value:u64));
    pub async fn get_by_user_id(user_id: u64) -> Result<Vec<Self>> {
        let mut rb = DB.clone();
        Ok(Record::select_by_column(&mut rb, "user_id", user_id).await?)
    }

    async fn get(&mut self, rb: &mut dyn Executor) -> Result<u64> {
        Ok(match self.id {
            None => {
                let record = Record::select_by_info(rb, self.item_id, self.user_id, self.home_id)
                    .await?
                    .ok_or(anyhow!("can't find record"))?;
                *self = record;
                self.id.unwrap()
            }
            Some(id) => {
                *self = Record::get_by_id(rb, id)
                    .await
                    .ok_or(anyhow!("can't find record"))?;
                id
            }
        })
    }

    pub async fn get_self(&mut self) -> Error {
        self.get(&mut DB.clone()).await?;
        Ok(())
    }

    async fn change_resource<F>(&mut self, change: F) -> Error
    where
        F: FnOnce(&mut Record) -> Error,
    {
        let mut rb = DB.clone();
        self.get(&mut rb).await?;
        change(self)?;
        Record::update_by_column(&mut rb, self, "id").await?;
        Ok(())
    }

    pub async fn change_home(&mut self, home_id: u64) -> Error {
        self.change_resource(|record| {
            if record.status == "sign" || record.status == "sending" {
                return Err(anyhow!("status err,can't change position"));
            }
            record.home_id = home_id;
            Ok(())
        })
        .await
    }

    pub async fn pay(&mut self) -> Error {
        self.change_resource(|record| {
            if record.status != "cart" {
                return Err(anyhow!("can't pay,and status is {}", record.status));
            }
            record.status = "pay".to_string();
            Ok(())
        })
        .await
    }

    pub async fn send(&mut self) -> Error {
        self.change_resource(|record| {
            if record.status != "pay" {
                return Err(anyhow!("can't send,and status is {}", record.status));
            }
            record.status = "send".to_string();
            Ok(())
        })
        .await
    }

    pub async fn sign(&mut self) -> Error {
        self.change_resource(|record| {
            if record.status != "send" {
                return Err(anyhow!("can't sign,and status is {}", record.status));
            }
            record.status = "sign".to_string();
            Ok(())
        })
        .await
    }

    pub async fn force_change_status(&mut self, status: String) -> Error {
        self.change_resource(move |record| {
            record.status = status;
            Ok(())
        })
        .await
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn add_record() {
        let u1: u64 = 1;
        let u2: i64 = -2;
        dbg!(u1.checked_add_signed(u2));
        dbg!(u1);
        dbg!(u2.checked_add_unsigned(u1));
    }
}
