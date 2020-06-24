use std::env;
use std::error::Error;
use std::fs;

use compiler::{config::Config, input, tokenizer};

fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let files = input::get_files(config.source_path.clone());
    // dbg!(&config.source_path);
    // println!("Files: {:?}", files);
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).map_err(|err| format!("Problem parsing arguments: {}", err))?;

    run(&config).map_err(|err| format!("Application error: {}", err).into())
}
