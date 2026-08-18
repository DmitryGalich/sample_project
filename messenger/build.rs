fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/messenger.proto");

    // Явно указываем массив файлов и массив базовых директорий поиска
    tonic_build::configure()
        .compile(
            &["proto/messenger.proto"], // Что компилируем
            &["proto"]                  // Где искать импорты (корневая папка для proto)
        )?;
        
    Ok(())
}
