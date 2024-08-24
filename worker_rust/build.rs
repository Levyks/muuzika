fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = [
        "../proto/common.proto",
        "../proto/connection_handler.proto",
    ];

    println!("cargo:rerun-if-changed=../proto");
    for proto in protos {
        println!("cargo:rerun-if-changed={}", proto);
    }

    tonic_build::configure()
        .build_client(false)
        .compile(
            &protos,
            &["../proto"],
        )?;

    Ok(())
}