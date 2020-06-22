use std::error::Error;
use std::fs;

use crate::{config::Config, input, tokenizer};

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let files = input::get_files(config.source_path.clone());
    dbg!(&config.source_path);
    println!("Files: {:?}", files);
    for file in files {
        let tokens_result = tokenizer::tokens_to_xml(tokenizer::tokenize(
            fs::read_to_string(file.as_path())?.as_str(),
        )?);
        let mut target_path = file.clone();
        target_path.set_extension("out.xml");
        fs::write(target_path.as_path(), tokens_result)?;
        println!("Wrote to: {}", target_path.as_path().display());
    }
    Ok(())
}
