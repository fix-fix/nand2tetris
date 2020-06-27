use std::env;
use std::error::Error;
use std::fs;

use compiler::{config::Config, input, parser};

fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let files = input::get_files(config.source_path.clone());
    for file in files {
        let tokens_result =
            parser::result_to_xml(parser::parse(fs::read_to_string(file.as_path())?.as_str())?);
        let mut target_path = file.clone();
        target_path.set_extension("parser-out.xml");
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
