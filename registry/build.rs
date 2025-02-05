use glob::glob;

const PROTO_FOLDER: &str = "../proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let absolute_path = std::fs::canonicalize(PROTO_FOLDER)?;
    let absolute_path = absolute_path.to_str().ok_or("Failed to convert path to string")?;

    let paths = glob(format!("{}/**/*.proto", absolute_path).as_str())?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    tonic_build::configure()
        .build_client(false)
        .build_server(true)
        .compile_protos(&paths, &[absolute_path])?;

    Ok(())
}
