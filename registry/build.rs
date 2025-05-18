use glob::glob;

const PROTO_FOLDER: &str = "../proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths = glob(format!("{}/**/*.proto", PROTO_FOLDER).as_str())?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    println!("{:?}", paths);

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&paths, &[PROTO_FOLDER])?;

    Ok(())
}
