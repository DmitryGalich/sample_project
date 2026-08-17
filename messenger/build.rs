fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_proto("../proto/messenger.proto")?;
    Ok(())
}
