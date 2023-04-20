use rbatis::Rbatis;
use rbdc_mysql::driver::MysqlDriver;
use std::collections::HashMap;

use util::rbatis::init::InitTable;

pub mod item;

util::init!("mysql://root:123456@localhost:3306/test";
    item::Item,item::Record;);
