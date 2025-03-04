use std::error::Error;
use std::fs;

pub fn run(file: &str) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(file)?;
    println!("{contents}");

    Ok(())
}
