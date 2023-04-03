use crate::repo::{
    auth::{InsertIntoRole, Job, Role},
    user,
};

pub async fn validate(user_id: u64, url: &str) -> bool {
    let user_role = Role::get_by_user(user_id).await;
    let job_role = Role::get_by_job(url).await;
    if user_role.contains("root") || job_role.contains("normal") {
        return true;
    }
    user_role.into_iter().any(|role| job_role.contains(&role))
}

pub async fn set_job_auth(url: String, role_name: &str) -> anyhow::Result<()> {
    Job::new(url.to_string()).insert_into_role(role_name).await
}

pub async fn set_role_auth(user_name: &str, role_name: &str) -> anyhow::Result<()> {
    user::get_user_by_info(user_name, None)
        .await?
        .insert_into_role(role_name)
        .await
}
