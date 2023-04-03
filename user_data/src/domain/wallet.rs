use crate::repo::wallet::Balance;
use anyhow::Result;

pub async fn recharge_to_wallet(mut b: Balance, num: u64) -> Result<Balance> {
    b.operate_num(num.try_into()?, false).await?;
    Ok(b)
}

pub async fn cash_out_from_wallet(mut b: Balance, num: u64) -> Result<Balance> {
    b.operate_num(-num.try_into()?, false).await?;
    Ok(b)
}

pub async fn root_operate(mut b: Balance, num: i64, force: bool) -> Result<Balance> {
    b.operate_num(num, force).await?;
    Ok(b)
}
