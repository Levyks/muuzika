use glob::glob;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../proto");

    let files = glob("../proto/**/*.proto")?
        .map(|entry| entry.unwrap())
        .collect::<Vec<_>>();

    tonic_build::configure()
        .build_client(false)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&files, &["../proto"])?;

    Ok(())
}
