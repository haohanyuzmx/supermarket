use async_trait::async_trait;
use log::error;
use rbatis::executor::RBatisTxExecutorGuard;
use rbatis::Rbatis;

#[macro_export]
macro_rules! insert {
    ($($table:ty),*) => {
        {
            let mut create_map:HashMap<String, String> = HashMap::new();
            $(
            match <$table>::create_table() { (table_name, create_sql) => {create_map.insert(table_name,create_sql);} };
            )*
            create_map
        }
    };
}

pub trait InitTable {
    fn create_table() -> (String, String);
}

#[async_trait]
pub trait InitItem {
    async fn init();
}

#[macro_export]
macro_rules! init {
    ($url:expr;$($table:ty),*;$($init:ty),*) => {
        lazy_static::lazy_static! {
            static ref DB: Rbatis = connect();
        }
        fn connect() -> Rbatis {
            //fast_log::init(fast_log::Config::new().console()).expect("log init fail");
            let rb = Rbatis::new();
            rb.init(MysqlDriver {}, $url)
                .expect("conn to db fail");
            rb
        }
        pub async fn init() {
            let mut should_create = $crate::insert!($($table),*);
            let rb = DB.clone();
            let tables: Vec<HashMap<String, String>> =
                rb.query_decode("show tables", vec![]).await.unwrap();
            for table_names in &tables {
                for (_, table_name) in table_names {
                    should_create.remove(table_name);
                }
            }
            for (_, create_table_sql) in &should_create {
                rb.exec(create_table_sql, vec![])
                    .await
                    .expect("create table err");
            }
            $(
                <$init>::init().await;
            )*
        }
    };
}

pub async fn get_tx_set_defer(db: Rbatis) -> anyhow::Result<RBatisTxExecutorGuard> {
    let tx_no_defer = db.acquire_begin().await?;
    let tx = tx_no_defer.defer_async(|mut tx| async move {
        if !tx.done {
            if let Err(e) = tx.rollback().await {
                error!("run defer rollback err {}", e)
            };
        }
    });
    Ok(tx)
}
