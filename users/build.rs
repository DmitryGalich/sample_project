fn main() -> Result<(), Box<dyn std::error::Error>> {

    tonic_build::configure()
        .compile(
            &["proto/users.proto"],
            &["module_proto"]
        )?;
        
    Ok(())
}
