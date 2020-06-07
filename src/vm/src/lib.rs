mod code;
pub mod config;
mod instruction;
mod parser;

use std::error::Error;
use std::fs;
use std::path;

pub fn run(config: config::Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(&config.filename)?;
    let mut target_filename = path::PathBuf::from(&config.filename);
    let filename = target_filename.clone();
    let filename_stem = filename
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid filename")?;
    let generated = code::generate_code(parser::create(&contents, filename_stem).parse());
    target_filename.set_extension("asm");
    fs::write(target_filename, generated)?;
    Ok(())
}
