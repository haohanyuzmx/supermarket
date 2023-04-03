pub mod comment;

use rbatis::Rbatis;
use rbdc_mysql::driver::MysqlDriver;
use std::collections::HashMap;

use util::rbatis::init::InitTable;

type Error = anyhow::Result<()>;

util::init!("mysql://root:123456@localhost:3306/test";
    comment::Comment;);
