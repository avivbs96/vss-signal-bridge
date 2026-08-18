fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the vendored protoc so the build has no system dependency.
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
    tonic_build::configure()
        .build_server(false)
        .compile_protos(&["proto/kuksa/val/v1/val.proto"], &["proto"])?;
    Ok(())
}
