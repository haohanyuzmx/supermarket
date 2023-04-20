macro_rules! jude {
    ([$now:expr]=>!$should1:expr,$($should2:expr),+)=>{
        jude!([$now]=>!$should1)&&jude!([$now]=>!$($should2),+)
    };
    ([$now:expr]=>!$should:expr)=>{
        $now!=$should
    };
}

fn main() {
    let a = jude!([1]=>!2,3,4,5);
    println!("Hello, world!");
}
