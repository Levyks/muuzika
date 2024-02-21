fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("cargo:rerun-if-changed=../proto");

    tonic_build::configure()
        .build_client(false)
        .compile(
            &[
                "../proto/lobby.proto"
            ],
            &["../proto"]
        )?;
    Ok(())
}