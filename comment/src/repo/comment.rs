use super::Error;
use crate::domain::comment::CommentNode;
use crate::repo::DB;
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use rbatis::{crud, impl_select};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use table_rbs::CreateTable;
use util::get;
use util::rbatis::init::{get_tx_set_defer, InitItem};

// TODO:所有相关接口
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, CreateTable)]
pub struct Comment {
    pub id: Option<u64>,
    #[index]
    pub user_id: u64,
    pub user_name: String,
    pub father_comment: u64,
    #[index]
    pub item_id: u64,
    pub comment: String,
}

crud!(Comment {});

impl Comment {
    pub fn new(
        comment: String,
        user_name: String,
        user_id: u64,
        item_id: u64,
        father: Option<u64>,
    ) -> Self {
        Self {
            user_id,
            user_name,
            father_comment: father.unwrap_or(0),
            item_id,
            comment,
            ..Default::default()
        }
    }
    get!(pub(DB) Comment;(id->id_value:u64));

    pub async fn get_by_item_id(item_id: u64) -> Result<HashMap<u64, Vec<Comment>>> {
        Ok(
            Comment::select_by_column(&mut DB.clone(), "item_id", item_id)
                .await?
                .into_iter()
                .fold(HashMap::new(), |mut map, item| {
                    map.entry(item.father_comment).or_insert(vec![]).push(item);
                    map
                }),
        )
    }

    pub async fn get_by_user_id(user_id: u64) -> Result<Vec<Comment>> {
        Ok(Comment::select_by_column(&mut DB.clone(), "user_id", user_id).await?)
    }

    pub async fn create(&mut self) -> Error {
        self.id = Comment::insert(&mut DB.clone(), self)
            .await?
            .last_insert_id
            .as_u64();
        Ok(())
    }

    pub async fn change(&mut self, comment_str: String) -> Result<()> {
        self.comment = comment_str;
        Comment::update_by_column(&mut DB.clone(), self, "id").await?;
        Ok(())
    }
}
