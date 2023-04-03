use rbatis::Rbatis;
use rbdc_mysql::driver::MysqlDriver;
use std::collections::HashMap;

use util::rbatis::init::InitTable;

pub mod home;
pub mod wallet;

type Error = anyhow::Result<()>;

util::init!("mysql://root:123456@localhost:3306/test";
    home::Home,wallet::Balance;);
