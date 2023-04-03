#[macro_export]
macro_rules! get {
    ($rb_type:ty;($column_name:ident->$column_value:ident:$column_type:ty)) => {
        paste::paste! {
            #[allow(dead_code)]
            async fn [<get_by_$column_name>](
                rb: &mut dyn rbatis::executor::Executor,
                $column_value: $column_type,
            ) -> Option<Self> {
                match <$rb_type>::select_by_column(rb, &stringify!($column_name), $column_value).await {
                    Ok(mut maybe) => maybe.pop(),
                    _ => None,
                }
            }
        }
    };
    (pub($rb:expr) $rb_type:ty;($column_name:ident->$column_value:ident:$column_type:ty)) => {
        paste::paste! {
            #[allow(dead_code)]
            pub async fn [<select_by_$column_name>](
                $column_value: $column_type,
            ) -> Option<Self> {
                match <$rb_type>::select_by_column(&mut $rb.clone(), &stringify!($column_name), $column_value).await {
                    Ok(mut maybe) => maybe.pop(),
                    _ => None,
                }
            }
        }
    };
}
