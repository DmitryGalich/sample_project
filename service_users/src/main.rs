use std::env;

fn main() {
    let key = "EXPOSED_ADDR";

    match env::var(key) {
        Ok(val) => println!("{key} is set to: {val}"),
        Err(e) => println!("Couldn't read {key}: {e}"),
    }
}
