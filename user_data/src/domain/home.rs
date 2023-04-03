use anyhow::{anyhow, Result};

use crate::repo;

#[derive(Default)]
pub struct Home {
    pub id: u64,
    pub user_id: u64,
    pub home_address: String,
}

impl From<repo::home::Home> for Home {
    fn from(value: repo::home::Home) -> Self {
        Self {
            id: value.id.unwrap(),
            user_id: value.user_id,
            home_address: value.home_address,
        }
    }
}

pub async fn add_home_addr(home: repo::home::Home) -> Result<Home> {
    set_home(None, Some(home)).await
}

pub async fn change_home_addr(old: repo::home::Home, new: repo::home::Home) -> Result<Home> {
    set_home(Some(old), Some(new)).await
}

pub async fn delete_home_addr(home: repo::home::Home) -> Result<Home> {
    set_home(Some(home), None).await
}

async fn set_home(old: Option<repo::home::Home>, new: Option<repo::home::Home>) -> Result<Home> {
    if old.is_none() && new.is_none() {
        return Err(anyhow!("no request"));
    }

    if old.is_none() {
        let mut home = new.unwrap();
        home.create_check().await?;
        return Ok(home.into());
    }
    if new.is_none() {
        let home = old.unwrap();
        home.delete().await?;
        return Ok(Home::default());
    }
    let mut home = old.unwrap();
    home.change_addr(new.unwrap().home_address).await?;
    return Ok(home.into());
}

pub async fn get_all_by_user(user_id: u64) -> Result<Vec<Home>> {
    Ok(repo::home::Home::get_all_by_user(user_id)
        .await?
        .into_iter()
        .map(|home| Home::from(home))
        .collect())
}

pub async fn get_all() -> Result<Vec<Home>> {
    Ok(repo::home::Home::get_all()
        .await?
        .into_iter()
        .map(|home| Home::from(home))
        .collect())
}
