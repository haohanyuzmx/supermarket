use std::collections::HashMap;

use rbatis::Rbatis;
use rbdc_mysql::driver::MysqlDriver;

use util::rbatis::init::{InitItem, InitTable};

pub mod auth;
pub mod user;

util::init!("mysql://root:123456@localhost:3306/test";
    user::User,auth::Role,auth::RoleBindJob,auth::Job,auth::UserBindRole;auth::Role);
