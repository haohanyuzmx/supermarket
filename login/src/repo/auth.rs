use std::collections::HashSet;
use std::hash::Hash;
use std::vec;

use async_trait::async_trait;
use rbatis::Rbatis;
use serde::{Deserialize, Serialize};

use table_rbs::CreateTable;

use crate::repo::user::{get_user_by_info, User};
use crate::repo::DB;

trait GetRoleID {
    fn get_role_id(&self) -> u64;
}

macro_rules! impl_get_role_id {
    ($ty:ty) => {
        impl GetRoleID for $ty {
            fn get_role_id(&self) -> u64 {
                self.role_id
            }
        }
    };
}

//TODO:直接生成二级的函数
macro_rules! get_fn {
    ($rb_type:ty;($column_name:ident->$column_value:ident:$column_type:ty);$item:expr) => {
        #[allow(dead_code)]
        async fn get(
            rb: &mut dyn rbatis::executor::Executor,
            $column_name: &str,
            $column_value: $column_type,
        ) -> Option<Self> {
            match <$rb_type>::select_by_column(rb, $column_name, $column_value).await {
                Ok(mut maybe) => maybe.pop(),
                _ => None,
            }
        }
        #[allow(dead_code)]
        async fn get_or_insert(
            rb: &mut dyn rbatis::executor::Executor,
            $column_name: &str,
            $column_value: $column_type,
        ) -> anyhow::Result<Self> {
            let mut result = <$rb_type>::select_by_column(rb, $column_name, $column_value).await?;
            if result.len() == 1 {
                return Ok(result.pop().unwrap());
            }
            let mut item = $item;
            item.id = <$rb_type>::insert(rb, &item).await?.last_insert_id.as_u64();
            Ok(item)
        }
    };
}
//TODO:事务
#[async_trait]
pub trait InsertIntoRole {
    async fn insert_into_role(&mut self, role_name: &str) -> anyhow::Result<()>;
}
#[async_trait]
impl InsertIntoRole for Job {
    async fn insert_into_role(&mut self, role_name: &str) -> anyhow::Result<()> {
        let mut rb = DB.clone();
        let rb_caller = &mut rb;
        self.id = Job::get_or_insert_by_url(rb_caller, &self.url).await?.id;
        let role = Role::get_or_insert_by_name(rb_caller, role_name).await?;
        RoleBindJob::new(role.id.unwrap(), self.id.unwrap())
            .get_or_insert(rb_caller)
            .await
    }
}

#[async_trait]
impl InsertIntoRole for User {
    async fn insert_into_role(&mut self, role_name: &str) -> anyhow::Result<()> {
        let mut rb = DB.clone();
        let rb_caller = &mut rb;
        let role = Role::get_or_insert_by_name(rb_caller, role_name).await?;
        //TODO:默认Self可以get_id？
        UserBindRole::new(self.id.unwrap(), role.id.unwrap())
            .get_or_insert(rb_caller)
            .await
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, Hash, PartialEq, Eq, CreateTable)]
pub struct Role {
    pub id: Option<u64>,
    #[index]
    pub role_name: String,
}

rbatis::crud!(Role {});

impl Role {
    fn new(name: String) -> Self {
        Self {
            id: None,
            role_name: name,
        }
    }
    pub async fn get_by_job(url: &str) -> HashSet<String> {
        let mut rb = DB.clone();
        let vec_job = Job::select_by_column(&mut rb, "url", url)
            .await
            .unwrap_or(vec![]);
        if vec_job.len() == 0 {
            return HashSet::new();
        }
        let role_job = RoleBindJob::select_by_column(&mut rb, "job_id", vec_job[0].id.unwrap())
            .await
            .unwrap_or(vec![]);
        Role::get_role_by_vecid(&mut rb, role_job).await
    }
    pub async fn get_by_user(user_id: u64) -> HashSet<String> {
        let mut rb = DB.clone();
        let user_role = UserBindRole::select_by_column(&mut rb, "user_id", user_id)
            .await
            .unwrap_or(vec![]);
        Role::get_role_by_vecid(&mut rb, user_role).await
    }
    async fn get_role_by_vecid(rb: &mut Rbatis, role_ids: Vec<impl GetRoleID>) -> HashSet<String> {
        let mut result = HashSet::new();
        let role_id = role_ids.into_iter().map(|bind| bind.get_role_id());
        for id in role_id {
            Role::select_by_column(rb, "id", id)
                .await
                .unwrap_or(vec![])
                .into_iter()
                .for_each(|role| {
                    result.insert(role.role_name);
                })
        }
        result
    }
    get_fn!(Role;(role_name->role_name_value:&str);Role::new(role_name_value.to_string()));
    async fn get_or_insert_by_name(rb: &mut Rbatis, name: &str) -> anyhow::Result<Self> {
        Role::get_or_insert(rb, "role_name", name).await
    }
}

#[async_trait]
impl util::rbatis::init::InitItem for Role {
    async fn init() {
        let mut rb = DB.clone();
        let role_names = vec!["root", "normal", "worker", "user"];
        for name in role_names {
            Role::get_or_insert_by_name(&mut rb, name)
                .await
                .expect(&format!("init err {}", name));
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, CreateTable)]
pub struct Job {
    pub id: Option<u64>,
    #[index]
    pub url: String,
}

rbatis::crud!(Job {});

impl Job {
    pub fn new(url: String) -> Self {
        Self { id: None, url }
    }
    get_fn!(Job;(url->url_value : &str);Job::new(url_value.to_string()));
    async fn get_or_insert_by_url(rb: &mut Rbatis, url: &str) -> anyhow::Result<Self> {
        Job::get_or_insert(rb, "url", url).await
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, CreateTable)]
pub struct RoleBindJob {
    pub id: Option<u64>,
    #[index]
    pub role_id: u64,
    #[index]
    pub job_id: u64,
}

impl_get_role_id!(RoleBindJob);
rbatis::crud!(RoleBindJob {});
rbatis::impl_select!(RoleBindJob{select_by_info(role_id:u64,job_id:u64)->Option
    =>"`where role_id = #{role_id} and job_id = #{job_id}`"});

impl RoleBindJob {
    fn new(role_id: u64, job_id: u64) -> Self {
        Self {
            id: None,
            role_id,
            job_id,
        }
    }
    async fn get_or_insert(&mut self, rb: &mut Rbatis) -> anyhow::Result<()> {
        if let Some(role_job) = RoleBindJob::select_by_info(rb, self.role_id, self.job_id).await? {
            self.id = role_job.id;
            return Ok(());
        }
        self.id = RoleBindJob::insert(rb, self).await?.last_insert_id.as_u64();
        return Ok(());
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, CreateTable)]
pub struct UserBindRole {
    pub id: Option<u64>,
    #[index]
    pub user_id: u64,
    #[index]
    pub role_id: u64,
}

#[async_trait]
impl util::rbatis::init::InitItem for UserBindRole {
    async fn init() {
        let user_roles = vec![("333", "root"), ("hhh", "normal"), ("worker", "worker")];
        for (user_name, role_name) in user_roles {
            get_user_by_info(user_name, None)
                .await
                .unwrap()
                .insert_into_role(role_name)
                .await
                .unwrap();
        }
    }
}

impl_get_role_id!(UserBindRole);
rbatis::crud!(UserBindRole {});
rbatis::impl_select!(UserBindRole{select_by_info(user_id:u64,role_id:u64)->Option
    =>"`where user_id = #{user_id} and role_id = #{role_id}`"});

impl UserBindRole {
    fn new(user_id: u64, role_id: u64) -> Self {
        Self {
            id: None,
            user_id,
            role_id,
        }
    }
    async fn get_or_insert(&mut self, rb: &mut Rbatis) -> anyhow::Result<()> {
        if let Some(user_role) =
            UserBindRole::select_by_info(rb, self.user_id, self.role_id).await?
        {
            self.id = user_role.id;
            return Ok(());
        }
        self.id = UserBindRole::insert(rb, self)
            .await?
            .last_insert_id
            .as_u64();
        return Ok(());
    }
}
