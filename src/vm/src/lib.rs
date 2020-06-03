pub mod code;
pub mod config;
pub mod instruction;
pub mod parser;

use std::error::Error;
use std::fs;
use std::path;

pub fn run(config: config::Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(&config.filename)?;
    let generated = code::generate_code(parser::parse(contents));
    let mut target_filename = path::PathBuf::from(&config.filename);
    target_filename.set_extension("asm");
    fs::write(target_filename, generated)?;
    Ok(())
}
