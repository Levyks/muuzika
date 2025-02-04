use glob::glob;

const PROTO_FOLDER: &str = "../proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={}", PROTO_FOLDER);

    let paths = glob(format!("{}/**/*.proto", PROTO_FOLDER).as_str())?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    for path in &paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    tonic_build::configure()
        .build_client(false)
        .build_server(true)
        .compile_protos(&paths, &[PROTO_FOLDER])?;

    Ok(())
}
