use std::result;

use chrono::Duration;
use lazy_static::lazy_static;
use redis::AsyncCommands;
use thiserror::Error;

lazy_static! {
    static ref REDIS_CLINET: redis::Client = redis::Client::open("redis://127.0.0.1/").unwrap();
}

type Result<T> = result::Result<T, CacheErr>;

#[derive(Error, Debug)]
pub enum CacheErr {
    #[error("get conn err,{0}")]
    GetConErr(redis::RedisError),
    #[error("exec command err,{0}")]
    ExecErr(#[from] redis::RedisError),
}

async fn get_con() -> Result<redis::aio::Connection> {
    REDIS_CLINET
        .get_async_connection()
        .await
        .map_err(|e| CacheErr::GetConErr(e))
}

pub async fn set_rt_with_ttl(rt: &str, user_info: &str, timeout: Duration) -> Result<()> {
    get_con()
        .await?
        .set_ex(rt, user_info, timeout.num_seconds() as usize)
        .await?;
    Ok(())
}

pub async fn get_user_by_rt(rt: &str) -> Result<String> {
    let str: String = get_con().await?.get(rt).await?;
    Ok(str)
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;

    use super::*;

    #[test]
    fn set_rt() {
        tokio_test::block_on(async {
            set_rt_with_ttl("123", "233", Duration::seconds(1))
                .await
                .unwrap();
            assert_eq!(get_user_by_rt("123").await.unwrap(), "233");
            sleep(std::time::Duration::new(1, 0));
            println!("{:?}", get_user_by_rt("123").await)
        })
    }
}
