use std::{env, error::Error, fs};

use compiler::{config::Config, input, node_printer, parser};

fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let files = input::get_files(config.source_path.clone());
    for file in files {
        let tokens_result = node_printer::result_to_xml(parser::parse(
            fs::read_to_string(file.as_path())?.as_str(),
        )?);
        let mut target_path = file.clone();
        target_path.set_extension("parser-out.xml");
        fs::write(target_path.as_path(), tokens_result)?;
        println!("Wrote to: {}", target_path.as_path().display());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).map_err(|err| format!("Problem parsing arguments: {}", err))?;

    run(&config).map_err(|err| {
        // Print error manually because `main` func error reporter preseves escapes
        println!("Application error:\n{}", err);
        "Error"
    })?;
    Ok(())
}
