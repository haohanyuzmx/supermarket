fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // let mut include=config_path.clone();
    // include.push("proto");
    // let mut proto=include.clone();
    // proto.push("validate.proto");
    // let mut out=config_path.clone();
    // out.push("src");
    // out.push("pb");
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/pb")
        .compile(&["proto/validate.proto"], &["proto"])
        .unwrap();
    //tonic_build::compile_protos("proto/validate.proto")?;
    Ok(())
}
