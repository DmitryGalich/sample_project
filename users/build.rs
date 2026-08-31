use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = "src/generated";
    
    // Создаем папку, если её нет
    if !Path::new(out_dir).exists() {
        fs::create_dir_all(out_dir)?;
    }

    tonic_build::configure()
        .out_dir(out_dir) 
        .compile_protos(&["proto/users.proto"], &["proto"])?;
        
    Ok(())
}
