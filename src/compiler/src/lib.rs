pub mod config;
pub mod token;
pub mod tokenizer;

use std::error::Error;

pub fn run(_config: &config::Config) -> Result<(), Box<dyn Error>> {
    Ok(())
}
