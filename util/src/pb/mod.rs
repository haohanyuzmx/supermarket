pub mod client;
pub mod home;
pub mod validate;
pub mod wallet;

pub async fn check_url_auth(url: &str, auth: &str) {
    if !client::add_url_auth(url.to_string(), auth.to_string())
        .await
        .expect(&format!("start init url failed,{},{}", url, auth))
    {
        panic!("init url failed,{},{}", url, auth)
    }
}

pub async fn init_url_auth(url_auths: &[(&str, &str)]) {
    for url_auth in url_auths {
        check_url_auth(url_auth.0, url_auth.1).await
    }
}
