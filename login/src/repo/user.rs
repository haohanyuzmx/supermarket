use std::fmt::{Display, Formatter};
use std::result;

use rbatis::rbdc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use table_rbs::CreateTable;

use crate::repo::DB;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, CreateTable)]
pub struct User {
    pub id: Option<u64>,
    #[index]
    pub user_name: String,
    pub pass_word: String,
}

impl User {
    fn new(user_name: String, pass_word: String) -> Self {
        Self {
            id: None,
            user_name,
            pass_word,
        }
    }
}

rbatis::crud!(User {});
rbatis::impl_select!(User{select_by_info(name:&str,password:&str)->Option
    =>"`where user_name = #{name} and pass_word = #{password}`"});

pub type Result<T> = result::Result<T, DBExecErr>;

#[derive(Debug, Error)]
pub enum DBExecErr {
    UserExist,
    UserNotFound,
    ExecErr(#[from] rbdc::Error),
}

impl Display for DBExecErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "db exec err {:?}", self)
    }
}

pub async fn create_user(user_name: String, pass_word: String) -> Result<User> {
    let mut user = User::new(user_name, pass_word);
    let rb = DB.clone();
    let tx_no_defer = rb.acquire_begin().await?;
    let mut tx = tx_no_defer.defer_async(|mut tx| async move {
        if !tx.done {
            if let Err(e) = tx.rollback().await {
                fast_log::print(format!("defer fun call rollback err {}", e)).unwrap_or(());
            };
        }
    });
    //TODO: 使用索引查询，指定查询字段而不是*
    if let Ok(u) = User::select_by_column(&mut tx, "user_name", user.user_name.as_str()).await {
        if u.len() != 0 {
            return Err(DBExecErr::UserExist);
        }
    }

    user.id = User::insert(&mut tx, &user).await?.last_insert_id.as_u64();
    tx.commit().await?;
    Ok(user)
}

pub async fn get_user_by_info(user_name: &str, pass_word: Option<&str>) -> Result<User> {
    let mut rb = DB.clone();
    match pass_word {
        None => User::select_by_column(&mut rb, "user_name", user_name)
            .await?
            .pop()
            .ok_or_else(|| DBExecErr::UserNotFound),

        Some(pass_word) => User::select_by_info(&mut rb, user_name, pass_word)
            .await?
            .ok_or_else(|| DBExecErr::UserNotFound),
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::thread::sleep;
    use std::time::Duration;

    use crate::repo::init;

    use super::*;

    #[test]
    fn select() {
        tokio_test::block_on(async {
            init().await;
            //let user=create_user("333".to_string(),"333".to_string()).await?;
            match create_user("333".to_string(), "333".to_string())
                .await
                .err()
            {
                Some(DBExecErr::UserExist) => {}
                _ => {
                    panic!("there is user--233")
                }
            }
        })
    }

    #[test]
    fn delete() {
        tokio_test::block_on(async {
            init().await;
            let u = User {
                id: Some(1),
                user_name: "733".to_string(),
                pass_word: "433".to_string(),
            };
            let user = get_user_by_info(&u.user_name, Some(&u.pass_word))
                .await
                .unwrap();
            assert_eq!(u, user)
        })
    }
}
