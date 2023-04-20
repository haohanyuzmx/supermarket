use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use tokio::process::Command;
use tokio::spawn;
use tokio::sync::mpsc::{channel, Sender};

#[tokio::main]
async fn main() {
    let (err_sender, mut err_receiver) = channel(4);
    let run_bin = |bin: &'static str| run_bin(err_sender.clone(), bin);
    run_bin("login");
    run_bin("user_data");
    run_bin("sale");
    run_bin("comment");
    err_receiver.recv().await.expect("impossible");
}

fn run_bin(err_sender: Sender<()>, bin: &'static str) {
    spawn(async move {
        let result = Command::new("cargo")
            .args(["run", "--bin", bin, "--", &format!("-r=./{}.log", bin)])
            .output()
            .await;
        err_sender.send(()).await.expect("send impossible");
        println!("{:?} bin {:?}", bin, result)
    });
    sleep(Duration::from_secs_f32(0.5))
}
