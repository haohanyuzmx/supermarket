use clap::Parser;
use std::fs::File;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    pub rust_log_file: Option<String>,
}

pub fn init() {
    let args = Args::parse();
    if args.rust_log_file.is_some() {
        let file = args.rust_log_file.unwrap();
        let out = File::options()
            .write(true)
            .create(true)
            .append(false)
            .open(file)
            .expect("impossible");
        tracing_subscriber::fmt()
            .with_writer(out)
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }
}
